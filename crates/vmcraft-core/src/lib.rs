//! L1 VM workstation core: VBoxManage + Hyper-V + sandbox over one Command bus.
//! No panics, no unwraps: every failure is a `down`/`error` JSON value.

pub mod bridge;
pub mod creds;
pub mod error;
pub mod import;
pub mod llm;
pub mod vm;

use serde_json::{json, Value};

/// Canonical command IDs (dotted ArtCraft bus + underscore MCP alias both route here).
pub const CMD_STATUS: &str = "vmcraft.status";
pub const CMD_VM_LIST: &str = "vmcraft.vm_list";
pub const CMD_VM_INFO: &str = "vmcraft.vm_info";
pub const CMD_SNAPSHOT_LIST: &str = "vmcraft.snapshot_list";
pub const CMD_NETWORK_LIST: &str = "vmcraft.network_list";
pub const CMD_HOST_INFO: &str = "vmcraft.host_info";
pub const CMD_SANDBOX_RUN: &str = "vmcraft.sandbox_run";

#[derive(Debug, Clone)]
pub enum Command {
    Status,
    VmList { limit: Option<u32> },
    VmInfo { vm: String },
    SnapshotList { vm: String },
    NetworkList,
    HostInfo,
    SandboxRun { lang: String, code: String },
    BridgeEvent { title: String, summary: String },
}

pub async fn dispatch(cmd: Command) -> Value {
    match cmd {
        Command::Status => vm::status().await,
        Command::VmList { limit } => vm::vm_list(limit).await,
        Command::VmInfo { vm } => vm::vm_info(&vm).await,
        Command::SnapshotList { vm } => vm::snapshot_list(&vm).await,
        Command::NetworkList => vm::network_list().await,
        Command::HostInfo => vm::host_info().await,
        Command::SandboxRun { lang, code } => vm::sandbox_run(&lang, &code).await,
        Command::BridgeEvent { title, summary } => {
            json!({"tool": "bridge.event", "status": "ok", "title": title, "summary": summary})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_future_is_send() {
        fn assert_send<T: Send>(_: T) {}
        assert_send(dispatch(Command::Status));
    }
}
