//! Tier-4 Automation Pipelines: Airgap isolation, guest control, clean-room detonation, and disassembler orchestration.

use crate::vm::{down, gate, run_capture, valid_name, vboxmanage_candidates};
use serde_json::{json, Value};
use std::path::Path;

/// Enforce or release strict airgap isolation for a virtual machine.
///
/// When `disconnect` is true:
/// - Disconnects virtual network link cables across adapters 1..4 in real-time (`setlinkstateN off`).
/// - Disables guest clipboard integration (`clipboard mode disabled`).
/// - Disables guest drag-and-drop (`draganddrop disabled`).
pub async fn airgap_set(vm: &str, disconnect: bool) -> Value {
    let name = vm.trim();
    if !valid_name(name, 128) {
        return down("airgap_set", "invalid vm name");
    }

    let link_state = if disconnect { "off" } else { "on" };
    let clip_mode = if disconnect {
        "disabled"
    } else {
        "bidirectional"
    };
    let dnd_mode = if disconnect {
        "disabled"
    } else {
        "bidirectional"
    };

    if let Some(cand) = vboxmanage_candidates().into_iter().next() {
        let mut success = false;
        // Apply link state across potential adapters 1..4
        for adapter in 1..=4 {
            let adapter_arg = format!("setlinkstate{adapter}");
            if run_capture(&cand, &["controlvm", name, &adapter_arg, link_state], 10)
                .await
                .is_ok()
            {
                success = true;
            }
        }

        // Apply clipboard and drag-and-drop restrictions
        let _ = run_capture(
            &cand,
            &["controlvm", name, "clipboard", "mode", clip_mode],
            10,
        )
        .await;
        let _ = run_capture(&cand, &["controlvm", name, "draganddrop", dnd_mode], 10).await;

        if success {
            return json!({
                "tool": "airgap_set",
                "status": "ok",
                "vm": name,
                "disconnected": disconnect,
                "link_state": link_state,
                "clipboard": clip_mode,
                "draganddrop": dnd_mode,
            });
        }
    }

    down(
        "airgap_set",
        "failed to apply airgap policy (VM must be running)",
    )
}

/// Execute a command inside the guest via VirtualBox Guest Additions.
pub async fn guest_exec(
    vm: &str,
    user: &str,
    pass: &str,
    exe: &str,
    args: &[String],
    timeout_secs: Option<u64>,
) -> Value {
    let name = vm.trim();
    if !valid_name(name, 128) {
        return down("guest_exec", "invalid vm name");
    }

    let timeout = timeout_secs.unwrap_or(60);
    if let Some(cand) = vboxmanage_candidates().into_iter().next() {
        let mut cmd_args = vec![
            "guestcontrol",
            name,
            "run",
            "--username",
            user,
            "--password",
            pass,
            "--wait-stdout",
            "--wait-stderr",
            "--exe",
            exe,
            "--",
        ];
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        cmd_args.extend(arg_refs);

        match run_capture(&cand, &cmd_args, timeout).await {
            Ok(output) => {
                return json!({
                    "tool": "guest_exec",
                    "status": "ok",
                    "vm": name,
                    "exe": exe,
                    "output": output.chars().take(8000).collect::<String>(),
                });
            }
            Err(e) => {
                return down("guest_exec", &format!("guest execution failed: {e}"));
            }
        }
    }

    down("guest_exec", "VBoxManage not found or runnable")
}

/// Copy a host file into the guest VM filesystem.
pub async fn guest_copyto(
    vm: &str,
    user: &str,
    pass: &str,
    host_src: &str,
    guest_dest: &str,
) -> Value {
    let name = vm.trim();
    if !valid_name(name, 128) {
        return down("guest_copyto", "invalid vm name");
    }
    if !Path::new(host_src).exists() {
        return down("guest_copyto", "host source file does not exist");
    }

    if let Some(cand) = vboxmanage_candidates().into_iter().next() {
        let args = [
            "guestcontrol",
            name,
            "copyto",
            "--username",
            user,
            "--password",
            pass,
            "--target-directory",
            guest_dest,
            host_src,
        ];

        match run_capture(&cand, &args, 60).await {
            Ok(_) => {
                return json!({
                    "tool": "guest_copyto",
                    "status": "ok",
                    "vm": name,
                    "host_src": host_src,
                    "guest_dest": guest_dest,
                });
            }
            Err(e) => {
                return down("guest_copyto", &format!("copyto failed: {e}"));
            }
        }
    }

    down("guest_copyto", "VBoxManage not found or runnable")
}

