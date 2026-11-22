//! VM/hypervisor protocol: VBoxManage subprocess + Hyper-V probe + sandbox exec.
//! Read-only by default. Mutating ops are NOT in Tier 1 (gated behind --yes in L2).

use crate::creds::with_timeout_secs;
use serde_json::{json, Value};
use std::time::Duration;

pub(crate) fn vboxmanage_candidates() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var("VBOXMANAGE_PATH") {
        let t = p.trim().to_string();
        if !t.is_empty() {
            out.push(t);
        }
    }
    let default_win = r"C:\Program Files\Oracle\VirtualBox\VBoxManage.exe";
    if std::path::Path::new(default_win).exists() {
        out.push(default_win.to_string());
    }
    out.push("VBoxManage".to_string());
    if !out.iter().any(|s| s == default_win) {
        out.push(default_win.to_string());
    }
    out
}

pub(crate) async fn run_capture(prog: &str, args: &[&str], secs: u64) -> Result<String, String> {
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

pub(crate) fn down(tool: &str, reason: &str) -> Value {
    json!({"tool": tool, "status": "down", "reason": reason})
}

pub(crate) fn gate(tool: &str, confirm: bool) -> Option<Value> {
    if confirm || mutate_allowed() {
        None
    } else {
        Some(
            json!({"tool": tool, "status": "error", "reason": "mutating op requires --yes or VMCRAFT_ALLOW_MUTATE=1"}),
        )
    }
}

pub(crate) fn valid_name(value: &str, max: usize) -> bool {
    let t = value.trim();
    if t.is_empty() || t.chars().count() > max {
        return false;
    }
    t.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || c == ' '
            || c == '_'
            || c == '-'
            || c == '.'
            || c == '('
            || c == ')'
    })
}

fn valid_token(value: &str, max: usize) -> bool {
    let t = value.trim();
    if t.is_empty() || t.chars().count() > max {
        return false;
    }
    t.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

fn valid_ipv4(value: &str) -> bool {
    let parts: Vec<&str> = value.trim().split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| {
        !p.is_empty()
            && p.len() <= 3
            && p.chars().all(|c| c.is_ascii_digit())
            && p.parse::<u8>().is_ok()
    })
}

