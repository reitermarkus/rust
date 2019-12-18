#![unstable(reason = "not public", issue = "0", feature = "net_fd")]

use crate::cmp;
use crate::io::{self, Read, Initializer, IoSlice, IoSliceMut};
use crate::mem;
use crate::sync::atomic::{AtomicBool, Ordering};
use crate::sys::{cvt, net::netc};
use crate::sys_common::AsInner;

use libc::{c_int, c_void, ssize_t};

#[derive(Debug)]
pub struct NetFileDesc {
    fd: c_int,
}

fn max_len() -> usize {
    // The maximum read limit on most posix-like systems is `SSIZE_MAX`,
    // with the man page quoting that if the count of bytes to read is
    // greater than `SSIZE_MAX` the result is "unspecified".
    //
    // On macOS, however, apparently the 64-bit libc is either buggy or
    // intentionally showing odd behavior by rejecting any read with a size
    // larger than or equal to INT_MAX. To handle both of these the read
    // size is capped on both platforms.
    if cfg!(target_os = "macos") {
        <c_int>::max_value() as usize - 1
    } else {
        <ssize_t>::max_value() as usize
    }
}

impl NetFileDesc {
    pub fn new(fd: c_int) -> NetFileDesc {
        NetFileDesc { fd }
    }

    pub fn raw(&self) -> c_int { self.fd }

    /// Extracts the actual file descriptor without closing it.
    pub fn into_raw(self) -> c_int {
        let fd = self.fd;
        mem::forget(self);
        fd
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            netc::read(self.fd,
                       buf.as_mut_ptr() as *mut c_void,
                       cmp::min(buf.len(), max_len()))
        })?;
        Ok(ret as usize)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            netc::readv(self.fd,
                        bufs.as_ptr() as *const libc::iovec,
                        cmp::min(bufs.len(), c_int::max_value() as usize) as c_int)
        })?;
        Ok(ret as usize)
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            netc::write(self.fd,
                        buf.as_ptr() as *const c_void,
                        cmp::min(buf.len(), max_len()))
        })?;
        Ok(ret as usize)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            netc::writev(self.fd,
                         bufs.as_ptr() as *const libc::iovec,
                         cmp::min(bufs.len(), c_int::max_value() as usize) as c_int)
        })?;
        Ok(ret as usize)
    }

    #[cfg(target_os = "linux")]
    pub fn get_cloexec(&self) -> io::Result<bool> {
        unsafe {
            Ok((cvt(libc::fcntl(self.fd, libc::F_GETFD))? & libc::FD_CLOEXEC) != 0)
        }
    }

    #[cfg(not(any(target_env = "newlib",
                  target_os = "solaris",
                  target_os = "emscripten",
                  target_os = "fuchsia",
                  target_os = "l4re",
                  target_os = "linux",
                  target_os = "haiku",
                  target_os = "redox")))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            cvt(libc::ioctl(self.fd, libc::FIOCLEX))?;
            Ok(())
        }
    }
    #[cfg(any(target_env = "newlib",
              target_os = "solaris",
              target_os = "emscripten",
              target_os = "fuchsia",
              target_os = "l4re",
              target_os = "linux",
              target_os = "haiku",
              target_os = "redox"))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            let previous = cvt(libc::fcntl(self.fd, libc::F_GETFD))?;
            let new = previous | libc::FD_CLOEXEC;
            if new != previous {
                cvt(libc::fcntl(self.fd, libc::F_SETFD, new))?;
            }
            Ok(())
        }
    }
  }

impl<'a> Read for &'a NetFileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl AsInner<c_int> for NetFileDesc {
    fn as_inner(&self) -> &c_int { &self.fd }
}

impl Drop for NetFileDesc {
    fn drop(&mut self) {
        // Note that errors are ignored when closing a file descriptor. The
        // reason for this is that if an error occurs we don't actually know if
        // the file descriptor was closed or not, and if we retried (for
        // something like EINTR), we might close another valid file descriptor
        // opened after we closed ours.
        let _ = unsafe { netc::close(self.fd) };
    }
}