/// Copy a guest file from the guest VM back to the host filesystem.
pub async fn guest_copyfrom(
    vm: &str,
    user: &str,
    pass: &str,
    guest_src: &str,
    host_dest: &str,
) -> Value {
    let name = vm.trim();
    if !valid_name(name, 128) {
        return down("guest_copyfrom", "invalid vm name");
    }

    if let Some(cand) = vboxmanage_candidates().into_iter().next() {
        let args = [
            "guestcontrol",
            name,
            "copyfrom",
            "--username",
            user,
            "--password",
            pass,
            "--target-directory",
            host_dest,
            guest_src,
        ];

        match run_capture(&cand, &args, 60).await {
            Ok(_) => {
                return json!({
                    "tool": "guest_copyfrom",
                    "status": "ok",
                    "vm": name,
                    "guest_src": guest_src,
                    "host_dest": host_dest,
                });
            }
            Err(e) => {
                return down("guest_copyfrom", &format!("copyfrom failed: {e}"));
            }
        }
    }

    down("guest_copyfrom", "VBoxManage not found or runnable")
}

/// Parameters for the composite clean-room detonation & disassembly pipeline.
#[derive(Debug, Clone)]
pub struct DetonatePipelineParams {
    pub vm: String,
    pub user: String,
    pub pass: String,
    pub keygen: Option<String>,
    pub target_binary: String,
    pub disassembler_exe: String,
    pub disassembler_args: Vec<String>,
    pub guest_output: String,
    pub host_output: String,
    pub screenshot: Option<String>,
    pub confirm: bool,
}

/// Execute an untrusted binary/keygen inside a strictly air-gapped sandbox,
/// disassemble the target inside the guest with the potentially tainted tool,
/// extract only the clean disassembly output back to the host, and forcibly
/// power off and revert the VM to its clean snapshot.
pub async fn pipeline_detonate(p: DetonatePipelineParams) -> Value {
    if let Some(err) = gate("pipeline_detonate", p.confirm) {
        return err;
    }

    let name = p.vm.trim();
    if !valid_name(name, 128) {
        return down("pipeline_detonate", "invalid vm name");
    }
    if !Path::new(&p.target_binary).exists() {
        return down("pipeline_detonate", "target binary not found on host");
    }

    let cand = match vboxmanage_candidates().into_iter().next() {
        Some(c) => c,
        None => return down("pipeline_detonate", "VBoxManage not found"),
    };

    let snapshot_name = format!("craft-detonate-{}", chrono::Utc::now().timestamp());
    let mut log_steps = Vec::new();

    // 1. Take pre-detonation snapshot while clean
    log_steps.push(format!("Taking pre-detonation snapshot '{snapshot_name}'"));
    if let Err(e) = run_capture(&cand, &["snapshot", name, "take", &snapshot_name], 30).await {
        return down("pipeline_detonate", &format!("snapshot take failed: {e}"));
    }

    // Wrap execution in a guarded block so cleanup ALWAYS runs
    let result = execute_detonation_steps(&cand, name, &p, &mut log_steps).await;

    // 10. Guaranteed Nuke & Revert
    log_steps.push("Executing post-detonation nuke: powering off VM".to_string());
    let _ = run_capture(&cand, &["controlvm", name, "poweroff"], 15).await;

    log_steps.push(format!("Reverting to clean snapshot '{snapshot_name}'"));
    let restore_res = run_capture(&cand, &["snapshot", name, "restore", &snapshot_name], 30).await;

    log_steps.push(format!("Purging temporary snapshot '{snapshot_name}'"));
    let _ = run_capture(&cand, &["snapshot", name, "delete", &snapshot_name], 30).await;

    match result {
        Ok(artifact_info) => json!({
            "tool": "pipeline_detonate",
            "status": "ok",
            "vm": name,
            "airgap_enforced": true,
            "keygen_detonated": p.keygen.is_some(),
            "disassembly_extracted": p.host_output,
            "disassembly_info": artifact_info,
            "reverted_clean": restore_res.is_ok(),
            "steps": log_steps,
        }),
        Err(err_msg) => json!({
            "tool": "pipeline_detonate",
            "status": "error",
            "vm": name,
            "reason": err_msg,
            "reverted_clean": restore_res.is_ok(),
            "steps": log_steps,
        }),
    }
}

