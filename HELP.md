# vmcraft Help & User Guide

Pure-Rust ArtCraft Virtual Machine workstation (`vmcraft-core` L1, `vmcraft-mcp` L2 CLI + MCP stdio, `vmcraft-app` L3 egui desktop).

## 1. Get Started

1. **Launch**: Run `.\start.ps1` (or `cargo run -p vmcraft-app`) to open the desktop workstation. Status auto-polls VirtualBox and Hyper-V on startup.
2. **Inspect VMs**: Check the **Dashboard** tab for registered VirtualBox and Hyper-V VMs. Click **Select** on any VM (for example `ubu-2404-lts`) to target it across tabs.
3. **Create a Baseline Snapshot**: Open **Snapshots**, enter `clean-base`, check **allow mutate (== --yes)**, and click **Create** before making changes to a VM.
4. **Run a Guest Command or Screenshot**: Use **Shot** to capture a live framebuffer PNG in-app, or use `vmcraft-mcp guest exec` / `pipeline naked-test` to run commands inside a Linux or Windows guest.
5. **Restore & Roll Back**: Restore your baseline snapshot from **Snapshots** when finished.
6. **Provision & Attach ISOs**: Open the **ISOs** tab to browse curated Linux images (Alpine, Ubuntu LTS, Debian, Fedora, Arch) and EDK2 UEFI Shell 2.2 boot ISOs. Use **Resolve URL** to inspect the HTTPS link, **Download ISO** (with `allow mutate` enabled) to stream it with automatic `.sha256` sidecar generation, and **Attach to VM** to mount it on the selected VirtualBox VM's optical drive.

## 2. Functions

- **Dashboard & VMs**: 4 KPI summary cards, search/filter chips (`All`, `Running`, `VirtualBox`, `Hyper-V`), live `RUNNING` / `STOPPED` status pills (`VBoxManage list runningvms` + Hyper-V CIM), parsed `Selected VM Details`, in-GUI **Guest Runner & Clean-Room Linux Craft Test** (`guest_exec` + `pipeline_naked_install`), and VM create/start/stop/delete.
- **Snapshots**: List, create, restore, and delete VirtualBox snapshots.
- **Network & Disks**: Structured tables for VirtualBox host-only interfaces and `.vdi` disk images (`State`, `Format`, `Capacity`, `Location`, `UUID`).
- **ISOs (Library & Fetcher)**: Curated Linux (Alpine 3.21 Virt, Ubuntu 24.04.2 Server/Desktop, Debian 12 Netinst, Fedora 41, Arch Latest) and UEFI Shell 2.2 (Release/Debug) HTTPS streaming downloads with atomic `.part` -> `.iso` promotion, 1 MiB chunked SHA-256 verification, `.sha256` sidecars in `~/.vmcraft/isos` (`VMCRAFT_ISO_DIR`), and one-click VirtualBox optical attach (`IDE`/`SATA` auto-detect). Optional external `Fido.ps1` (GPLv3, never bundled) resolves Windows 10/11 signed Microsoft CDN URLs with a 20-hour local cache (`.fido-url-cache.json`) to avoid Microsoft Sentinel rate-limits.
- **Shot**: Capture live VM framebuffer PNG screenshots and render them inside the GUI.
- **Detonate**: Clean-room airgapped execution pipeline (`airgap_set`, `guestcontrol`, snapshot pre-capture + unconditional rollback).
- **Swarm (Firecracker)**: Probe WSL2 `/dev/kvm` and `firecracker` binary presence, list running Firecracker processes, spawn/kill micro-VMs.
- **Sandbox**: 30-second timeout local subprocess runner with a Python (`python`/`py`/`python3`) and PowerShell selector and formatted stdout/stderr output.
- **Commands (ArtCraft Bus)**: Browse and dispatch all 32 `Command::dispatch` operations.

## 3. Usage (GUI, CLI & MCP)

