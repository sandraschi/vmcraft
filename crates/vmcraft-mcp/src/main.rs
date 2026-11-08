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
}

#[derive(Subcommand, Debug)]
enum SnapOp {
    List {
        #[arg(long)]
        vm: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum NetOp {
    List {
        #[arg(long)]
        json: bool,
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
        {"name": "vmcraft_snapshot_list", "description": "Snapshot list (read-only)"},
        {"name": "vmcraft_network_list", "description": "Host-only networks (read-only)"},
        {"name": "vmcraft_host_info", "description": "Host info (read-only)"},
        {"name": "vmcraft_sandbox_run", "description": "Ephemeral code run, 30s timeout"}
    ]})
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
        "vmcraft_snapshot_list" | "vmcraft.snapshot_list" | "snapshot_management" => {
            dispatch(Command::SnapshotList { vm: s("vm") }).await
        }
        "vmcraft_network_list" | "vmcraft.network_list" | "network_management" => {
            dispatch(Command::NetworkList).await
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
                json!({"jsonrpc": "2.0", "id": id, "result": {"protocolVersion": "2024-11-05", "serverInfo": {"name": "vmcraft-mcp", "version": "0.1.0"}}})
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
        },
        Some(Top::Snapshot { op }) => match op {
            SnapOp::List { vm, .. } => print_json(&dispatch(Command::SnapshotList { vm }).await),
        },
        Some(Top::Network { op }) => match op {
            NetOp::List { .. } => print_json(&dispatch(Command::NetworkList).await),
        },
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
        assert!(arr >= 7);
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
}
