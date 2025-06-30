// SSH module - re-exports the real SSH implementation with cleaner names
pub use crate::ssh_real::{
    RealSshClient as SshClient,
    RealSshConfig as SshConfig,
    RealSshSession as SshSession,
    RealAuthMethod as AuthMethod,
    RealSshCommandResult as SshCommandResult,
    SystemInfo,
};