async fn run_vbox(args: &[&str], secs: u64) -> Result<String, String> {
    let mut last_err = String::from("no VBoxManage candidate");
    for cand in vboxmanage_candidates() {
        match run_capture(&cand, args, secs).await {
            Ok(out) => return Ok(out),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

pub async fn status() -> Value {
    let vbox = vbox_probe().await;
    let hyperv = hyperv_probe().await;
    json!({
        "tool": "vmcraft.status",
        "status": "ok",
        "vbox_present": vbox,
        "hyperv_present": hyperv,
        "mutate_allowed": mutate_allowed(),
        "note": "Tier-3 live multi-hypervisor + craft command bus. Mutating verbs gated behind --yes / VMCRAFT_ALLOW_MUTATE=1."
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

pub async fn hyperv_probe() -> bool {
    let output = run_capture(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "if (Get-Command Get-VM -ErrorAction SilentlyContinue) { Write-Output 'HV_OK' }",
        ],
        5,
    )
    .await;
    match output {
        Ok(t) => t.trim().contains("HV_OK"),
        Err(_) => false,
    }
}

pub async fn hyperv_list() -> Value {
    let output = run_capture(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", "Get-VM -ErrorAction SilentlyContinue | Select-Object Name, State, Id | ConvertTo-Json -Compress"],
        10,
    ).await;
    match output {
        Ok(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return json!({"tool": "hyperv_list", "status": "ok", "count": 0, "vms": []});
            }
            let parsed: Value =
                serde_json::from_str(trimmed).unwrap_or_else(|_| Value::Array(Vec::new()));
            let list = match parsed {
                Value::Array(arr) => arr,
                Value::Object(obj) => vec![Value::Object(obj)],
                _ => Vec::new(),
            };
            let mut vms = Vec::new();
            for item in list {
                let name = item
                    .get("Name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let id = item
                    .get("Id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let state_code = item.get("State").and_then(|v| v.as_u64()).unwrap_or(0);
                let state = match state_code {
                    2 => "running",
                    3 => "off",
                    4 => "stopping",
                    6 => "saved",
                    9 => "paused",
                    _ => "unknown",
                };
                if !name.is_empty() {
                    vms.push(
                        json!({"name": name, "uuid": id, "state": state, "provider": "hyperv"}),
                    );
                }
            }
            json!({"tool": "hyperv_list", "status": "ok", "count": vms.len(), "vms": vms})
        }
        Err(e) => down("hyperv_list", &format!("Hyper-V query failed: {e}")),
    }
}

pub async fn hyperv_start(vm: &str, confirm: bool) -> Value {
    if let Some(g) = gate("hyperv_start", confirm) {
        return g;
    }
    if !valid_name(vm, 64) {
        return json!({"tool": "hyperv_start", "status": "error", "reason": "vm must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    let script = format!("Start-VM -Name '{}'", vm.replace('\'', "''"));
    match run_capture(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", &script],
        30,
    )
    .await
    {
        Ok(_) => json!({"tool": "hyperv_start", "status": "ok", "vm": vm, "action": "start"}),
        Err(e) => json!({"tool": "hyperv_start", "status": "error", "vm": vm, "reason": e}),
    }
}

pub async fn hyperv_stop(vm: &str, force: bool, confirm: bool) -> Value {
    if let Some(g) = gate("hyperv_stop", confirm) {
        return g;
    }
    if !valid_name(vm, 64) {
        return json!({"tool": "hyperv_stop", "status": "error", "reason": "vm must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    let flag = if force { " -TurnOff" } else { " -Save" };
    let script = format!("Stop-VM -Name '{}'{}", vm.replace('\'', "''"), flag);
    match run_capture(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", &script],
        30,
    )
    .await
    {
        Ok(_) => json!({"tool": "hyperv_stop", "status": "ok", "vm": vm, "action": "stop"}),
        Err(e) => json!({"tool": "hyperv_stop", "status": "error", "vm": vm, "reason": e}),
    }
}

pub async fn vm_list(limit: Option<u32>) -> Value {
    let mut all_vms: Vec<Value> = Vec::new();
    let mut vbox_found = false;

    for cand in vboxmanage_candidates() {
        if let Ok(text) = run_capture(&cand, &["list", "vms"], 15).await {
            vbox_found = true;
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
                all_vms.push(json!({"name": name, "uuid": id, "provider": "virtualbox"}));
                if let Some(lim) = limit {
                    if all_vms.len() >= lim as usize {
                        break;
                    }
                }
            }
            break;
        }
    }

    // Also include Hyper-V VMs if under limit
    if limit.map(|l| all_vms.len() < l as usize).unwrap_or(true) {
        let hv_res = hyperv_list().await;
        if let Some(arr) = hv_res.get("vms").and_then(|v| v.as_array()) {
            for v in arr {
                all_vms.push(v.clone());
                if let Some(lim) = limit {
                    if all_vms.len() >= lim as usize {
                        break;
                    }
                }
            }
        }
    }

    if vbox_found || !all_vms.is_empty() {
        return json!({"tool": "vm_list", "status": "ok", "count": all_vms.len(), "vms": all_vms});
    }

    down("vm_list", "No hypervisor found (VirtualBox or Hyper-V)")
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
                let mut snapshots = Vec::new();
                let mut current_name = String::new();
                let mut current_uuid = String::new();
                let mut current_desc = String::new();

                for line in text.lines() {
                    let line = line.trim();
                    if let Some(pos) = line.find('=') {
                        let key = line[..pos].trim();
                        let val = line[pos + 1..].trim().trim_matches('"');
                        if key.starts_with("SnapshotName") && !key.starts_with("CurrentSnapshot") {
                            if !current_name.is_empty() {
                                snapshots.push(json!({
                                    "name": current_name,
                                    "uuid": current_uuid,
                                    "description": current_desc,
                                }));
                                current_uuid.clear();
                                current_desc.clear();
                            }
                            current_name = val.to_string();
                        } else if key.starts_with("SnapshotUUID")
                            && !key.starts_with("CurrentSnapshot")
                        {
                            current_uuid = val.to_string();
                        } else if key.starts_with("SnapshotDescription") {
                            current_desc = val.to_string();
                        }
                    }
                }
                if !current_name.is_empty() {
                    snapshots.push(json!({
                        "name": current_name,
                        "uuid": current_uuid,
                        "description": current_desc,
                    }));
                }

                return json!({
                    "tool": "snapshot_list",
                    "status": "ok",
                    "vm": name,
                    "count": snapshots.len(),
                    "snapshots": snapshots,
                    "raw": text.chars().take(4000).collect::<String>()
                });
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
        match run_capture(&cand, &["list", "hostonlyifs"], 15).await {
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

pub async fn vm_create(vm: &str, ostype: &str, memory_mb: u32, confirm: bool) -> Value {
    if let Some(g) = gate("vm_create", confirm) {
        return g;
    }
    if !valid_name(vm, 64) {
        return json!({"tool": "vm_create", "status": "error", "reason": "vm must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    if !valid_token(ostype, 32) {
        return json!({"tool": "vm_create", "status": "error", "reason": "ostype must be 1..32 chars [A-Za-z0-9_.-] e.g. Ubuntu_64"});
    }
    if !(512..=65536).contains(&memory_mb) {
        return json!({"tool": "vm_create", "status": "error", "reason": "memory_mb must be 512..65536"});
    }
    let name = vm.trim();
    let mem = memory_mb.to_string();
    match run_vbox(
        &[
            "createvm",
            "--name",
            name,
            "--ostype",
            ostype.trim(),
            "--register",
        ],
        60,
    )
    .await
    {
        Ok(out) => match run_vbox(&["modifyvm", name, "--memory", &mem, "--cpus", "2"], 60).await {
            Ok(modify) => {
                json!({"tool": "vm_create", "status": "ok", "vm": name, "ostype": ostype.trim(), "memory_mb": memory_mb, "create": out.chars().take(1000).collect::<String>(), "modify": modify.chars().take(1000).collect::<String>()})
            }
            Err(e) => {
                json!({"tool": "vm_create", "status": "error", "reason": format!("modifyvm failed: {e}"), "vm": name})
            }
        },
        Err(e) => json!({"tool": "vm_create", "status": "error", "reason": e, "vm": name}),
    }
}

pub async fn vm_start(vm: &str, confirm: bool) -> Value {
    if let Some(g) = gate("vm_start", confirm) {
        return g;
    }
    if !valid_name(vm, 64) {
        return json!({"tool": "vm_start", "status": "error", "reason": "vm must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    match run_vbox(&["startvm", vm.trim(), "--type", "headless"], 60).await {
        Ok(out) => {
            json!({"tool": "vm_start", "status": "ok", "vm": vm.trim(), "output": out.chars().take(1000).collect::<String>()})
        }
        Err(e) => json!({"tool": "vm_start", "status": "error", "reason": e, "vm": vm.trim()}),
    }
}

pub async fn vm_stop(vm: &str, force: bool, confirm: bool) -> Value {
    if let Some(g) = gate("vm_stop", confirm) {
        return g;
    }
    if !valid_name(vm, 64) {
        return json!({"tool": "vm_stop", "status": "error", "reason": "vm must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    let action = if force { "poweroff" } else { "acpipowerbutton" };
    match run_vbox(&["controlvm", vm.trim(), action], 60).await {
        Ok(out) => {
            json!({"tool": "vm_stop", "status": "ok", "vm": vm.trim(), "action": action, "output": out.chars().take(1000).collect::<String>()})
        }
        Err(e) => json!({"tool": "vm_stop", "status": "error", "reason": e, "vm": vm.trim()}),
    }
}

pub async fn vm_delete(vm: &str, confirm: bool) -> Value {
    if let Some(g) = gate("vm_delete", confirm) {
        return g;
    }
    if !valid_name(vm, 64) {
        return json!({"tool": "vm_delete", "status": "error", "reason": "vm must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    match run_vbox(&["unregistervm", vm.trim(), "--delete"], 60).await {
        Ok(out) => {
            json!({"tool": "vm_delete", "status": "ok", "vm": vm.trim(), "output": out.chars().take(1000).collect::<String>()})
        }
        Err(e) => json!({"tool": "vm_delete", "status": "error", "reason": e, "vm": vm.trim()}),
    }
}

pub async fn snapshot_create(vm: &str, snap: &str, confirm: bool) -> Value {
    if let Some(g) = gate("snapshot_create", confirm) {
        return g;
    }
    if !valid_name(vm, 64) || !valid_name(snap, 64) {
        return json!({"tool": "snapshot_create", "status": "error", "reason": "vm and snap must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    match run_vbox(&["snapshot", vm.trim(), "take", snap.trim()], 60).await {
        Ok(out) => {
            json!({"tool": "snapshot_create", "status": "ok", "vm": vm.trim(), "snap": snap.trim(), "output": out.chars().take(1000).collect::<String>()})
        }
        Err(e) => json!({"tool": "snapshot_create", "status": "error", "reason": e}),
    }
}

pub async fn snapshot_restore(vm: &str, snap: &str, confirm: bool) -> Value {
    if let Some(g) = gate("snapshot_restore", confirm) {
        return g;
    }
    if !valid_name(vm, 64) || !valid_name(snap, 64) {
        return json!({"tool": "snapshot_restore", "status": "error", "reason": "vm and snap must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    match run_vbox(&["snapshot", vm.trim(), "restore", snap.trim()], 60).await {
        Ok(out) => {
            json!({"tool": "snapshot_restore", "status": "ok", "vm": vm.trim(), "snap": snap.trim(), "output": out.chars().take(1000).collect::<String>()})
        }
        Err(e) => json!({"tool": "snapshot_restore", "status": "error", "reason": e}),
    }
}

pub async fn snapshot_delete(vm: &str, snap: &str, confirm: bool) -> Value {
    if let Some(g) = gate("snapshot_delete", confirm) {
        return g;
    }
    if !valid_name(vm, 64) || !valid_name(snap, 64) {
        return json!({"tool": "snapshot_delete", "status": "error", "reason": "vm and snap must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    match run_vbox(&["snapshot", vm.trim(), "delete", snap.trim()], 60).await {
        Ok(out) => {
            json!({"tool": "snapshot_delete", "status": "ok", "vm": vm.trim(), "snap": snap.trim(), "output": out.chars().take(1000).collect::<String>()})
        }
        Err(e) => json!({"tool": "snapshot_delete", "status": "error", "reason": e}),
    }
}

pub async fn network_create(ip: &str, netmask: &str, confirm: bool) -> Value {
    if let Some(g) = gate("network_create", confirm) {
        return g;
    }
    if !valid_ipv4(ip) || !valid_ipv4(netmask) {
        return json!({"tool": "network_create", "status": "error", "reason": "ip and netmask must be IPv4 dotted-quad"});
    }
    let created = match run_vbox(&["hostonlyif", "create"], 60).await {
        Ok(out) => out,
        Err(e) => return json!({"tool": "network_create", "status": "error", "reason": e}),
    };
    let iface = created
        .lines()
        .find_map(|l| {
            let t = l.trim();
            if t.starts_with('\'') && t.contains("was successfully created") {
                Some(
                    t.trim_matches('\'')
                        .split("' was")
                        .next()
                        .unwrap_or("")
                        .to_string(),
                )
            } else if t.contains("VirtualBox Host-Only") {
                Some(t.trim_matches(|c| c == '\'' || c == '"').to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();
    if iface.is_empty() {
        return json!({"tool": "network_create", "status": "error", "reason": "created but could not parse interface name", "raw": created.chars().take(1000).collect::<String>()});
    }
    match run_vbox(
        &[
            "hostonlyif",
            "ipconfig",
            iface.trim(),
            "--ip",
            ip.trim(),
            "--netmask",
            netmask.trim(),
        ],
        60,
    )
    .await
    {
        Ok(out) => {
            json!({"tool": "network_create", "status": "ok", "iface": iface.trim(), "ip": ip.trim(), "netmask": netmask.trim(), "output": out.chars().take(1000).collect::<String>()})
        }
        Err(e) => {
            json!({"tool": "network_create", "status": "error", "reason": e, "iface": iface.trim()})
        }
    }
}

pub async fn network_remove(net: &str, confirm: bool) -> Value {
    if let Some(g) = gate("network_remove", confirm) {
        return g;
    }
    if !valid_name(net, 64) {
        return json!({"tool": "network_remove", "status": "error", "reason": "net must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    match run_vbox(&["hostonlyif", "remove", net.trim()], 60).await {
        Ok(out) => {
            json!({"tool": "network_remove", "status": "ok", "net": net.trim(), "output": out.chars().take(1000).collect::<String>()})
        }
        Err(e) => json!({"tool": "network_remove", "status": "error", "reason": e}),
    }
}

pub async fn disk_list() -> Value {
    match run_vbox(&["list", "hdds"], 15).await {
        Ok(out) => {
            json!({"tool": "disk_list", "status": "ok", "raw": out.chars().take(4000).collect::<String>()})
        }
        Err(e) => down("disk_list", &e),
    }
}

pub async fn disk_create(path: &str, size_gb: u32, confirm: bool) -> Value {
    if let Some(g) = gate("disk_create", confirm) {
        return g;
    }
    let p = path.trim();
    if p.is_empty() || p.chars().count() > 260 || !p.to_ascii_lowercase().ends_with(".vdi") {
        return json!({"tool": "disk_create", "status": "error", "reason": "path must be 1..260 chars ending in .vdi"});
    }
    if !(1..=2000).contains(&size_gb) {
        return json!({"tool": "disk_create", "status": "error", "reason": "size_gb must be 1..2000"});
    }
    if let Some(parent) = std::path::Path::new(p).parent() {
        if !parent.as_os_str().is_empty() && !parent.is_dir() {
            return json!({"tool": "disk_create", "status": "error", "reason": "parent directory does not exist"});
        }
    }
    let size_mb = (size_gb as u64) * 1024;
    let size_str = size_mb.to_string();
    match run_vbox(
        &["createmedium", "disk", "--filename", p, "--size", &size_str],
        120,
    )
    .await
    {
        Ok(out) => {
            json!({"tool": "disk_create", "status": "ok", "path": p, "size_gb": size_gb, "output": out.chars().take(1000).collect::<String>()})
        }
        Err(e) => json!({"tool": "disk_create", "status": "error", "reason": e}),
    }
}

pub async fn vm_shot(vm: &str, out: &str) -> Value {
    if !valid_name(vm, 64) {
        return json!({"tool": "vm_shot", "status": "error", "reason": "vm must be 1..64 chars [A-Za-z0-9 _-.()]"});
    }
    let o = out.trim();
    if o.is_empty() || o.chars().count() > 260 || !o.to_ascii_lowercase().ends_with(".png") {
        return json!({"tool": "vm_shot", "status": "error", "reason": "out must be 1..260 chars ending in .png"});
    }
    if let Some(parent) = std::path::Path::new(o).parent() {
        if !parent.as_os_str().is_empty() && std::fs::create_dir_all(parent).is_err() {
            return json!({"tool": "vm_shot", "status": "error", "reason": "cannot create output dir"});
        }
    }
    match run_vbox(&["controlvm", vm.trim(), "screenshotpng", o], 30).await {
        Ok(raw) => match std::fs::read(o) {
            Ok(bytes) => {
                if bytes.len() < 24 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
                    return json!({"tool": "vm_shot", "status": "error", "reason": "output is not a valid PNG (VM running?)"});
                }
                let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
                let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
                json!({"tool": "vm_shot", "status": "ok", "vm": vm.trim(), "path": o, "bytes": bytes.len(), "width": w, "height": h, "output": raw.chars().take(500).collect::<String>()})
            }
            Err(e) => {
                json!({"tool": "vm_shot", "status": "error", "reason": format!("screenshot taken but cannot read file: {e}")})
            }
        },
        Err(e) => {
            json!({"tool": "vm_shot", "status": "error", "reason": e, "hint": "VM must be running for screenshotpng"})
        }
    }
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
            vec!["-NoProfile", "-NonInteractive"],
            "run.ps1",
        )
    };
    let fpath = dir.join(fname);
    if std::fs::write(&fpath, code).is_err() {
        return json!({"tool": "sandbox_run", "status": "error", "reason": "cannot write temp file"});
    }
    let fstr = fpath.to_string_lossy().to_string();
    let fut = async {
        let out = if l == "python" {
            let mut last_err = String::from("no python interpreter found");
            let mut found = None;
            for cand in ["python", "py", "python3"] {
                let mut cmd = tokio::process::Command::new(cand);
                cmd.arg(&fstr);
                cmd.kill_on_drop(true);
                match cmd.output().await {
                    Ok(o) => {
                        found = Some(o);
                        break;
                    }
                    Err(e) => {
                        last_err = format!("spawn {cand}: {e}");
                    }
                }
            }
            match found {
                Some(o) => o,
                None => return Err(last_err),
            }
        } else {
            let mut cmd = tokio::process::Command::new(prog);
            for a in args {
                cmd.arg(a);
            }
            cmd.arg("-File");
            cmd.arg(&fstr);
            cmd.kill_on_drop(true);
            cmd.output()
                .await
                .map_err(|e| format!("spawn {prog}: {e}"))?
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_name_rejects_injection() {
        assert!(!valid_name("a; rm -rf /", 64));
        assert!(!valid_name("a&b", 64));
        assert!(!valid_name("", 64));
        assert!(valid_name("win11-automated", 64));
        assert!(valid_name("ubu 2404 (test)", 64));
    }

    #[test]
    fn ipv4_validation() {
        assert!(valid_ipv4("192.168.56.1"));
        assert!(!valid_ipv4("999.1.1.1"));
        assert!(!valid_ipv4("abc"));
    }

    #[tokio::test]
    async fn mutate_gate_refuses_without_confirm() {
        let v = vm_start("some-vm", false).await;
        let reason = v.get("reason").and_then(|r| r.as_str()).unwrap_or("");
        if !mutate_allowed() {
            assert!(reason.contains("--yes"));
        } else {
            assert!(v.get("tool").is_some());
        }
    }

    #[tokio::test]
    async fn snapshot_gate_refuses_without_confirm() {
        let v = snapshot_create("vm", "snap", false).await;
        if !mutate_allowed() {
            assert!(v
                .get("reason")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .contains("--yes"));
        }
    }
}
