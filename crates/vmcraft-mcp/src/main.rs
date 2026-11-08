//! vmcraft-mcp: CLI-first binary + MCP 2024-11-05 stdio server.
//! stdout = pure JSON-RPC. Logs on stderr only.

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use vmcraft_core::{dispatch, Command};

#[derive(Parser, Debug)]
#[command(name = "vmcraft-mcp", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Top>,
}

#[derive(Subcommand, Debug)]
enum Top {
    Status,
    Vm {
        #[command(subcommand)]
        op: VmOp,
    },
    Snapshot {
        #[command(subcommand)]
        op: SnapOp,
    },
    Network {
        #[command(subcommand)]
        op: NetOp,
    },
    Disk {
        #[command(subcommand)]
        op: DiskOp,
    },
    Shot {
        #[arg(long)]
        vm: String,
        #[arg(long)]
        out: String,
    },
    Host {
        #[command(subcommand)]
        op: HostOp,
    },
    Sandbox {
        #[command(subcommand)]
        op: SandboxOp,
    },
    Tools,
    Mcp,
}

#[derive(Subcommand, Debug)]
enum VmOp {
    List {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        limit: Option<u32>,
    },
    Info {
        #[arg(long)]
        vm: String,
        #[arg(long)]
        json: bool,
    },
    Create {
        #[arg(long)]
        vm: String,
        #[arg(long, default_value = "Ubuntu_64")]
        ostype: String,
        #[arg(long, default_value_t = 4096)]
        memory_mb: u32,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    Start {
        #[arg(long)]
        vm: String,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    Stop {
        #[arg(long)]
        vm: String,
        #[arg(long, default_value_t = false)]
        force: bool,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    Delete {
        #[arg(long)]
        vm: String,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SnapOp {
    List {
        #[arg(long)]
        vm: String,
        #[arg(long)]
        json: bool,
    },
    Create {
        #[arg(long)]
        vm: String,
        #[arg(long)]
        snap: String,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    Restore {
        #[arg(long)]
        vm: String,
        #[arg(long)]
        snap: String,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    Delete {
        #[arg(long)]
        vm: String,
        #[arg(long)]
        snap: String,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
enum NetOp {
    List {
        #[arg(long)]
        json: bool,
    },
    Create {
        #[arg(long, default_value = "192.168.56.1")]
        ip: String,
        #[arg(long, default_value = "255.255.255.0")]
        netmask: String,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    Remove {
        #[arg(long)]
        net: String,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
enum DiskOp {
    List,
    Create {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t = 20)]
        size_gb: u32,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
enum HostOp {
    Info {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SandboxOp {
    Run {
        #[arg(long, default_value = "python")]
        lang: String,
        #[arg(long)]
        code: String,
        #[arg(long)]
        json: bool,
    },
}

fn print_json(v: &Value) {
    println!(
        "{}",
        serde_json::to_string(v).unwrap_or_else(|_| "{\"status\":\"error\"}".to_string())
    );
}

fn tools_list() -> Value {
    json!({"tools": [
        {"name": "vmcraft.status", "description": "Daemon status + hypervisor presence"},
        {"name": "vmcraft_vm_list", "description": "List VirtualBox VMs (read-only)"},
        {"name": "vmcraft_vm_info", "description": "VM info (read-only)"},
        {"name": "vmcraft_vm_create", "description": "Create+register VM (needs --yes / allow)"},
        {"name": "vmcraft_vm_start", "description": "Start VM headless (needs --yes / allow)"},
        {"name": "vmcraft_vm_stop", "description": "Stop VM poweroff|acpi (needs --yes / allow)"},
        {"name": "vmcraft_vm_delete", "description": "Unregister+delete VM (needs --yes / allow)"},
        {"name": "vmcraft_snapshot_list", "description": "Snapshot list (read-only)"},
        {"name": "vmcraft_snapshot_create", "description": "Take snapshot (needs --yes / allow)"},
        {"name": "vmcraft_snapshot_restore", "description": "Restore snapshot (needs --yes / allow)"},
        {"name": "vmcraft_snapshot_delete", "description": "Delete snapshot (needs --yes / allow)"},
        {"name": "vmcraft_network_list", "description": "Host-only networks (read-only)"},
        {"name": "vmcraft_network_create", "description": "Create hostonlyif + ipconfig (needs --yes / allow)"},
        {"name": "vmcraft_network_remove", "description": "Remove hostonlyif (needs --yes / allow)"},
        {"name": "vmcraft_disk_list", "description": "List virtual disks (read-only)"},
        {"name": "vmcraft_disk_create", "description": "Create .vdi (needs --yes / allow)"},
        {"name": "vmcraft_vm_shot", "description": "Screenshot running VM to PNG (read-only, VM must run)"},
        {"name": "vmcraft_host_info", "description": "Host info (read-only)"},
        {"name": "vmcraft_sandbox_run", "description": "Ephemeral code run, 30s timeout"}
    ]})
}

fn b(args: &Value, k: &str) -> bool {
    args.get(k).and_then(|v| v.as_bool()).unwrap_or(false)
}

async fn call_tool(name: &str, args: &Value) -> Value {
    let s = |k: &str| {
        args.get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let u = |k: &str| {
        args.get(k)
            .and_then(serde_json::Value::as_u64)
            .map(|v| v as u32)
    };
    match name {
        "vmcraft.status" | "vmcraft_status" => dispatch(Command::Status).await,
        "vmcraft_vm_list" | "vmcraft.vm_list" | "vm_management"
            if args
                .get("operation")
                .and_then(|v| v.as_str())
                .unwrap_or("list")
                == "list" =>
        {
            dispatch(Command::VmList { limit: u("limit") }).await
        }
        "vmcraft_vm_info" | "vmcraft.vm_info" => dispatch(Command::VmInfo { vm: s("vm") }).await,
        "vmcraft_vm_create" | "vmcraft.vm_create" => {
            dispatch(Command::VmCreate {
                vm: s("vm"),
                ostype: if s("ostype").is_empty() {
                    "Ubuntu_64".to_string()
                } else {
                    s("ostype")
                },
                memory_mb: u("memory_mb").unwrap_or(4096),
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_vm_start" | "vmcraft.vm_start" => {
            dispatch(Command::VmStart {
                vm: s("vm"),
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_vm_stop" | "vmcraft.vm_stop" => {
            dispatch(Command::VmStop {
                vm: s("vm"),
                force: b(args, "force"),
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_vm_delete" | "vmcraft.vm_delete" => {
            dispatch(Command::VmDelete {
                vm: s("vm"),
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_snapshot_list" | "vmcraft.snapshot_list" | "snapshot_management"
            if args.get("snap").is_none() =>
        {
            dispatch(Command::SnapshotList { vm: s("vm") }).await
        }
        "vmcraft_snapshot_create" | "vmcraft.snapshot_create" | "snapshot_management" => {
            dispatch(Command::SnapshotCreate {
                vm: s("vm"),
                snap: s("snap"),
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_snapshot_restore" | "vmcraft.snapshot_restore" => {
            dispatch(Command::SnapshotRestore {
                vm: s("vm"),
                snap: s("snap"),
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_snapshot_delete" | "vmcraft.snapshot_delete" => {
            dispatch(Command::SnapshotDelete {
                vm: s("vm"),
                snap: s("snap"),
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_network_list" | "vmcraft.network_list" | "network_management"
            if args.get("ip").is_none() && args.get("net").is_none() =>
        {
            dispatch(Command::NetworkList).await
        }
        "vmcraft_network_create" | "vmcraft.network_create" => {
            dispatch(Command::NetworkCreate {
                ip: s("ip"),
                netmask: if s("netmask").is_empty() {
                    "255.255.255.0".to_string()
                } else {
                    s("netmask")
                },
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_network_remove" | "vmcraft.network_remove" => {
            dispatch(Command::NetworkRemove {
                net: s("net"),
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_disk_list" | "vmcraft.disk_list" => dispatch(Command::DiskList).await,
        "vmcraft_disk_create" | "vmcraft.disk_create" => {
            dispatch(Command::DiskCreate {
                path: s("path"),
                size_gb: u("size_gb").unwrap_or(20),
                confirm: b(args, "confirm") || b(args, "yes"),
            })
            .await
        }
        "vmcraft_vm_shot" | "vmcraft.vm_shot" => {
            dispatch(Command::VmShot {
                vm: s("vm"),
                out: s("out"),
            })
            .await
        }
        "vmcraft_host_info" | "vmcraft.host_info" | "system_management" => {
            dispatch(Command::HostInfo).await
        }
        "vmcraft_sandbox_run" | "vmcraft.sandbox_run" | "sandbox_management" => {
            dispatch(Command::SandboxRun {
                lang: s("lang"),
                code: s("code"),
            })
            .await
        }
        _ => {
            json!({"content": [{"type": "text", "text": format!("unknown tool: {name}")}], "isError": true})
        }
    }
}

async fn run_mcp_stdio() -> Result<(), String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader
            .read_line(&mut line)
            .await
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let resp = json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": format!("parse error: {e}")}});
                let out = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
                stdout
                    .write_all(out.as_bytes())
                    .await
                    .map_err(|e| e.to_string())?;
                stdout.write_all(b"\n").await.map_err(|e| e.to_string())?;
                stdout.flush().await.map_err(|e| e.to_string())?;
                continue;
            }
        };
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let resp = match method.as_str() {
            "initialize" => {
                json!({"jsonrpc": "2.0", "id": id, "result": {"protocolVersion": "2024-11-05", "serverInfo": {"name": "vmcraft-mcp", "version": "0.2.0"}}})
            }
            "tools/list" => json!({"jsonrpc": "2.0", "id": id, "result": tools_list()}),
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or(json!({}));
                let tool = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                let result = call_tool(&tool, &args).await;
                json!({"jsonrpc": "2.0", "id": id, "result": {"content": [{"type": "text", "text": result.to_string()}]}})
            }
            _ => {
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": "method not found"}})
            }
        };
        let out = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
        stdout
            .write_all(out.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        stdout.write_all(b"\n").await.map_err(|e| e.to_string())?;
        stdout.flush().await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        None | Some(Top::Status) => print_json(&dispatch(Command::Status).await),
        Some(Top::Vm { op }) => match op {
            VmOp::List { limit, .. } => print_json(&dispatch(Command::VmList { limit }).await),
            VmOp::Info { vm, .. } => print_json(&dispatch(Command::VmInfo { vm }).await),
            VmOp::Create {
                vm,
                ostype,
                memory_mb,
                yes,
            } => print_json(
                &dispatch(Command::VmCreate {
                    vm,
                    ostype,
                    memory_mb,
                    confirm: yes,
                })
                .await,
            ),
            VmOp::Start { vm, yes } => {
                print_json(&dispatch(Command::VmStart { vm, confirm: yes }).await)
            }
            VmOp::Stop { vm, force, yes } => print_json(
                &dispatch(Command::VmStop {
                    vm,
                    force,
                    confirm: yes,
                })
                .await,
            ),
            VmOp::Delete { vm, yes } => {
                print_json(&dispatch(Command::VmDelete { vm, confirm: yes }).await)
            }
        },
        Some(Top::Snapshot { op }) => match op {
            SnapOp::List { vm, .. } => print_json(&dispatch(Command::SnapshotList { vm }).await),
            SnapOp::Create { vm, snap, yes } => print_json(
                &dispatch(Command::SnapshotCreate {
                    vm,
                    snap,
                    confirm: yes,
                })
                .await,
            ),
            SnapOp::Restore { vm, snap, yes } => print_json(
                &dispatch(Command::SnapshotRestore {
                    vm,
                    snap,
                    confirm: yes,
                })
                .await,
            ),
            SnapOp::Delete { vm, snap, yes } => print_json(
                &dispatch(Command::SnapshotDelete {
                    vm,
                    snap,
                    confirm: yes,
                })
                .await,
            ),
        },
        Some(Top::Network { op }) => match op {
            NetOp::List { .. } => print_json(&dispatch(Command::NetworkList).await),
            NetOp::Create { ip, netmask, yes } => print_json(
                &dispatch(Command::NetworkCreate {
                    ip,
                    netmask,
                    confirm: yes,
                })
                .await,
            ),
            NetOp::Remove { net, yes } => {
                print_json(&dispatch(Command::NetworkRemove { net, confirm: yes }).await)
            }
        },
        Some(Top::Disk { op }) => match op {
            DiskOp::List => print_json(&dispatch(Command::DiskList).await),
            DiskOp::Create { path, size_gb, yes } => print_json(
                &dispatch(Command::DiskCreate {
                    path,
                    size_gb,
                    confirm: yes,
                })
                .await,
            ),
        },
        Some(Top::Shot { vm, out }) => print_json(&dispatch(Command::VmShot { vm, out }).await),
        Some(Top::Host { op }) => match op {
            HostOp::Info { .. } => print_json(&dispatch(Command::HostInfo).await),
        },
        Some(Top::Sandbox { op }) => match op {
            SandboxOp::Run { lang, code, .. } => {
                print_json(&dispatch(Command::SandboxRun { lang, code }).await)
            }
        },
        Some(Top::Tools) => print_json(&tools_list()),
        Some(Top::Mcp) => {
            if let Err(e) = run_mcp_stdio().await {
                eprintln!("mcp stdio error: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn catalog_lists_tools() {
        let t = tools_list();
        let arr = t
            .get("tools")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        assert!(arr >= 19);
    }

    #[tokio::test]
    async fn unknown_tool_becomes_error_content() {
        let v = call_tool("nope", &json!({})).await;
        assert_eq!(v.get("isError"), Some(&json!(true)));
    }

    #[tokio::test]
    async fn legacy_portmanteau_alias_routes() {
        let v = call_tool("vm_management", &json!({"operation": "list"})).await;
        assert!(v.get("tool").is_some());
    }

    #[tokio::test]
    async fn mutate_without_confirm_is_refused() {
        let v = call_tool("vmcraft_vm_start", &json!({"vm": "some-vm"})).await;
        if !vmcraft_core::vm::mutate_allowed() {
            assert!(v
                .get("reason")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .contains("--yes"));
        }
    }
}
