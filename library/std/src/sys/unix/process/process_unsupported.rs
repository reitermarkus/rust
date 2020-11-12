use crate::ffi::OsStr;
use crate::fmt;
use crate::io;
use crate::marker::PhantomData;
use crate::path::Path;
use crate::sys::fs::File;
use crate::sys::pipe::AnonPipe;
use crate::sys_common::process::{CommandEnv, CommandEnvs};
use crate::ffi::{CStr, CString, OsString};

pub use crate::ffi::OsString as EnvKey;

use crate::sys::process::process_common::*;

use libc::{c_char, c_int, gid_t, uid_t, EXIT_FAILURE, EXIT_SUCCESS};

#[path = "../../unsupported/common.rs"]
#[deny(unsafe_op_in_unsafe_fn)]
mod unsupported;

use unsupported::*;

impl Command {
    pub fn spawn(
        &mut self,
        _default: Stdio,
        _needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        unsupported()
    }
    
    pub fn exec(&mut self, default: Stdio) -> io::Error {
        unsupported_err()
    }
}

pub struct Process(Void);

impl Process {
    pub fn id(&self) -> u32 {
        match self.0 {}
    }

    pub fn kill(&mut self) -> io::Result<()> {
        match self.0 {}
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        match self.0 {}
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        match self.0 {}
    }
}

pub struct ExitStatus(Void);

impl ExitStatus {
    pub fn success(&self) -> bool {
        match self.0 {}
    }

    pub fn code(&self) -> Option<i32> {
        match self.0 {}
    }

    fn exited(&self) -> bool {
        match self.0 {}
    }

    pub fn signal(&self) -> Option<i32> {
        match self.0 {}
    }
}

impl Clone for ExitStatus {
    fn clone(&self) -> ExitStatus {
        match self.0 {}
    }
}

/// Converts a raw `c_int` to a type-safe `ExitStatus` by wrapping it without copying.
impl From<c_int> for ExitStatus {
    fn from(a: c_int) -> ExitStatus {
        panic!("Unsupported")
    }
}

impl Copy for ExitStatus {}

impl PartialEq for ExitStatus {
    fn eq(&self, _other: &ExitStatus) -> bool {
        match self.0 {}
    }
}

impl Eq for ExitStatus {}

impl fmt::Debug for ExitStatus {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {}
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {}
    }
}
