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
pub const CMD_VM_CREATE: &str = "vmcraft.vm_create";
pub const CMD_VM_START: &str = "vmcraft.vm_start";
pub const CMD_VM_STOP: &str = "vmcraft.vm_stop";
pub const CMD_VM_DELETE: &str = "vmcraft.vm_delete";
pub const CMD_SNAPSHOT_LIST: &str = "vmcraft.snapshot_list";
pub const CMD_SNAPSHOT_CREATE: &str = "vmcraft.snapshot_create";
pub const CMD_SNAPSHOT_RESTORE: &str = "vmcraft.snapshot_restore";
pub const CMD_SNAPSHOT_DELETE: &str = "vmcraft.snapshot_delete";
pub const CMD_NETWORK_LIST: &str = "vmcraft.network_list";
pub const CMD_NETWORK_CREATE: &str = "vmcraft.network_create";
pub const CMD_NETWORK_REMOVE: &str = "vmcraft.network_remove";
pub const CMD_DISK_LIST: &str = "vmcraft.disk_list";
pub const CMD_DISK_CREATE: &str = "vmcraft.disk_create";
pub const CMD_VM_SHOT: &str = "vmcraft.vm_shot";
pub const CMD_HOST_INFO: &str = "vmcraft.host_info";
pub const CMD_SANDBOX_RUN: &str = "vmcraft.sandbox_run";

#[derive(Debug, Clone)]
pub enum Command {
    Status,
    VmList {
        limit: Option<u32>,
    },
    VmInfo {
        vm: String,
    },
    VmCreate {
        vm: String,
        ostype: String,
        memory_mb: u32,
        confirm: bool,
    },
    VmStart {
        vm: String,
        confirm: bool,
    },
    VmStop {
        vm: String,
        force: bool,
        confirm: bool,
    },
    VmDelete {
        vm: String,
        confirm: bool,
    },
    SnapshotList {
        vm: String,
    },
    SnapshotCreate {
        vm: String,
        snap: String,
        confirm: bool,
    },
    SnapshotRestore {
        vm: String,
        snap: String,
        confirm: bool,
    },
    SnapshotDelete {
        vm: String,
        snap: String,
        confirm: bool,
    },
    NetworkList,
    NetworkCreate {
        ip: String,
        netmask: String,
        confirm: bool,
    },
    NetworkRemove {
        net: String,
        confirm: bool,
    },
    DiskList,
    DiskCreate {
        path: String,
        size_gb: u32,
        confirm: bool,
    },
    VmShot {
        vm: String,
        out: String,
    },
    HostInfo,
    SandboxRun {
        lang: String,
        code: String,
    },
    BridgeEvent {
        title: String,
        summary: String,
    },
}

pub async fn dispatch(cmd: Command) -> Value {
    match cmd {
        Command::Status => vm::status().await,
        Command::VmList { limit } => vm::vm_list(limit).await,
        Command::VmInfo { vm } => vm::vm_info(&vm).await,
        Command::VmCreate {
            vm,
            ostype,
            memory_mb,
            confirm,
        } => vm::vm_create(&vm, &ostype, memory_mb, confirm).await,
        Command::VmStart { vm, confirm } => vm::vm_start(&vm, confirm).await,
        Command::VmStop { vm, force, confirm } => vm::vm_stop(&vm, force, confirm).await,
        Command::VmDelete { vm, confirm } => vm::vm_delete(&vm, confirm).await,
        Command::SnapshotList { vm } => vm::snapshot_list(&vm).await,
        Command::SnapshotCreate { vm, snap, confirm } => {
            vm::snapshot_create(&vm, &snap, confirm).await
        }
        Command::SnapshotRestore { vm, snap, confirm } => {
            vm::snapshot_restore(&vm, &snap, confirm).await
        }
        Command::SnapshotDelete { vm, snap, confirm } => {
            vm::snapshot_delete(&vm, &snap, confirm).await
        }
        Command::NetworkList => vm::network_list().await,
        Command::NetworkCreate {
            ip,
            netmask,
            confirm,
        } => vm::network_create(&ip, &netmask, confirm).await,
        Command::NetworkRemove { net, confirm } => vm::network_remove(&net, confirm).await,
        Command::DiskList => vm::disk_list().await,
        Command::DiskCreate {
            path,
            size_gb,
            confirm,
        } => vm::disk_create(&path, size_gb, confirm).await,
        Command::VmShot { vm, out } => vm::vm_shot(&vm, &out).await,
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
