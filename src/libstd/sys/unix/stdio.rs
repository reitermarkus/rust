use crate::io::{self, IoSlice, IoSliceMut};
use crate::mem::ManuallyDrop;
use crate::sys::fd::FileDesc;

#[cfg(target_arch = "xtensa")]
mod xtensa {
    use super::*;

    extern "C" {
        fn ets_write_char_uart(c: libc::c_char);
    }

    pub fn write(buf: &[u8]) -> io::Result<usize> {
        for &b in buf.iter() {
            unsafe { ets_write_char_uart(b as libc::c_char) }
        }

        Ok(buf.len())
    }
}

pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

impl Stdin {
    pub fn new() -> io::Result<Stdin> {
        Ok(Stdin(()))
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDIN_FILENO)).read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDIN_FILENO)).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        true
    }
}

impl Stdout {
    pub fn new() -> io::Result<Stdout> {
        Ok(Stdout(()))
    }
}

impl io::Write for Stdout {
    #[cfg(target_arch = "xtensa")]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        xtensa::write(buf)
    }

    #[cfg(not(target_arch = "xtensa"))]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDOUT_FILENO)).write(buf)
    }

    #[cfg(not(target_arch = "xtensa"))]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDOUT_FILENO)).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub fn new() -> io::Result<Stderr> {
        Ok(Stderr(()))
    }
}

impl io::Write for Stderr {
    #[cfg(target_arch = "xtensa")]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        xtensa::write(buf)
    }

    #[cfg(not(target_arch = "xtensa"))]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDERR_FILENO)).write(buf)
    }

    #[cfg(not(target_arch = "xtensa"))]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDERR_FILENO)).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn is_ebadf(err: &io::Error) -> bool {
    err.raw_os_error() == Some(libc::EBADF as i32)
}

pub const STDIN_BUF_SIZE: usize = crate::sys_common::io::DEFAULT_BUF_SIZE;

pub fn panic_output() -> Option<impl io::Write> {
    Stderr::new().ok()
}
