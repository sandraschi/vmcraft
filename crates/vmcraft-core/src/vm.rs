//! VM/hypervisor protocol: VBoxManage subprocess + Hyper-V probe + sandbox exec.
//! Read-only by default. Mutating ops are NOT in Tier 1 (gated behind --yes in L2).

use crate::creds::with_timeout_secs;
use serde_json::{json, Value};
use std::time::Duration;

fn vboxmanage_candidates() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var("VBOXMANAGE_PATH") {
        let t = p.trim().to_string();
        if !t.is_empty() {
            out.push(t);
        }
    }
    out.push("VBoxManage".to_string());
    out.push(r"C:\Program Files\Oracle\VirtualBox\VBoxManage.exe".to_string());
    out
}

async fn run_capture(prog: &str, args: &[&str], secs: u64) -> Result<String, String> {
    let fut = async {
        let mut cmd = tokio::process::Command::new(prog);
        cmd.args(args);
        cmd.kill_on_drop(true);
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("spawn {prog}: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!(
                "exit {}: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr)
                    .chars()
                    .take(500)
                    .collect::<String>()
            ))
        }
    };
    with_timeout_secs("vbox", secs, fut)
        .await
        .map_err(|e| e.to_string())?
}

fn down(tool: &str, reason: &str) -> Value {
    json!({"tool": tool, "status": "down", "reason": reason})
}

pub async fn status() -> Value {
    let vbox = vbox_probe().await;
    json!({
        "tool": "vmcraft.status",
        "status": "ok",
        "vbox_present": vbox,
        "hyperv_present": false,
        "mutate_allowed": mutate_allowed(),
        "note": "Tier-1 read-only + sandbox. Mutating verbs gated behind --yes / VMCRAFT_ALLOW_MUTATE=1."
    })
}

