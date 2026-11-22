//! L1 VM workstation core: VBoxManage + Hyper-V + sandbox over one Command bus.
//! No panics, no unwraps: every failure is a `down`/`error` JSON value.

pub mod bridge;
pub mod creds;
pub mod error;
pub mod firecracker;
pub mod import;
pub mod llm;
pub mod pipeline;
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
pub const CMD_HYPERV_LIST: &str = "vmcraft.hyperv_list";
pub const CMD_HYPERV_START: &str = "vmcraft.hyperv_start";
pub const CMD_HYPERV_STOP: &str = "vmcraft.hyperv_stop";
pub const CMD_AIRGAP_SET: &str = "vmcraft.airgap_set";
pub const CMD_GUEST_EXEC: &str = "vmcraft.guest_exec";
pub const CMD_GUEST_COPYTO: &str = "vmcraft.guest_copyto";
pub const CMD_GUEST_COPYFROM: &str = "vmcraft.guest_copyfrom";
pub const CMD_PIPELINE_DETONATE: &str = "vmcraft.pipeline_detonate";
pub const CMD_PIPELINE_NAKED_INSTALL: &str = "vmcraft.pipeline_naked_install";
pub const CMD_FIRECRACKER_PROBE: &str = "vmcraft.firecracker_probe";
pub const CMD_FIRECRACKER_LIST: &str = "vmcraft.firecracker_list";
pub const CMD_FIRECRACKER_SPAWN: &str = "vmcraft.firecracker_spawn";
pub const CMD_FIRECRACKER_KILL: &str = "vmcraft.firecracker_kill";

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
    HypervList,
    HypervStart {
        vm: String,
        confirm: bool,
    },
    HypervStop {
        vm: String,
        force: bool,
        confirm: bool,
    },
    AirgapSet {
        vm: String,
        disconnect: bool,
    },
    GuestExec {
        vm: String,
        user: String,
        pass: String,
        exe: String,
        args: Vec<String>,
        timeout_secs: Option<u64>,
    },
    GuestCopyTo {
        vm: String,
        user: String,
        pass: String,
        host_src: String,
        guest_dest: String,
    },
    GuestCopyFrom {
        vm: String,
        user: String,
        pass: String,
        guest_src: String,
        host_dest: String,
    },
    PipelineDetonate {
        vm: String,
        user: String,
        pass: String,
        keygen: Option<String>,
        target_binary: String,
        disassembler_exe: String,
        disassembler_args: Vec<String>,
        guest_output: String,
        host_output: String,
        screenshot: Option<String>,
        confirm: bool,
    },
    PipelineNakedInstall {
        repo: String,
        baseline_vm: Option<String>,
        user: String,
        pass: String,
        snapshot: Option<String>,
        command: Option<String>,
        linux: bool,
        confirm: bool,
    },
    FirecrackerProbe,
    FirecrackerList,
    FirecrackerSpawn {
        vm_id: String,
        vcpu_count: u32,
        mem_size_mib: u32,
        kernel_path: String,
        rootfs_path: String,
        boot_args: Option<String>,
        confirm: bool,
    },
    FirecrackerKill {
        vm_id: String,
        confirm: bool,
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
        Command::HypervList => vm::hyperv_list().await,
        Command::HypervStart { vm, confirm } => vm::hyperv_start(&vm, confirm).await,
        Command::HypervStop { vm, force, confirm } => vm::hyperv_stop(&vm, force, confirm).await,
        Command::AirgapSet { vm, disconnect } => pipeline::airgap_set(&vm, disconnect).await,
        Command::GuestExec {
            vm,
            user,
            pass,
            exe,
            args,
            timeout_secs,
        } => pipeline::guest_exec(&vm, &user, &pass, &exe, &args, timeout_secs).await,
        Command::GuestCopyTo {
            vm,
            user,
            pass,
            host_src,
            guest_dest,
        } => pipeline::guest_copyto(&vm, &user, &pass, &host_src, &guest_dest).await,
        Command::GuestCopyFrom {
            vm,
            user,
            pass,
            guest_src,
            host_dest,
        } => pipeline::guest_copyfrom(&vm, &user, &pass, &guest_src, &host_dest).await,
        Command::PipelineDetonate {
            vm,
            user,
            pass,
            keygen,
            target_binary,
            disassembler_exe,
            disassembler_args,
            guest_output,
            host_output,
            screenshot,
            confirm,
        } => {
            pipeline::pipeline_detonate(pipeline::DetonatePipelineParams {
                vm,
                user,
                pass,
                keygen,
                target_binary,
                disassembler_exe,
                disassembler_args,
                guest_output,
                host_output,
                screenshot,
                confirm,
            })
            .await
        }
        Command::PipelineNakedInstall {
            repo,
            baseline_vm,
            user,
            pass,
            snapshot,
            command,
            linux,
            confirm,
        } => {
            pipeline::pipeline_naked_install(pipeline::NakedInstallParams {
                repo,
                baseline_vm,
                user,
                pass,
                snapshot,
                command,
                linux,
                confirm,
            })
            .await
        }
        Command::FirecrackerProbe => firecracker::firecracker_probe().await,
        Command::FirecrackerList => firecracker::firecracker_list().await,
        Command::FirecrackerSpawn {
            vm_id,
            vcpu_count,
            mem_size_mib,
            kernel_path,
            rootfs_path,
            boot_args,
            confirm,
        } => {
            firecracker::firecracker_spawn(firecracker::FirecrackerSpawnParams {
                vm_id,
                vcpu_count,
                mem_size_mib,
                kernel_path,
                rootfs_path,
                boot_args,
                confirm,
            })
            .await
        }
        Command::FirecrackerKill { vm_id, confirm } => {
            firecracker::firecracker_kill(&vm_id, confirm).await
        }
        Command::BridgeEvent { title, summary } => {
            json!({"tool": "bridge.event", "status": "ok", "title": title, "summary": summary})
        }
    }
}

