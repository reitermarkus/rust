use crate::convert::TryFrom;
use crate::fmt;
use crate::io::{self, IoSlice, IoSliceMut};
use crate::net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr};
use crate::sys::unsupported;
use crate::time::Duration;
#[allow(unused_extern_crates)] pub extern crate libc as netc;
pub struct TcpStream(!);

impl TcpStream {
    pub fn connect(_: io::Result<&SocketAddr>) -> io::Result<TcpStream> {
        unsupported()
    }

    pub fn connect_timeout(_: &SocketAddr, _: Duration) -> io::Result<TcpStream> {
        unsupported()
    }

    pub fn set_read_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        !
    }

    pub fn set_write_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        !
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        !
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        !
    }

    pub fn peek(&self, _: &mut [u8]) -> io::Result<usize> {
        !
    }

    pub fn read(&self, _: &mut [u8]) -> io::Result<usize> {
        !
    }

    pub fn read_vectored(&self, _: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        !
    }

    pub fn write(&self, _: &[u8]) -> io::Result<usize> {
        !
    }

    pub fn write_vectored(&self, _: &[IoSlice<'_>]) -> io::Result<usize> {
        !
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        !
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        !
    }

    pub fn shutdown(&self, _: Shutdown) -> io::Result<()> {
        !
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        !
    }

    pub fn set_nodelay(&self, _: bool) -> io::Result<()> {
        !
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        !
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        !
    }

    pub fn ttl(&self) -> io::Result<u32> {
        !
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        !
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        !
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        !
    }
}

pub struct TcpListener(!);

impl TcpListener {
    pub fn bind(_: io::Result<&SocketAddr>) -> io::Result<TcpListener> {
        unsupported()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        !
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        !
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        !
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        !
    }

    pub fn ttl(&self) -> io::Result<u32> {
        !
    }

    pub fn set_only_v6(&self, _: bool) -> io::Result<()> {
        !
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        !
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        !
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        !
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        !
    }
}

pub struct UdpSocket(!);

impl UdpSocket {
    pub fn bind(_: io::Result<&SocketAddr>) -> io::Result<UdpSocket> {
        unsupported()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        !
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        !
    }

    pub fn recv_from(&self, _: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        !
    }

    pub fn peek_from(&self, _: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        !
    }

    pub fn send_to(&self, _: &[u8], _: &SocketAddr) -> io::Result<usize> {
        !
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        !
    }

    pub fn set_read_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        !
    }

    pub fn set_write_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        !
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        !
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        !
    }

    pub fn set_broadcast(&self, _: bool) -> io::Result<()> {
        !
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        !
    }

    pub fn set_multicast_loop_v4(&self, _: bool) -> io::Result<()> {
        !
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        !
    }

    pub fn set_multicast_ttl_v4(&self, _: u32) -> io::Result<()> {
        !
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        !
    }

    pub fn set_multicast_loop_v6(&self, _: bool) -> io::Result<()> {
        !
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        !
    }

    pub fn join_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr) -> io::Result<()> {
        !
    }

    pub fn join_multicast_v6(&self, _: &Ipv6Addr, _: u32) -> io::Result<()> {
        !
    }

    pub fn leave_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr) -> io::Result<()> {
        !
    }

    pub fn leave_multicast_v6(&self, _: &Ipv6Addr, _: u32) -> io::Result<()> {
        !
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        !
    }

    pub fn ttl(&self) -> io::Result<u32> {
        !
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        !
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        !
    }

    pub fn recv(&self, _: &mut [u8]) -> io::Result<usize> {
        !
    }

    pub fn peek(&self, _: &mut [u8]) -> io::Result<usize> {
        !
    }

    pub fn send(&self, _: &[u8]) -> io::Result<usize> {
        !
    }

    pub fn connect(&self, _: io::Result<&SocketAddr>) -> io::Result<()> {
        !
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        !
    }
}

pub struct LookupHost(!);

impl LookupHost {
    pub fn port(&self) -> u16 {
        !
    }
}

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        !
    }
}

impl TryFrom<&str> for LookupHost {
    type Error = io::Error;

    fn try_from(_v: &str) -> io::Result<LookupHost> {
        unsupported()
    }
}

impl<'a> TryFrom<(&'a str, u16)> for LookupHost {
    type Error = io::Error;

    fn try_from(_v: (&'a str, u16)) -> io::Result<LookupHost> {
        unsupported()
    }
}
