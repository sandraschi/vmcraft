# vmcraft Help & User Guide

Pure-Rust ArtCraft Virtual Machine workstation (`vmcraft-core` L1, `vmcraft-mcp` L2 CLI + MCP stdio, `vmcraft-app` L3 egui desktop).

## 1. Get Started

1. **Launch**: Run `.\start.ps1` (or `cargo run -p vmcraft-app`) to open the desktop workstation. Status auto-polls VirtualBox and Hyper-V on startup.
2. **Inspect VMs**: Check the **Dashboard** tab for registered VirtualBox and Hyper-V VMs. Click **Select** on any VM (for example `ubu-2404-lts`) to target it across tabs.
3. **Create a Baseline Snapshot**: Open **Snapshots**, enter `clean-base`, check **allow mutate (== --yes)**, and click **Create** before making changes to a VM.
4. **Run a Guest Command or Screenshot**: Use **Shot** to capture a live framebuffer PNG in-app, or use `vmcraft-mcp guest exec` / `pipeline naked-test` to run commands inside a Linux or Windows guest.
5. **Restore & Roll Back**: Restore your baseline snapshot from **Snapshots** when finished.

## 2. Functions

- **Dashboard & VMs**: Multi-hypervisor VM discovery (`VBoxManage` + Hyper-V CIM), hardware inspection, create, start (headless), stop (ACPI), and delete.
- **Snapshots**: List, create, restore, and delete VirtualBox snapshots.
- **Network & Disks**: Manage VirtualBox host-only interfaces and inspect/provision `.vdi` disk images.
- **Shot**: Capture live VM framebuffer PNG screenshots and render them inside the GUI.
- **Detonate**: Clean-room airgapped execution pipeline (`airgap_set`, `guestcontrol`, snapshot pre-capture + unconditional rollback).
- **Swarm (Firecracker)**: Probe WSL2 `/dev/kvm` and `firecracker` binary presence, list running Firecracker processes, spawn/kill micro-VMs.
- **Sandbox**: 30-second timeout local subprocess runner for `python`, `powershell`, and `pwsh`.
- **Commands (ArtCraft Bus)**: Browse and dispatch all 32 `Command::dispatch` operations.

## 3. Usage (GUI, CLI & MCP)

- **GUI**: Use the top menu bar (`Session`, `Edit`, `View`, `Command`, `Window`, `Help`) or tab strip. Press `Ctrl+K` (or `Ctrl+Shift+P`) for the Command Palette, `F1` for Keyboard Shortcuts, and `Ctrl+MouseWheel` to zoom.
- **CLI**:
  ```powershell
  vmcraft-mcp status
  vmcraft-mcp vm list
  vmcraft-mcp vm info --vm ubu-2404-lts
  vmcraft-mcp snapshot create --vm ubu-2404-lts --snap clean-base --yes
  vmcraft-mcp pipeline naked-test --repo owner/repo --baseline ubu-2404-lts --snapshot clean-desktop-2026-11-22 --user user --pass user --linux --yes
  ```
- **MCP (`stdio`)**: Run `vmcraft-mcp mcp` (stdout is reserved for JSON-RPC frames; diagnostics go to stderr).

## 4. Tips & Tricks

- Always take a clean snapshot before testing installers or untrusted binaries.
- Use `vmcraft-mcp commands --json` and `vmcraft-mcp describe <cmd>` to inspect exact parameter schemas before scripting.
- For `vmcraft-mcp guest exec`, pass hyphenated guest flags using `=` syntax (for example `"--args=-lc"`).
- Use `Ctrl+1` (or `View -> Reset Zoom`) to snap UI scale back to `1.0`.

## 5. Common Pitfalls & Safety

- **Mutating Safety Gate**: Destructive or state-changing operations (`vm create/start/stop/delete`, `snapshot create/restore/delete`, `network create/remove`, `airgap`, `guest.exec`, `pipeline.detonate`, `pipeline.naked_install`, `firecracker spawn/kill`) are refused unless `--yes`, `VMCRAFT_ALLOW_MUTATE=1`, or the UI `allow mutate` checkbox is enabled.
- **Guest Additions Credentials**: `VBoxManage guestcontrol` requires Guest Additions running inside the guest and valid guest OS credentials (`--user` / `--pass`).
- **WSL2 Firecracker**: `firecracker_*` commands require `wsl.exe` with `/dev/kvm` and the `firecracker` binary installed inside WSL2.

## 6. Planned Extensions

- Native QEMU/KVM (Linux) and Apple Virtualization.framework (macOS) providers.
- Live GUI `--control PORT` bridge.
- Migrate `vmcraft-core/src/llm.rs` to shared `craft-ai-core`.

## 7. Other Crafts

| Craft | Integration / Handoff |
|---|---|
| `devcraft` | Fleet orchestration and clean-room `naked-test` verification target. |
| `wincraft` | Windows host diagnostics; `vmcraft` handles VM/hypervisor isolation. |
| `craft-launcher` | Launches and manages installed Craft workstations. |
