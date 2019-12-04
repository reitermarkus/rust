use crate::fmt;
use crate::io;
use crate::sys::process::process_common::*;
use crate::sys::{unsupported, unsupported_err, Void};

use libc::c_int;

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

impl Command {
    pub fn spawn(
        &mut self,
        _default: Stdio,
        _needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        unsupported()
    }

    pub fn exec(&mut self, _default: Stdio) -> io::Error {
        unsupported_err()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Processes
////////////////////////////////////////////////////////////////////////////////

/// The unique ID of the process (this should never be negative).
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitStatus(Void);

impl ExitStatus {
    pub fn success(&self) -> bool {
        match self.0 {}
    }

    pub fn code(&self) -> Option<i32> {
        match self.0 {}
    }

    pub fn signal(&self) -> Option<i32> {
        match self.0 {}
    }
}

impl From<c_int> for ExitStatus {
    fn from(_a: c_int) -> ExitStatus {
        unimplemented!()
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {}
    }
}