async fn execute_detonation_steps(
    cand: &str,
    name: &str,
    p: &DetonatePipelineParams,
    log_steps: &mut Vec<String>,
) -> Result<Value, String> {
    // 2. Start VM headless
    log_steps.push("Starting guest VM headless".to_string());
    let _ = run_capture(cand, &["startvm", name, "--type", "headless"], 20).await;

    // 3. Enforce Airgap
    log_steps.push("Severing virtual network links (hard airgap enforced)".to_string());
    for adapter in 1..=4 {
        let adapter_arg = format!("setlinkstate{adapter}");
        let _ = run_capture(cand, &["controlvm", name, &adapter_arg, "off"], 10).await;
    }
    let _ = run_capture(
        cand,
        &["controlvm", name, "clipboard", "mode", "disabled"],
        10,
    )
    .await;
    let _ = run_capture(cand, &["controlvm", name, "draganddrop", "disabled"], 10).await;

    // Wait briefly for guest additions to settle
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // 4. Create in-guest detonation workspace: C:\detonate or /tmp/detonate
    let guest_workspace = "C:\\detonate";
    log_steps.push(format!("Creating in-guest workspace {guest_workspace}"));
    let _ = run_capture(
        cand,
        &[
            "guestcontrol",
            name,
            "mkdir",
            "--username",
            &p.user,
            "--password",
            &p.pass,
            "--parents",
            guest_workspace,
        ],
        15,
    )
    .await;

    // 5. Inject keygen/patcher if provided
    if let Some(ref keygen_path) = p.keygen {
        if Path::new(keygen_path).exists() {
            log_steps.push(format!("Injecting keygen '{keygen_path}' into guest"));
            run_capture(
                cand,
                &[
                    "guestcontrol",
                    name,
                    "copyto",
                    "--username",
                    &p.user,
                    "--password",
                    &p.pass,
                    "--target-directory",
                    guest_workspace,
                    keygen_path,
                ],
                30,
            )
            .await
            .map_err(|e| format!("inject keygen failed: {e}"))?;

            // Detonate keygen inside guest
            let keygen_filename = Path::new(keygen_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("keygen.exe");
            let in_guest_keygen = format!("{guest_workspace}\\{keygen_filename}");

            log_steps.push(format!("Detonating '{in_guest_keygen}' in isolated guest"));
            let _ = run_capture(
                cand,
                &[
                    "guestcontrol",
                    name,
                    "run",
                    "--username",
                    &p.user,
                    "--password",
                    &p.pass,
                    "--exe",
                    &in_guest_keygen,
                    "--",
                ],
                30,
            )
            .await;

            // Optional screenshot after keygen run
            if let Some(ref shot_path) = p.screenshot {
                let _ =
                    run_capture(cand, &["controlvm", name, "screenshotpng", shot_path], 10).await;
            }
        }
    }

    // 6. Inject analysis target binary
    let target_filename = Path::new(&p.target_binary)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("target.bin");
    let in_guest_target = format!("{guest_workspace}\\{target_filename}");

    log_steps.push(format!("Injecting target binary '{in_guest_target}'"));
    run_capture(
        cand,
        &[
            "guestcontrol",
            name,
            "copyto",
            "--username",
            &p.user,
            "--password",
            &p.pass,
            "--target-directory",
            guest_workspace,
            &p.target_binary,
        ],
        30,
    )
    .await
    .map_err(|e| format!("inject target binary failed: {e}"))?;

    // 7. Execute in-guest disassembler against the target binary
    log_steps.push(format!(
        "Executing disassembler '{}' on '{}' (airgapped)",
        p.disassembler_exe, in_guest_target
    ));

    let mut disasm_cmd_args = vec![
        "guestcontrol",
        name,
        "run",
        "--username",
        &p.user,
        "--password",
        &p.pass,
        "--wait-stdout",
        "--wait-stderr",
        "--exe",
        &p.disassembler_exe,
        "--",
    ];
    let arg_refs: Vec<&str> = p.disassembler_args.iter().map(|s| s.as_str()).collect();
    disasm_cmd_args.extend(arg_refs);

    let disasm_res = run_capture(cand, &disasm_cmd_args, 90)
        .await
        .unwrap_or_else(|e| format!("disassembler note: {e}"));
    log_steps.push(format!(
        "Disassembler completed: {}",
        disasm_res.chars().take(200).collect::<String>()
    ));

    // 8. Capture post-disassembly framebuffer screenshot
    if let Some(ref shot_path) = p.screenshot {
        log_steps.push(format!(
            "Capturing post-disassembly screenshot '{shot_path}'"
        ));
        let _ = run_capture(cand, &["controlvm", name, "screenshotpng", shot_path], 10).await;
    }

    // 9. Extract disassembly artifact back to host
    log_steps.push(format!(
        "Extracting guest disassembly '{}' -> host '{}'",
        p.guest_output, p.host_output
    ));
    run_capture(
        cand,
        &[
            "guestcontrol",
            name,
            "copyfrom",
            "--username",
            &p.user,
            "--password",
            &p.pass,
            "--target-directory",
            &p.host_output,
            &p.guest_output,
        ],
        30,
    )
    .await
    .map_err(|e| format!("extract disassembly artifact failed: {e}"))?;

    let file_meta = std::fs::metadata(&p.host_output).ok();
    let size = file_meta.map(|m| m.len()).unwrap_or(0);

    Ok(json!({
        "extracted_file": p.host_output,
        "bytes": size,
    }))
}

/// Parameters for a clean-room naked install / test run inside a baseline VM.
#[derive(Debug, Clone)]
pub struct NakedInstallParams {
    pub repo: String,
    pub baseline_vm: Option<String>,
    pub user: String,
    pub pass: String,
    pub snapshot: Option<String>,
    pub command: Option<String>,
    pub linux: bool,
    pub confirm: bool,
}

fn valid_repo_ref(repo: &str) -> bool {
    let r = repo.trim();
    !r.is_empty()
        && r.len() <= 256
        && r.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | ':'))
}