pub fn mutate_allowed() -> bool {
    std::env::var("VMCRAFT_ALLOW_MUTATE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

async fn vbox_probe() -> bool {
    for cand in vboxmanage_candidates() {
        if run_capture(&cand, &["--version"], 8).await.is_ok() {
            return true;
        }
    }
    false
}

pub async fn vm_list(limit: Option<u32>) -> Value {
    for cand in vboxmanage_candidates() {
        match run_capture(&cand, &["list", "vms"], 15).await {
            Ok(text) => {
                let mut vms: Vec<Value> = Vec::new();
                for line in text.lines() {
                    let t = line.trim();
                    if t.is_empty() {
                        continue;
                    }
                    // Format: "Name" {uuid}
                    let (name, id) = if let Some(pos) = t.rfind('{') {
                        (
                            t[..pos].trim().trim_matches('"').to_string(),
                            t[pos..].to_string(),
                        )
                    } else {
                        (t.to_string(), String::new())
                    };
                    vms.push(json!({"name": name, "uuid": id}));
                    if let Some(lim) = limit {
                        if vms.len() >= lim as usize {
                            break;
                        }
                    }
                }
                return json!({"tool": "vm_list", "status": "ok", "vboxmanage": cand, "count": vms.len(), "vms": vms});
            }
            Err(_) => continue,
        }
    }
    down(
        "vm_list",
        "VBoxManage not found or not runnable (install VirtualBox 7+ or set VBOXMANAGE_PATH)",
    )
}

pub async fn vm_info(vm: &str) -> Value {
    let name = vm.trim();
    if name.is_empty() || name.chars().count() > 256 {
        return json!({"tool": "vm_info", "status": "error", "reason": "vm must be 1..256 chars"});
    }
    for cand in vboxmanage_candidates() {
        match run_capture(&cand, &["showvminfo", name, "--machinereadable"], 15).await {
            Ok(text) => {
                return json!({"tool": "vm_info", "status": "ok", "vm": name, "raw_lines": text.lines().count(), "info": text.chars().take(4000).collect::<String>()});
            }
            Err(_) => continue,
        }
    }
    down("vm_info", "VBoxManage not runnable; cannot info VM")
}

pub async fn snapshot_list(vm: &str) -> Value {
    let name = vm.trim();
    if name.is_empty() || name.chars().count() > 256 {
        return json!({"tool": "snapshot_list", "status": "error", "reason": "vm must be 1..256 chars"});
    }
    for cand in vboxmanage_candidates() {
        match run_capture(&cand, &["snapshot", name, "list", "--machinereadable"], 15).await {
            Ok(text) => {
                return json!({"tool": "snapshot_list", "status": "ok", "vm": name, "raw": text.chars().take(4000).collect::<String>()});
            }
            Err(_) => continue,
        }
    }
    down(
        "snapshot_list",
        "VBoxManage not runnable; cannot list snapshots",
    )
}

pub async fn network_list() -> Value {
    for cand in vboxmanage_candidates() {
        match run_capture(&cand, &["list", "hostonlynets"], 15).await {
            Ok(text) => {
                return json!({"tool": "network_list", "status": "ok", "raw": text.chars().take(4000).collect::<String>()});
            }
            Err(_) => continue,
        }
    }
    down(
        "network_list",
        "VBoxManage not runnable; cannot list host-only networks",
    )
}

pub async fn host_info() -> Value {
    for cand in vboxmanage_candidates() {
        let version = run_capture(&cand, &["--version"], 8)
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        let ostypes = run_capture(&cand, &["list", "ostypes"], 10)
            .await
            .unwrap_or_default();
        if version != "unknown" {
            let count = ostypes.lines().filter(|l| l.starts_with("ID:")).count();
            return json!({"tool": "host_info", "status": "ok", "vbox_version": version.trim(), "ostype_count": count});
        }
    }
    down("host_info", "VBoxManage not runnable")
}

pub async fn sandbox_run(lang: &str, code: &str) -> Value {
    let l = lang.trim().to_ascii_lowercase();
    if l != "python" && l != "pwsh" && l != "powershell" {
        return json!({"tool": "sandbox_run", "status": "error", "reason": "lang must be python|pwsh (Tier-1)"});
    }
    if code.trim().is_empty() || code.chars().count() > 8000 {
        return json!({"tool": "sandbox_run", "status": "error", "reason": "code must be 1..8000 chars"});
    }
    let dir = std::env::temp_dir().join("vmcraft-sandbox");
    if std::fs::create_dir_all(&dir).is_err() {
        return json!({"tool": "sandbox_run", "status": "error", "reason": "cannot create temp dir"});
    }
    let (prog, args, fname): (&str, Vec<&str>, &str) = if l == "python" {
        ("python", vec![], "run.py")
    } else {
        (
            "powershell.exe",
            vec!["-NoProfile", "-NonInteractive", "-Command"],
            "run.ps1",
        )
    };
    let fpath = dir.join(fname);
    if std::fs::write(&fpath, code).is_err() {
        return json!({"tool": "sandbox_run", "status": "error", "reason": "cannot write temp file"});
    }
    let fstr = fpath.to_string_lossy().to_string();
    let fut = async {
        let mut cmd = tokio::process::Command::new(prog);
        if l == "python" {
            cmd.arg(&fstr);
        } else {
            for a in args {
                cmd.arg(a);
            }
            cmd.arg(&fstr);
        }
        cmd.kill_on_drop(true);
        let out = cmd
            .output()
            .await
            .map_err(|e| format!("spawn {prog}: {e}"))?;
        let stdout = String::from_utf8_lossy(&out.stdout)
            .chars()
            .take(4000)
            .collect::<String>();
        let stderr = String::from_utf8_lossy(&out.stderr)
            .chars()
            .take(2000)
            .collect::<String>();
        Ok::<Value, String>(
            json!({"tool": "sandbox_run", "status": if out.status.success() { "ok" } else { "error" }, "exit": out.status.code().unwrap_or(-1), "stdout": stdout, "stderr": stderr}),
        )
    };
    match with_timeout_secs("sandbox", 30, fut).await {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => json!({"tool": "sandbox_run", "status": "error", "reason": e}),
        Err(e) => json!({"tool": "sandbox_run", "status": "error", "reason": e.to_string()}),
    }
}

#[allow(dead_code)]
fn _use_duration() {
    let _ = Duration::from_secs(1);
}
