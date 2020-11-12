pub use self::process_common::{Command, CommandArgs, ExitCode, Stdio, StdioPipes};
pub use self::process_inner::{ExitStatus, Process};
pub use crate::ffi::OsString as EnvKey;
pub use crate::sys_common::process::CommandEnvs;

mod process_common;
#[cfg(all(not(target_os = "fuchsia"), not(target_os = "none")))]
#[path = "process_unix.rs"]
mod process_inner;
#[cfg(target_os = "fuchsia")]
#[path = "process_fuchsia.rs"]
mod process_inner;
#[cfg(target_os = "none")]
#[path = "process_unsupported.rs"]
mod process_inner;
#[cfg(target_os = "fuchsia")]
mod zircon;