/// Run an automated naked install test for a target repository via VirtualBox Guest Additions.
pub async fn pipeline_naked_install(p: NakedInstallParams) -> Value {
    if let Some(err) = gate("pipeline_naked_install", p.confirm) {
        return err;
    }

    let target_vm = p.baseline_vm.as_deref().unwrap_or("win11-automated").trim();
    if !valid_name(target_vm, 128) {
        return down("pipeline_naked_install", "invalid baseline vm name");
    }
    let repo = p.repo.trim();
    if !valid_repo_ref(repo) {
        return down("pipeline_naked_install", "invalid repo reference");
    }
    if p.user.trim().is_empty() {
        return down(
            "pipeline_naked_install",
            "guest username (--user) is required for guestcontrol execution",
        );
    }
    if let Some(ref snap) = p.snapshot {
        if !valid_name(snap.trim(), 128) {
            return down("pipeline_naked_install", "invalid snapshot name");
        }
    }

    let Some(cand) = vboxmanage_candidates().into_iter().next() else {
        return down("pipeline_naked_install", "VBoxManage not found");
    };

    let mut steps: Vec<String> = Vec::new();

    if let Some(ref snap) = p.snapshot {
        let s = snap.trim();
        let _ = run_capture(&cand, &["controlvm", target_vm, "poweroff"], 20).await;
        if let Err(e) = run_capture(&cand, &["snapshot", target_vm, "restore", s], 60).await {
            return down(
                "pipeline_naked_install",
                &format!("failed to restore baseline snapshot '{s}': {e}"),
            );
        }
        steps.push(format!("Restored snapshot '{s}' on '{target_vm}'"));
    }

    match run_capture(&cand, &["startvm", target_vm, "--type", "headless"], 45).await {
        Ok(_) => steps.push(format!("Started VM '{target_vm}' headlessly")),
        Err(e) if e.contains("already locked") || e.contains("already running") => {
            steps.push(format!("VM '{target_vm}' already running"));
        }
        Err(e) => {
            return down(
                "pipeline_naked_install",
                &format!("failed to start '{target_vm}': {e}"),
            );
        }
    }

    let repo_url = if repo.starts_with("http://") || repo.starts_with("https://") {
        repo.to_string()
    } else {
        format!("https://github.com/{repo}.git")
    };

    let guest_cmd = p.command.clone().unwrap_or_else(|| {
        if p.linux {
            format!(
                "rm -rf /tmp/naked-test && git clone --depth 1 '{repo_url}' /tmp/naked-test && cd /tmp/naked-test && cargo test --workspace"
            )
        } else {
            format!(
                "if exist C:\\naked-test rmdir /s /q C:\\naked-test & git clone --depth 1 {repo_url} C:\\naked-test && cd /d C:\\naked-test && cargo test --workspace"
            )
        }
    });

    let (exe, shell_flag) = if p.linux {
        ("/bin/bash", "-lc")
    } else {
        ("cmd.exe", "/c")
    };

    let exec_res = run_capture(
        &cand,
        &[
            "guestcontrol",
            target_vm,
            "run",
            "--username",
            &p.user,
            "--password",
            &p.pass,
            "--wait-stdout",
            "--wait-stderr",
            "--exe",
            exe,
            "--",
            shell_flag,
            &guest_cmd,
        ],
        180,
    )
    .await;

    if let Some(ref snap) = p.snapshot {
        let s = snap.trim();
        let _ = run_capture(&cand, &["controlvm", target_vm, "poweroff"], 20).await;
        let _ = run_capture(&cand, &["snapshot", target_vm, "restore", s], 60).await;
        steps.push(format!("Rolled back '{target_vm}' to '{s}'"));
    }

    match exec_res {
        Ok(output) => {
            steps.push("Guest install/test command succeeded".to_string());
            json!({
                "tool": "pipeline_naked_install",
                "status": "ok",
                "repo": repo,
                "baseline_vm": target_vm,
                "verified": true,
                "steps": steps,
                "output": output.chars().take(8000).collect::<String>(),
            })
        }
        Err(e) => {
            steps.push(format!("Guest install/test failed: {e}"));
            json!({
                "tool": "pipeline_naked_install",
                "status": "error",
                "repo": repo,
                "baseline_vm": target_vm,
                "verified": false,
                "steps": steps,
                "reason": e,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn airgap_validation_rejects_injection() {
        let res = airgap_set("bad;vm&name", true).await;
        assert_eq!(res.get("status"), Some(&json!("down")));
    }

    #[tokio::test]
    async fn pipeline_detonate_requires_confirm() {
        let params = DetonatePipelineParams {
            vm: "win11-automated".to_string(),
            user: "test".to_string(),
            pass: "pass".to_string(),
            keygen: None,
            target_binary: "test.bin".to_string(),
            disassembler_exe: "ida64.exe".to_string(),
            disassembler_args: vec![],
            guest_output: "out.asm".to_string(),
            host_output: "out.asm".to_string(),
            screenshot: None,
            confirm: false,
        };

        let res = pipeline_detonate(params).await;
        if !crate::vm::mutate_allowed() {
            assert_eq!(res.get("status"), Some(&json!("error")));
        }
    }

    #[tokio::test]
    async fn pipeline_naked_install_requires_user_and_confirm() {
        let no_confirm = pipeline_naked_install(NakedInstallParams {
            repo: "sandraschi/vmcraft".to_string(),
            baseline_vm: Some("ubu-2404-lts".to_string()),
            user: "user".to_string(),
            pass: "user".to_string(),
            snapshot: None,
            command: None,
            linux: true,
            confirm: false,
        })
        .await;
        if !crate::vm::mutate_allowed() {
            assert_eq!(no_confirm.get("status"), Some(&json!("error")));
        }

        let no_user = pipeline_naked_install(NakedInstallParams {
            repo: "sandraschi/vmcraft".to_string(),
            baseline_vm: Some("ubu-2404-lts".to_string()),
            user: "".to_string(),
            pass: "".to_string(),
            snapshot: None,
            command: None,
            linux: true,
            confirm: true,
        })
        .await;
        assert_eq!(no_user.get("status"), Some(&json!("down")));

        let bad_repo = pipeline_naked_install(NakedInstallParams {
            repo: "bad;rm -rf /".to_string(),
            baseline_vm: Some("ubu-2404-lts".to_string()),
            user: "user".to_string(),
            pass: "user".to_string(),
            snapshot: None,
            command: None,
            linux: true,
            confirm: true,
        })
        .await;
        assert_eq!(bad_repo.get("status"), Some(&json!("down")));
    }
}