- **GUI**: Use the top menu bar (`Session`, `Edit`, `View`, `Command`, `Window`, `Help`) or the hideable single-click tab strip (`View -> Show Tab Bar` / `Ctrl+B`). Switch the status bar position via `View -> Status Bar` (`Bottom`, `Top`, `Off`). Press `Ctrl+K` (or `Ctrl+Shift+P`) for the Command Palette, `F1` for Keyboard Shortcuts, and `Ctrl+MouseWheel` / `Ctrl+=` / `Ctrl+-` / `Ctrl+1` to zoom.
- **CLI**:
  ```powershell
  vmcraft-mcp status
  vmcraft-mcp vm list
  vmcraft-mcp vm info --vm ubu-2404-lts
  vmcraft-mcp snapshot create --vm ubu-2404-lts --snap clean-base --yes
  vmcraft-mcp pipeline naked-test --repo owner/repo --baseline ubu-2404-lts --snapshot clean-desktop-2026-11-22 --user user --pass user --linux --yes
  vmcraft-mcp iso catalog --category linux
  vmcraft-mcp iso fetch --id uefi-shell-2.2-release --yes
  vmcraft-mcp iso list
  vmcraft-mcp iso verify --path "$env:USERPROFILE\.vmcraft\isos\UEFI-Shell-2.2-24H2-RELEASE.iso"
  vmcraft-mcp iso attach --vm ubu-2404-lts --iso "$env:USERPROFILE\.vmcraft\isos\UEFI-Shell-2.2-24H2-RELEASE.iso" --yes
  ```
- **MCP (`stdio`)**: Run `vmcraft-mcp mcp` (stdout is reserved for JSON-RPC frames; diagnostics go to stderr).

## 4. Tips & Tricks

- Always take a clean snapshot before testing installers or untrusted binaries.
- Use `vmcraft-mcp commands --json` and `vmcraft-mcp describe <cmd>` to inspect exact parameter schemas before scripting.
- For `vmcraft-mcp guest exec`, pass hyphenated guest flags using `=` syntax (for example `"--args=-lc"`).
- Use `Ctrl+1` (or `View -> Reset Zoom`) to snap UI scale back to `1.0`.
- Use `alpine-3.21-virt` (~62 MB) or `uefi-shell-2.2-release` (~12 MB) from **ISOs** for fast boot/optical-drive smoke tests.
- Microsoft's Sentinel rejects rapid back-to-back `Fido.ps1 -GetUrl` calls; `vmcraft` caches resolved Windows ISO URLs for 20 hours in `.fido-url-cache.json` automatically.

## 5. Common Pitfalls & Safety

- **Mutating Safety Gate**: Destructive or state-changing operations (`vm create/start/stop/delete`, `snapshot create/restore/delete`, `network create/remove`, `airgap`, `guest.exec`, `pipeline.detonate`, `pipeline.naked_install`, `firecracker spawn/kill`) are refused unless `--yes`, `VMCRAFT_ALLOW_MUTATE=1`, or the UI `allow mutate` checkbox is enabled.
- **Guest Additions Credentials**: `VBoxManage guestcontrol` requires Guest Additions running inside the guest and valid guest OS credentials (`--user` / `--pass`).
- **WSL2 Firecracker**: `firecracker_*` commands require `wsl.exe` with `/dev/kvm` and the `firecracker` binary installed inside WSL2.
- **HTTPS-Only ISO Downloads & Safety Gate**: `iso.fetch` rejects `http://`, `file://`, and path-traversal filenames; downloads and `iso.attach` require `--yes` / `allow mutate`, while `iso catalog`, `iso list`, `iso verify`, and `iso fetch --url-only` are read-only.

## 6. Planned Extensions

- Native QEMU/KVM (Linux) and Apple Virtualization.framework (macOS) providers.
- Live GUI `--control PORT` bridge.
- Migrate `vmcraft-core/src/llm.rs` to shared `craft-ai-core`.
- Pure-Rust `autounattend.xml` and `cloud-init` NoCloud seed ISO generator paired with the **ISOs** tab.

## 7. Other Crafts

| Craft | Integration / Handoff |
|---|---|
| `devcraft` | Fleet orchestration and clean-room `naked-test` verification target. |
| `wincraft` | Windows host diagnostics; `vmcraft` handles VM/hypervisor isolation. |
| `craft-launcher` | Launches and manages installed Craft workstations. |