pub fn commands_catalog() -> Value {
    json!([
        {
            "id": "status",
            "label": "App Status",
            "description": "Show vmcraft hypervisor presence, daemon state, and mutation policy",
            "params": {}
        },
        {
            "id": "vm.list",
            "label": "List Virtual Machines",
            "description": "List registered VirtualBox and Hyper-V virtual machines",
            "params": {
                "limit": "number (optional)"
            }
        },
        {
            "id": "vm.info",
            "label": "Virtual Machine Info",
            "description": "Retrieve comprehensive machine-readable hardware and state metadata for a VM",
            "params": {
                "vm": "string (name or uuid)"
            }
        },
        {
            "id": "vm.create",
            "label": "Create Virtual Machine",
            "description": "Provision and register a new VM with guest OS type and RAM allocation",
            "params": {
                "vm": "string",
                "ostype": "string (default: Ubuntu_64)",
                "memory_mb": "number (default: 4096)",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "vm.start",
            "label": "Start Virtual Machine",
            "description": "Boot an existing virtual machine headless",
            "params": {
                "vm": "string",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "vm.stop",
            "label": "Stop Virtual Machine",
            "description": "Power off or gracefully ACPI-shutdown a running VM",
            "params": {
                "vm": "string",
                "force": "boolean (default: false)",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "vm.delete",
            "label": "Delete Virtual Machine",
            "description": "Unregister and permanently delete a virtual machine and its settings",
            "params": {
                "vm": "string",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "snapshot.list",
            "label": "List Snapshots",
            "description": "List point-in-time snapshots and tree metadata for a virtual machine",
            "params": {
                "vm": "string"
            }
        },
        {
            "id": "snapshot.create",
            "label": "Take Snapshot",
            "description": "Create a live or offline named snapshot for a virtual machine",
            "params": {
                "vm": "string",
                "snap": "string",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "snapshot.restore",
            "label": "Restore Snapshot",
            "description": "Revert virtual machine state back to a named snapshot",
            "params": {
                "vm": "string",
                "snap": "string",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "snapshot.delete",
            "label": "Delete Snapshot",
            "description": "Purge and merge a snapshot from the snapshot tree",
            "params": {
                "vm": "string",
                "snap": "string",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "network.list",
            "label": "List Host-Only Networks",
            "description": "List VirtualBox host-only networking interfaces and DHCP leases",
            "params": {}
        },
        {
            "id": "network.create",
            "label": "Create Host-Only Network",
            "description": "Create and configure a host-only interface with IP address and netmask",
            "params": {
                "ip": "string (default: 192.168.56.1)",
                "netmask": "string (default: 255.255.255.0)",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "network.remove",
            "label": "Remove Host-Only Network",
            "description": "Remove a host-only network interface",
            "params": {
                "net": "string",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "disk.list",
            "label": "List Virtual Disks",
            "description": "List registered VDI and VMDK hard disk images",
            "params": {}
        },
        {
            "id": "disk.create",
            "label": "Create Virtual Disk",
            "description": "Create a dynamically allocated .vdi virtual hard drive image",
            "params": {
                "path": "string",
                "size_gb": "number (default: 20)",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "vm.shot",
            "label": "Capture Framebuffer Screenshot",
            "description": "Capture a real-time pixel framebuffer screenshot of a running VM to PNG",
            "params": {
                "vm": "string",
                "out": "string (path to output PNG)"
            }
        },
        {
            "id": "host.info",
            "label": "Host & Hypervisor Info",
            "description": "Query host capabilities, VirtualBox version, and supported guest OS templates",
            "params": {}
        },
        {
            "id": "sandbox.run",
            "label": "Run Ephemeral Sandbox",
            "description": "Execute isolated Python or PowerShell script with strict 30s timeout",
            "params": {
                "lang": "string (python|pwsh)",
                "code": "string"
            }
        },
        {
            "id": "hyperv.list",
            "label": "List Hyper-V Virtual Machines",
            "description": "Query native Windows Hyper-V virtual machines and current states",
            "params": {}
        },
        {
            "id": "hyperv.start",
            "label": "Start Hyper-V Virtual Machine",
            "description": "Boot a Hyper-V virtual machine",
            "params": {
                "vm": "string",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "hyperv.stop",
            "label": "Stop Hyper-V Virtual Machine",
            "description": "Stop a running Hyper-V virtual machine",
            "params": {
                "vm": "string",
                "force": "boolean (default: false)",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "airgap.set",
            "label": "Set Airgap Isolation",
            "description": "Enforce or release physical network disconnection, clipboard disable, and drag-and-drop block",
            "params": {
                "vm": "string",
                "disconnect": "boolean (default: true)"
            }
        },
        {
            "id": "guest.exec",
            "label": "Execute Inside Guest",
            "description": "Run a program inside the guest VM via Guest Additions",
            "params": {
                "vm": "string",
                "user": "string",
                "pass": "string",
                "exe": "string",
                "args": "array of strings",
                "timeout_secs": "number (default: 60)"
            }
        },
        {
            "id": "guest.copyto",
            "label": "Copy File to Guest",
            "description": "Copy a file from host system into guest virtual machine",
            "params": {
                "vm": "string",
                "user": "string",
                "pass": "string",
                "host_src": "string",
                "guest_dest": "string"
            }
        },
        {
            "id": "guest.copyfrom",
            "label": "Copy File from Guest",
            "description": "Extract a file from guest virtual machine back to host system",
            "params": {
                "vm": "string",
                "user": "string",
                "pass": "string",
                "guest_src": "string",
                "host_dest": "string"
            }
        },
        {
            "id": "pipeline.detonate",
            "label": "Detonate & Disassemble in Sandbox",
            "description": "Clean-room execution of keygen/untrusted binary under airgap, in-guest disassembler execution, clean artifact extraction, and auto-rollback",
            "params": {
                "vm": "string",
                "user": "string",
                "pass": "string",
                "keygen": "string (optional)",
                "target_binary": "string",
                "disassembler_exe": "string (default: ida64.exe)",
                "disassembler_args": "array of strings",
                "guest_output": "string",
                "host_output": "string",
                "screenshot": "string (optional)",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "pipeline.naked_install",
            "label": "Naked Install Test",
            "description": "Automated clean-room consumer install test for repo on pristine Windows baseline",
            "params": {
                "repo": "string",
                "baseline_vm": "string (optional)",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "firecracker.probe",
            "label": "Firecracker KVM Probe",
            "description": "Detect WSL2 /dev/kvm availability and Firecracker binary installation",
            "params": {}
        },
        {
            "id": "firecracker.list",
            "label": "List Firecracker Micro-VMs",
            "description": "List active Firecracker micro-VM processes and sockets in WSL2",
            "params": {}
        },
        {
            "id": "firecracker.spawn",
            "label": "Spawn Firecracker Micro-VM",
            "description": "Launch a sub-10ms ephemeral micro-VM with dedicated kernel and rootfs",
            "params": {
                "vm_id": "string",
                "vcpu": "number (default: 1)",
                "mem_size_mib": "number (default: 256)",
                "kernel_path": "string",
                "rootfs_path": "string",
                "boot_args": "string (optional)",
                "confirm": "boolean (default: false)"
            }
        },
        {
            "id": "firecracker.kill",
            "label": "Kill Firecracker Micro-VM",
            "description": "Terminate a running Firecracker micro-VM instance by VM ID",
            "params": {
                "vm_id": "string",
                "confirm": "boolean (default: false)"
            }
        }
    ])
}

pub fn describe_command(cmd_id: &str) -> Option<Value> {
    let clean = cmd_id.trim().trim_start_matches("vmcraft.");
    let catalog = commands_catalog();
    let arr = catalog.as_array()?;
    arr.iter()
        .find(|item| {
            item.get("id")
                .and_then(|v| v.as_str())
                .map(|id| id == clean || id == cmd_id.trim())
                .unwrap_or(false)
        })
        .cloned()
}

pub fn parse_command(id: &str, params: &Value) -> Result<Command, String> {
    let clean_id = id.trim().trim_start_matches("vmcraft.").replace('_', ".");
    match clean_id.as_str() {
        "status" => Ok(Command::Status),
        "vm.list" | "vm" => {
            let limit = params
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|u| u as u32);
            Ok(Command::VmList { limit })
        }
        "vm.info" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(Command::VmInfo { vm })
        }
        "vm.create" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let ostype = params
                .get("ostype")
                .and_then(|v| v.as_str())
                .unwrap_or("Ubuntu_64")
                .to_string();
            let memory_mb = params
                .get("memory_mb")
                .and_then(|v| v.as_u64())
                .unwrap_or(4096) as u32;
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::VmCreate {
                vm,
                ostype,
                memory_mb,
                confirm,
            })
        }
        "vm.start" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::VmStart { vm, confirm })
        }
        "vm.stop" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let force = params
                .get("force")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::VmStop { vm, force, confirm })
        }
        "vm.delete" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::VmDelete { vm, confirm })
        }
        "snapshot.list" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(Command::SnapshotList { vm })
        }
        "snapshot.create" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let snap = params
                .get("snap")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::SnapshotCreate { vm, snap, confirm })
        }
        "snapshot.restore" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let snap = params
                .get("snap")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::SnapshotRestore { vm, snap, confirm })
        }
        "snapshot.delete" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let snap = params
                .get("snap")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::SnapshotDelete { vm, snap, confirm })
        }
        "network.list" => Ok(Command::NetworkList),
        "network.create" => {
            let ip = params
                .get("ip")
                .and_then(|v| v.as_str())
                .unwrap_or("192.168.56.1")
                .to_string();
            let netmask = params
                .get("netmask")
                .and_then(|v| v.as_str())
                .unwrap_or("255.255.255.0")
                .to_string();
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::NetworkCreate {
                ip,
                netmask,
                confirm,
            })
        }
        "network.remove" => {
            let net = params
                .get("net")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::NetworkRemove { net, confirm })
        }
        "disk.list" => Ok(Command::DiskList),
        "disk.create" => {
            let path = params
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let size_gb = params.get("size_gb").and_then(|v| v.as_u64()).unwrap_or(20) as u32;
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::DiskCreate {
                path,
                size_gb,
                confirm,
            })
        }
        "vm.shot" | "shot" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let out = params
                .get("out")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(Command::VmShot { vm, out })
        }
        "host.info" => Ok(Command::HostInfo),
        "sandbox.run" => {
            let lang = params
                .get("lang")
                .and_then(|v| v.as_str())
                .unwrap_or("python")
                .to_string();
            let code = params
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(Command::SandboxRun { lang, code })
        }
        "hyperv.list" => Ok(Command::HypervList),
        "hyperv.start" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::HypervStart { vm, confirm })
        }
        "hyperv.stop" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let force = params
                .get("force")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::HypervStop { vm, force, confirm })
        }
        "airgap.set" | "airgap" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let disconnect = params
                .get("disconnect")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            Ok(Command::AirgapSet { vm, disconnect })
        }
        "guest.exec" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let user = params
                .get("user")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let pass = params
                .get("pass")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let exe = params
                .get("exe")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let args = params
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let timeout_secs = params.get("timeout_secs").and_then(|v| v.as_u64());
            Ok(Command::GuestExec {
                vm,
                user,
                pass,
                exe,
                args,
                timeout_secs,
            })
        }
        "guest.copyto" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let user = params
                .get("user")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let pass = params
                .get("pass")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let host_src = params
                .get("host_src")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let guest_dest = params
                .get("guest_dest")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(Command::GuestCopyTo {
                vm,
                user,
                pass,
                host_src,
                guest_dest,
            })
        }
        "guest.copyfrom" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let user = params
                .get("user")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let pass = params
                .get("pass")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let guest_src = params
                .get("guest_src")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let host_dest = params
                .get("host_dest")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(Command::GuestCopyFrom {
                vm,
                user,
                pass,
                guest_src,
                host_dest,
            })
        }
        "pipeline.detonate" | "detonate" => {
            let vm = params
                .get("vm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let user = params
                .get("user")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let pass = params
                .get("pass")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let keygen = params
                .get("keygen")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let target_binary = params
                .get("target_binary")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let disassembler_exe = params
                .get("disassembler_exe")
                .and_then(|v| v.as_str())
                .unwrap_or("ida64.exe")
                .to_string();
            let disassembler_args = params
                .get("disassembler_args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let guest_output = params
                .get("guest_output")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let host_output = params
                .get("host_output")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let screenshot = params
                .get("screenshot")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::PipelineDetonate {
                vm,
                user,
                pass,
                keygen,
                target_binary,
                disassembler_exe,
                disassembler_args,
                guest_output,
                host_output,
                screenshot,
                confirm,
            })
        }
        "pipeline.naked.install" | "naked.install" | "naked.test" => {
            let repo = params
                .get("repo")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let baseline_vm = params
                .get("baseline_vm")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let user = params
                .get("user")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let pass = params
                .get("pass")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let snapshot = params
                .get("snapshot")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let command = params
                .get("command")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let linux = params
                .get("linux")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::PipelineNakedInstall {
                repo,
                baseline_vm,
                user,
                pass,
                snapshot,
                command,
                linux,
                confirm,
            })
        }
        "firecracker.probe" => Ok(Command::FirecrackerProbe),
        "firecracker.list" => Ok(Command::FirecrackerList),
        "firecracker.spawn" => {
            let vm_id = params
                .get("vm_id")
                .and_then(|v| v.as_str())
                .unwrap_or("microvm-1")
                .to_string();
            let vcpu_count = params
                .get("vcpu")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32)
                .unwrap_or(1);
            let mem_size_mib = params
                .get("mem_size_mib")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32)
                .unwrap_or(256);
            let kernel_path = params
                .get("kernel_path")
                .and_then(|v| v.as_str())
                .unwrap_or("/vmlinux")
                .to_string();
            let rootfs_path = params
                .get("rootfs_path")
                .and_then(|v| v.as_str())
                .unwrap_or("/rootfs.ext4")
                .to_string();
            let boot_args = params
                .get("boot_args")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::FirecrackerSpawn {
                vm_id,
                vcpu_count,
                mem_size_mib,
                kernel_path,
                rootfs_path,
                boot_args,
                confirm,
            })
        }
        "firecracker.kill" => {
            let vm_id = params
                .get("vm_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let confirm = params
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Command::FirecrackerKill { vm_id, confirm })
        }
        _ => Err(format!("unknown command id '{id}'")),
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

    #[test]
    fn commands_catalog_has_unique_ids() {
        let cat = commands_catalog();
        let arr = cat.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
        assert!(arr.len() >= 30);
        let mut seen = std::collections::HashSet::new();
        for item in arr {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            assert!(!id.is_empty());
            assert!(seen.insert(id.to_string()), "duplicate command id: {id}");
        }
    }

    #[test]
    fn describe_command_resolves_every_catalog_entry() {
        let cat = commands_catalog();
        let arr = cat.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
        for item in arr {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let desc = describe_command(id);
            assert!(desc.is_some(), "describe_command returned None for '{id}'");
        }
    }

    #[test]
    fn parse_command_rejects_unknown_verb() {
        let res = parse_command("nope.unknown", &json!({}));
        assert!(res.is_err());
    }

    #[test]
    fn parse_command_roundtrips_vm_and_snapshot_verbs() {
        assert!(matches!(
            parse_command("vm.list", &json!({"limit": 10})),
            Ok(Command::VmList { limit: Some(10) })
        ));
        assert!(matches!(
            parse_command("vm.info", &json!({"vm": "ubu-2404-lts"})),
            Ok(Command::VmInfo { .. })
        ));
        assert!(matches!(
            parse_command(
                "snapshot.create",
                &json!({"vm": "u", "snap": "s", "confirm": true})
            ),
            Ok(Command::SnapshotCreate { confirm: true, .. })
        ));
        assert!(matches!(
            parse_command(
                "snapshot.restore",
                &json!({"vm": "u", "snap": "s", "confirm": true})
            ),
            Ok(Command::SnapshotRestore { confirm: true, .. })
        ));
    }

    #[test]
    fn parse_command_roundtrips_airgap_and_guest_verbs() {
        assert!(matches!(
            parse_command("airgap.set", &json!({"vm": "u", "disconnect": true})),
            Ok(Command::AirgapSet {
                disconnect: true,
                ..
            })
        ));
        assert!(matches!(
            parse_command("guest.exec", &json!({"vm": "u", "exe": "/bin/true"})),
            Ok(Command::GuestExec { .. })
        ));
    }

    #[test]
    fn parse_command_roundtrips_pipeline_naked_install() {
        let parsed = parse_command(
            "pipeline.naked_install",
            &json!({
                "repo": "sandraschi/vmcraft",
                "baseline_vm": "ubu-2404-lts",
                "user": "user",
                "pass": "user",
                "snapshot": "clean-desktop-2026-11-22",
                "linux": true,
                "confirm": true
            }),
        );
        assert!(matches!(
            parsed,
            Ok(Command::PipelineNakedInstall {
                linux: true,
                confirm: true,
                ..
            })
        ));
    }

    #[test]
    fn parse_command_roundtrips_firecracker_verbs() {
        assert!(matches!(
            parse_command("firecracker.probe", &json!({})),
            Ok(Command::FirecrackerProbe)
        ));
        assert!(matches!(
            parse_command("firecracker.list", &json!({})),
            Ok(Command::FirecrackerList)
        ));
        assert!(matches!(
            parse_command(
                "firecracker.spawn",
                &json!({"vm_id": "m1", "confirm": true})
            ),
            Ok(Command::FirecrackerSpawn { confirm: true, .. })
        ));
        assert!(matches!(
            parse_command("firecracker.kill", &json!({"vm_id": "m1", "confirm": true})),
            Ok(Command::FirecrackerKill { confirm: true, .. })
        ));
    }

    #[test]
    fn parse_command_roundtrips_network_and_disk_verbs() {
        assert!(matches!(
            parse_command("network.list", &json!({})),
            Ok(Command::NetworkList)
        ));
        assert!(matches!(
            parse_command("disk.list", &json!({})),
            Ok(Command::DiskList)
        ));
    }

    #[test]
    fn parse_command_roundtrips_sandbox_and_hyperv_verbs() {
        assert!(matches!(
            parse_command(
                "sandbox.run",
                &json!({"lang": "python", "code": "print(1)"})
            ),
            Ok(Command::SandboxRun { .. })
        ));
        assert!(matches!(
            parse_command("hyperv.list", &json!({})),
            Ok(Command::HypervList)
        ));
    }

    #[tokio::test]
    async fn dispatch_invalid_vm_returns_down_without_panic() {
        let res = dispatch(Command::VmInfo {
            vm: "bad;vm&name".to_string(),
        })
        .await;
        assert_eq!(res.get("status"), Some(&json!("down")));
    }
}
