use crate::ffi::CStr;
use crate::io::{self, IoSlice, IoSliceMut};
use crate::mem;
use crate::net::{SocketAddr, Shutdown};
use crate::str;
use crate::sys::net_fd::NetFileDesc;
use crate::sys_common::{AsInner, FromInner, IntoInner};
use crate::sys_common::net::{getsockopt, setsockopt, sockaddr_to_addr};
use crate::time::{Duration, Instant};
use crate::cmp;

use libc::{c_int, c_void, size_t, sockaddr, socklen_t, EAI_SYSTEM, MSG_PEEK};

pub use crate::sys::{cvt, cvt_r};

pub mod netc {
    #[cfg(target_os = "freertos")]
    mod lwip {
        pub use libc::{
            c_int, c_void, c_char, c_long, c_ulong, size_t, ssize_t, AF_INET, AF_INET6, sa_family_t, in_addr, sockaddr_in, in6_addr,
            sockaddr_in6, sockaddr, socklen_t, IPPROTO_IP, IPV6_JOIN_GROUP, IPPROTO_IPV6, IP_TTL,
            ipv6_mreq, ip_mreq, IP_ADD_MEMBERSHIP, IPV6_MULTICAST_LOOP, IP_DROP_MEMBERSHIP, IP_MULTICAST_LOOP,
            IP_MULTICAST_TTL, SO_BROADCAST, SOL_SOCKET, SO_SNDTIMEO, SO_RCVTIMEO, SOCK_DGRAM, sockaddr_storage,
            IPV6_V6ONLY, SOCK_STREAM, SO_REUSEADDR, addrinfo, IPV6_LEAVE_GROUP, IPV6_DROP_MEMBERSHIP, IPV6_ADD_MEMBERSHIP,
            iovec,
        };

