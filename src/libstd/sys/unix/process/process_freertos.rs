use crate::io;
use crate::sys::process::process_common::*;
use crate::sys::{unsupported, unsupported_err, Void};

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

impl Command {
    pub fn spawn(&mut self, _default: Stdio, _needs_stdin: bool)
                 -> io::Result<(Process, StdioPipes)> {
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
