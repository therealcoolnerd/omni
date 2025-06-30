// SSH module - re-exports the real SSH implementation with cleaner names
pub use crate::ssh_real::{
    RealAuthMethod as AuthMethod, RealSshClient as SshClient,
    RealSshCommandResult as SshCommandResult, RealSshConfig as SshConfig,
    RealSshSession as SshSession, SystemInfo,
};