        extern "C" {
            #[link_name = "lwip_close"]
            pub fn close(s: c_int) -> ssize_t;
            #[link_name = "lwip_read"]
            pub fn read(s: c_int, mem: *mut c_void, len: size_t) -> ssize_t;
            #[link_name = "lwip_readv"]
            pub fn readv(s: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t;
            #[link_name = "lwip_write"]
            pub fn write(s: c_int, dataptr: *const c_void, len: size_t) -> ssize_t;
            #[link_name = "lwip_writev"]
            pub fn writev(s: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t;
            #[link_name = "lwip_accept"]
            pub fn accept(s: c_int, addr: *mut sockaddr, addrlen: *mut socklen_t) -> c_int;
            #[link_name = "lwip_bind"]
            pub fn bind(s: c_int, name: *const sockaddr, namelen: socklen_t) -> c_int;
            #[link_name = "lwip_connect"]
            pub fn connect(s: c_int, name: *const sockaddr, namelen: socklen_t) -> c_int;
            #[link_name = "lwip_ioctl"]
            pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
            #[link_name = "lwip_getaddrinfo"]
            pub fn getaddrinfo(nodename: *const c_char, servname: *const c_char, hints: *const addrinfo, res: *mut *mut addrinfo) -> c_int;
            #[link_name = "lwip_freeaddrinfo"]
            pub fn freeaddrinfo(ai: *mut addrinfo);
            #[link_name = "lwip_getsockname"]
            pub fn getsockname(s: c_int, name: *mut sockaddr, namelen: *mut socklen_t) -> c_int;
            #[link_name = "lwip_getpeername"]
            pub fn getpeername(s: c_int, name: *mut sockaddr, namelen: *mut socklen_t) -> c_int;
            #[link_name = "lwip_listen"]
            pub fn listen(s: c_int, backlog: c_int) -> c_int;
            #[link_name = "lwip_send"]
            pub fn send(s: c_int, dataptr: *const c_void, size: size_t, flags: c_int) -> ssize_t;
            #[link_name = "lwip_sendto"]
            pub fn sendto(s: c_int, dataptr: *const c_void, size: size_t, flags: c_int, to: *const sockaddr, tolen: socklen_t) -> ssize_t;
            #[link_name = "lwip_recv"]
            pub fn recv(s: c_int, mem: *mut c_void, len: size_t, flags: c_int) -> ssize_t;
            #[link_name = "lwip_recvfrom"]
            pub fn recvfrom(s: c_int, mem: *mut c_void, len: size_t, flags: c_int, from: *mut sockaddr, fromlen: *mut socklen_t) -> ssize_t;
            #[link_name = "lwip_getsockopt"]
            pub fn getsockopt(s: c_int, level: c_int, optname: c_int, optval: *mut c_void, optlen: *mut socklen_t) -> c_int;
            #[link_name = "lwip_setsockopt"]
            pub fn setsockopt(s: c_int, level: c_int, optname: c_int, optval: *const c_void, optlen: socklen_t) -> c_int;
            #[link_name = "lwip_shutdown"]
            pub fn shutdown(s: c_int, how: c_int) -> c_int;
            #[link_name = "lwip_socket"]
            pub fn socket(domain: c_int, r#type: c_int, protocol: c_int) -> c_int;
        }
    }

    #[cfg(target_os = "freertos")]
    pub use lwip::*;

    #[cfg(not(target_os = "freertos"))]
    pub use libc::*;
}

pub type wrlen_t = size_t;

// See below for the usage of SOCK_CLOEXEC, but this constant is only defined on
// Linux currently (e.g., support doesn't exist on other platforms). In order to
// get name resolution to work and things to compile we just define a dummy
// SOCK_CLOEXEC here for other platforms. Note that the dummy constant isn't
// actually ever used (the blocks below are wrapped in `if cfg!` as well.
#[cfg(target_os = "linux")]
use libc::SOCK_CLOEXEC;
#[cfg(not(target_os = "linux"))]
const SOCK_CLOEXEC: c_int = 0;

// Another conditional constant for name resolution: Macos et iOS use
// SO_NOSIGPIPE as a setsockopt flag to disable SIGPIPE emission on socket.
// Other platforms do otherwise.
#[cfg(target_vendor = "apple")]
use libc::SO_NOSIGPIPE;
#[cfg(not(target_vendor = "apple"))]
const SO_NOSIGPIPE: c_int = 0;

pub struct Socket(NetFileDesc);

pub fn init() {}

pub fn cvt_gai(err: c_int) -> io::Result<()> {
    if err == 0 {
        return Ok(())
    }

    if err == EAI_SYSTEM {
        return Err(io::Error::last_os_error())
    }

    #[cfg(target_os = "freertos")]
    let detail = format!("error code {}", err);

    #[cfg(not(target_os = "freertos"))]
    let detail = unsafe {
        str::from_utf8(CStr::from_ptr(libc::gai_strerror(err)).to_bytes()).unwrap()
            .to_owned()
    };
    Err(io::Error::new(io::ErrorKind::Other,
                       &format!("failed to lookup address information: {}",
                                detail)[..]))
}

impl Socket {
    pub fn new(addr: &SocketAddr, ty: c_int) -> io::Result<Socket> {
        let fam = match *addr {
            SocketAddr::V4(..) => libc::AF_INET,
            SocketAddr::V6(..) => libc::AF_INET6,
        };
        Socket::new_raw(fam, ty)
    }

    pub fn new_raw(fam: c_int, ty: c_int) -> io::Result<Socket> {
        let fd = cvt(unsafe { netc::socket(fam, ty, 0) })?;
        let fd = NetFileDesc::new(fd);

        let socket = Socket(fd);
        Ok(socket)
    }

    pub fn new_pair(fam: c_int, ty: c_int) -> io::Result<(Socket, Socket)> {
        crate::sys::unsupported()
    }

    pub fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> io::Result<()> {
        self.set_nonblocking(true)?;
        let r = unsafe {
            let (addrp, len) = addr.into_inner();
            cvt(netc::connect(self.0.raw(), addrp, len))
        };
        self.set_nonblocking(false)?;

        match r {
            Ok(_) => return Ok(()),
            // there's no ErrorKind for EINPROGRESS :(
            Err(ref e) if e.raw_os_error() == Some(libc::EINPROGRESS) => {}
            Err(e) => return Err(e),
        }

        let mut pollfd = libc::pollfd {
            fd: self.0.raw(),
            events: libc::POLLOUT,
            revents: 0,
        };

        if timeout.as_secs() == 0 && timeout.subsec_nanos() == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                      "cannot set a 0 duration timeout"));
        }

        let start = Instant::now();

        loop {
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "connection timed out"));
            }

            let timeout = timeout - elapsed;
            let mut timeout = timeout.as_secs()
                .saturating_mul(1_000)
                .saturating_add(timeout.subsec_nanos() as u64 / 1_000_000);
            if timeout == 0 {
                timeout = 1;
            }

            let timeout = cmp::min(timeout, c_int::max_value() as u64) as c_int;

            match unsafe { libc::poll(&mut pollfd, 1, timeout) } {
                -1 => {
                    let err = io::Error::last_os_error();
                    if err.kind() != io::ErrorKind::Interrupted {
                        return Err(err);
                    }
                }
                0 => {}
                _ => {
                    // linux returns POLLOUT|POLLERR|POLLHUP for refused connections (!), so look
                    // for POLLHUP rather than read readiness
                    if pollfd.revents & libc::POLLHUP != 0 {
                        let e = self.take_error()?
                            .unwrap_or_else(|| {
                                io::Error::new(io::ErrorKind::Other, "no error set after POLLHUP")
                            });
                        return Err(e);
                    }

                    return Ok(());
                }
            }
        }
    }

    pub fn accept(&self, storage: *mut sockaddr, len: *mut socklen_t)
                  -> io::Result<Socket> {
        // Unfortunately the only known way right now to accept a socket and
        // atomically set the CLOEXEC flag is to use the `accept4` syscall on
        // Linux. This was added in 2.6.28, however, and because we support
        // 2.6.18 we must detect this support dynamically.
        if cfg!(target_os = "linux") {
            syscall! {
                fn accept4(
                    fd: c_int,
                    addr: *mut sockaddr,
                    addr_len: *mut socklen_t,
                    flags: c_int
                ) -> c_int
            }
            let res = cvt_r(|| unsafe {
                accept4(self.0.raw(), storage, len, SOCK_CLOEXEC)
            });
            match res {
                Ok(fd) => return Ok(Socket(NetFileDesc::new(fd))),
                Err(ref e) if e.raw_os_error() == Some(libc::ENOSYS) => {}
                Err(e) => return Err(e),
            }
        }

        let fd = cvt_r(|| unsafe {
            netc::accept(self.0.raw(), storage, len)
        })?;
        let fd = NetFileDesc::new(fd);

        Ok(Socket(fd))
    }

    pub fn duplicate(&self) -> io::Result<Socket> {
        crate::sys::unsupported()
    }

    fn recv_with_flags(&self, buf: &mut [u8], flags: c_int) -> io::Result<usize> {
        let ret = cvt(unsafe {
            netc::recv(self.0.raw(),
                       buf.as_mut_ptr() as *mut c_void,
                       buf.len(),
                       flags)
        })?;
        Ok(ret as usize)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv_with_flags(buf, 0)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv_with_flags(buf, MSG_PEEK)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    fn recv_from_with_flags(&self, buf: &mut [u8], flags: c_int)
                            -> io::Result<(usize, SocketAddr)> {
        let mut storage: libc::sockaddr_storage = unsafe { mem::zeroed() };
        let mut addrlen = mem::size_of_val(&storage) as libc::socklen_t;

        let n = cvt(unsafe {
            netc::recvfrom(self.0.raw(),
                        buf.as_mut_ptr() as *mut c_void,
                        buf.len(),
                        flags,
                        &mut storage as *mut _ as *mut _,
                        &mut addrlen)
        })?;
        Ok((n as usize, sockaddr_to_addr(&storage, addrlen as usize)?))
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, 0)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, MSG_PEEK)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    pub fn set_timeout(&self, dur: Option<Duration>, kind: libc::c_int) -> io::Result<()> {
        let timeout = match dur {
            Some(dur) => {
                if dur.as_secs() == 0 && dur.subsec_nanos() == 0 {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                              "cannot set a 0 duration timeout"));
                }

                let secs = if dur.as_secs() > libc::time_t::max_value() as u64 {
                    libc::time_t::max_value()
                } else {
                    dur.as_secs() as libc::time_t
                };
                let mut timeout = libc::timeval {
                    tv_sec: secs,
                    tv_usec: (dur.subsec_nanos() / 1000) as libc::suseconds_t,
                };
                if timeout.tv_sec == 0 && timeout.tv_usec == 0 {
                    timeout.tv_usec = 1;
                }
                timeout
            }
            None => {
                libc::timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                }
            }
        };
        setsockopt(self, libc::SOL_SOCKET, kind, timeout)
    }

    pub fn timeout(&self, kind: libc::c_int) -> io::Result<Option<Duration>> {
        let raw: libc::timeval = getsockopt(self, libc::SOL_SOCKET, kind)?;
        if raw.tv_sec == 0 && raw.tv_usec == 0 {
            Ok(None)
        } else {
            let sec = raw.tv_sec as u64;
            let nsec = (raw.tv_usec as u32) * 1000;
            Ok(Some(Duration::new(sec, nsec)))
        }
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let how = match how {
            Shutdown::Write => libc::SHUT_WR,
            Shutdown::Read => libc::SHUT_RD,
            Shutdown::Both => libc::SHUT_RDWR,
        };
        cvt(unsafe { netc::shutdown(self.0.raw(), how) })?;
        Ok(())
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        setsockopt(self, libc::IPPROTO_TCP, libc::TCP_NODELAY, nodelay as c_int)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        let raw: c_int = getsockopt(self, libc::IPPROTO_TCP, libc::TCP_NODELAY)?;
        Ok(raw != 0)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        let mut nonblocking = nonblocking as libc::c_int;
        cvt(unsafe { netc::ioctl(*self.as_inner(), libc::FIONBIO, &mut nonblocking) })
            .map(|_| ())
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        let raw: c_int = getsockopt(self, libc::SOL_SOCKET, libc::SO_ERROR)?;
        if raw == 0 {
            Ok(None)
        } else {
            Ok(Some(io::Error::from_raw_os_error(raw as i32)))
        }
    }
}

impl AsInner<c_int> for Socket {
    fn as_inner(&self) -> &c_int { self.0.as_inner() }
}

impl FromInner<c_int> for Socket {
    fn from_inner(fd: c_int) -> Socket { Socket(NetFileDesc::new(fd)) }
}

impl IntoInner<c_int> for Socket {
    fn into_inner(self) -> c_int { self.0.into_raw() }
}
