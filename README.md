# vmcraft

![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![Rust 1.90+](https://img.shields.io/badge/rust-1.90%2B-orange?logo=rust)
![MCP stdio](https://img.shields.io/badge/MCP-stdio-purple)
![ArtCraft harness](https://img.shields.io/badge/ArtCraft-harness-green)

Pure-Rust ArtCraft Virtual Machine workstation for local virtualization management and agentic tooling. One unified Rust workspace providing an in-process core domain library (L1), a dual-mode human CLI and JSON-RPC Model Context Protocol (MCP) server (L2), and an interactive native egui desktop workstation (L3).

Status: **v0.1.0 (pre-1.0, T2 test floor met; 60 workspace tests; CI green)** - L1 core + L2 CLI/MCP + L3 egui desktop app, multi-hypervisor VirtualBox 7+, Hyper-V, and Firecracker KVM Micro-VMs (via WSL2), Linux/UEFI/Fido ISO Library & Fetcher, ArtCraft Command Bus, Clean-Room Detonation Pipeline & Airgap Isolation.

- User Help Guide: [HELP.md](HELP.md)

- Changes: [CHANGELOG.md](CHANGELOG.md)
- Conversion Dossier: [docs/CONVERSION_DOSSIER.md](docs/CONVERSION_DOSSIER.md)
- Audit Log: [docs/CONVERSION_LOG.md](docs/CONVERSION_LOG.md)
- Agent Guide: [AGENTS.md](AGENTS.md)
- Handover: [HANDOVER.md](HANDOVER.md)

## Install

Tell Cursor, Claude, Antigravity, or OpenCode: **Install this.**

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-craft.ps1
```

Installs the Windows app and the CLI (Start Menu shortcut, CLI on the user PATH). `-RegisterClients` also registers the MCP exe with the IDEs that are installed. `-Clients cursor,claude-desktop` limits that list. `-Mcpb` installs the Claude Desktop extension bundle.

---

## Architecture

| Crate | Layer | Purpose | Docs |
|---|---|---|---|
| `crates/vmcraft-core` | L1 | In-process command dispatch bus (`Command::dispatch`), ArtCraft command bus registry (`commands_catalog`, `describe_command`, `parse_command`), VBoxManage orchestration, Hyper-V CIM probe, Linux/UEFI/Fido ISO catalog & SHA-256 fetcher (`iso.rs`), ephemeral sandbox execution, strict input validators. | [crates/vmcraft-core/README.md](crates/vmcraft-core/README.md) |
| `crates/vmcraft-mcp` | L2 | Binary providing human CLI subcommands (`commands --json`, `describe`, `run`, `batch`, `vm`, `hyperv`, `snapshot`, `iso`, etc.) and standard Model Context Protocol (MCP) JSON-RPC stdio. Dual-aliased to support both modern `vmcraft.*` names and legacy portmanteaus (`vm_management`, `snapshot_management`, `hyperv_management`, etc.). | [crates/vmcraft-mcp/README.md](crates/vmcraft-mcp/README.md) |
| `crates/vmcraft-app` | L3 | Native eframe/egui desktop application with 12 tabs (`Dashboard`, `VMs`, `Snapshots`, `Network`, `Disks`, `ISOs`, `Shot`, `Detonate`, `Swarm`, `Sandbox`, `Commands`, `Help`), non-blocking asynchronous message passing (`mpsc::channel`), live multi-hypervisor VM table, ISO library & fetcher, snapshot controls, in-app framebuffer image viewer, command bus explorer, and safety gating. | [crates/vmcraft-app/README.md](crates/vmcraft-app/README.md) |

---

## Feature Matrix

| Feature | Status | Notes |
|---|---|---|
| VirtualBox 7+ lifecycle (`vm_list`, `vm_info`, `vm_create`, `vm_start`, `vm_stop`, `vm_delete`) | Implemented | Shells out to `VBoxManage`; mutating ops gated behind `--yes` / `VMCRAFT_ALLOW_MUTATE=1`. |
| Snapshots, Host-Only Networks, `.vdi` Disks, Framebuffer Screenshots | Implemented | `snapshot_*`, `network_*`, `disk_*`, `vm_shot` with in-app PNG preview. |
| Hyper-V discovery & start/stop (`hyperv_list`, `hyperv_start`, `hyperv_stop`) | Implemented | Windows-only via PowerShell CIM; returns `status: down` when absent. |
| ArtCraft Command Bus (`commands_catalog`, `describe_command`, `parse_command`, `dispatch`) | Implemented | 37 cataloged operations shared across L1/L2/L3. |
| Linux, UEFI Shell & External-Fido Windows ISO Library (`iso_catalog`, `iso_list`, `iso_verify`, `iso_fetch`, `iso_attach`) | Implemented | Curated Linux + UEFI Shell HTTPS streaming fetcher with SHA-256 sidecars, VirtualBox optical attach, and optional external `Fido.ps1` Windows URL resolver with 20h cache (GPLv3 code never bundled). |
| Ephemeral Sandbox (`sandbox_run`) | Implemented | 30s timeout local subprocess runner for `python` / `powershell` / `pwsh`. |
| Airgap & Guest Control (`airgap_set`, `guest_exec`, `guest_copyto`, `guest_copyfrom`) | Implemented | Uses `VBoxManage` link-state / clipboard / DnD disablement and `guestcontrol`. |
| Clean-Room Detonation & Naked Install (`pipeline_detonate`, `pipeline_naked_install`) | Partial | Implemented over VirtualBox `guestcontrol`; requires pre-configured baseline VM + Guest Additions. |
| Firecracker KVM Micro-VMs (`firecracker_probe`, `firecracker_list`, `firecracker_spawn`, `firecracker_kill`) | Partial | Windows-host WSL2 bridge (`wsl.exe` + `/dev/kvm`); requires `firecracker` + kernel/rootfs inside WSL2. |
| QEMU / native Linux KVM / macOS Virtualization.framework providers | Planned | Not yet implemented (`HANDOVER.md` next step). |
| Live GUI control port (`--control PORT`) & `craft-ai-core` chat migration | Planned | Own `llm.rs` exists in `vmcraft-core`; control port not yet wired. |

---

## Desktop Workstation (`vmcraft-app`)

Launch via:
```powershell
.\start.ps1
# or via fleet launcher:
..\mcp-central-docs\starts\vmcraft-start.bat
```

### Features
- **Standard Craft Menu Bar & Help Center**: Top menu bar (`Session`, `Edit`, `View`, `Command`, `Window`, `Help`) with `switch_on_hover`, hideable tab strip (`View -> Show Tab Bar` / `Ctrl+B`), switchable status bar (`View -> Status Bar -> Bottom/Top/Off`), `Ctrl+K` Command Palette, `F1` Keyboard Shortcuts modal, and 7-tab Help Center ([HELP.md](HELP.md)).
- **Theme-Aware Dark & Light UI**: High-contrast dark slate and crisp light themes with rounded cards, live status pills, and `Ctrl+MouseWheel` / `Ctrl+=` / `Ctrl+-` / `Ctrl+1` zoom.
- **Instant Background Discovery**: Automatically polls hypervisors (VirtualBox + Hyper-V), live VM running states (`RUNNING` / `STOPPED`), and host templates upon opening without blocking the UI thread.
- **Dashboard**: 4 KPI summary cards (`Total VMs`, `Running Now`, `Selected Target`, `Quick Jump`), searchable/filterable VM table (`All`, `Running`, `VirtualBox`, `Hyper-V`) with live state and provider pills, and a readable `Recent Activity` feed (with collapsible raw JSON).
- **VMs Page**: Quick target selector, VM provisioning form, structured VM table with live state pills, parsed `Selected VM Details` grid, and an in-GUI **Guest Runner & Clean-Room Linux Craft Test** card (`guest_exec` and `pipeline_naked_install`).
- **Snapshots Page**: Point-in-time snapshot listing in a structured table, with one-click `Restore` and `Delete` per snapshot.
- **Network Page**: VirtualBox host-only network interface management (`list`, `create`, `remove`) rendered in a structured table.
- **Disks Page**: Virtual disk image (`.vdi`) inspection in a structured table (`State`, `Format`, `Capacity`, `Location`, `UUID`) and dynamic provisioning.
- **ISOs Page**: Curated Linux (Alpine, Ubuntu, Debian, Fedora, Arch), UEFI Shell 2.2 (Release/Debug), and optional external-Fido Windows ISO catalog with category filters, live search, streaming HTTPS download + SHA-256 verification, and one-click attach to the selected VirtualBox VM.
- **Shot Page**: Framebuffer screenshot capture with live in-app PNG rendering directly inside the GUI.
- **Sandbox Page**: Safe, isolated 30-second script runner with a Python (`python`/`py`/`python3` auto-detect) and PowerShell selector and formatted stdout/stderr output.
- **Detonate Page**: Clean-room detonation and disassembly pipeline under hardware-level airgap. Supports automated snapshotting, keygen explosion, target binary injection, in-guest disassembly, non-executable artifact extraction, and auto-nuke/rollback.
- **Commands Page**: Interactive ArtCraft command bus catalog and custom command dispatcher.
- **Help & Safety**: Visual toggle for mutating operations gate.

---

## MCP Server Configuration

To register `vmcraft` as an MCP server in Claude Desktop, OpenCode, or Cursor:

```json
{
  "mcpServers": {
    "vmcraft": {
      "command": "D:/Dev/repos/vmcraft/target/release/vmcraft-mcp.exe",
      "args": ["mcp"]
    }
  }
}
```

### MCP Tools & Dual Aliases
All tools are dual-aliased to ensure 100% backward compatibility with legacy tooling:
- `vmcraft.status` / `virtualization_shutdown` / `info_tools`
- `vmcraft.vm_list`, `vmcraft.vm_info`, `vmcraft.vm_create`, `vmcraft.vm_start`, `vmcraft.vm_stop`, `vmcraft.vm_delete` (legacy alias: `vm_management`)
- `vmcraft.snapshot_list`, `vmcraft.snapshot_create`, `vmcraft.snapshot_restore`, `vmcraft.snapshot_delete` (legacy alias: `snapshot_management`)
- `vmcraft.network_list`, `vmcraft.network_create`, `vmcraft.network_remove` (legacy alias: `network_management`)
- `vmcraft.disk_list`, `vmcraft.disk_create` (legacy alias: `storage_management`)
- `vmcraft.vm_shot` (captures PNG screenshot)
- `vmcraft.host_info` (legacy alias: `system_management`)
- `vmcraft.sandbox_run` (legacy alias: `create_sandbox`, `stop_sandbox`)
- `vmcraft.hyperv_list`, `vmcraft.hyperv_start`, `vmcraft.hyperv_stop` (legacy alias: `hyperv_management`)
- `vmcraft.commands`, `vmcraft.describe`, `vmcraft.run` (ArtCraft Command Bus)
- `vmcraft.airgap_set` (Hardware/link-level network disconnect & clipboard/DnD block)
- `vmcraft.guest_exec`, `vmcraft.guest_copyto`, `vmcraft.guest_copyfrom` (In-guest orchestration via guest additions)
- `vmcraft.pipeline_detonate` (Untrusted keygen explosion, in-guest disassembly under airgap, clean extraction, and auto-rollback)
- `vmcraft.pipeline_naked_install` (Clean baseline consumer install verification)
- `vmcraft.firecracker_probe`, `vmcraft.firecracker_list`, `vmcraft.firecracker_spawn`, `vmcraft.firecracker_kill` (Sub-10ms KVM micro-VMs)
- `vmcraft.iso_catalog`, `vmcraft.iso_list`, `vmcraft.iso_verify`, `vmcraft.iso_fetch`, `vmcraft.iso_attach` (Curated Linux/UEFI/Fido ISO catalog, local library scanner, streaming SHA-256 verifier, HTTPS fetcher, and VirtualBox optical attach)

---

## CLI Usage

```powershell
# Check hypervisor status
vmcraft-mcp status

# List all registered VMs (VirtualBox + Hyper-V)
vmcraft-mcp vm list

# Inspect detailed hardware configuration of a VM
vmcraft-mcp vm info --vm "ubu-2404-lts"

# Start a VM (mutating operations require --yes or VMCRAFT_ALLOW_MUTATE=1)
vmcraft-mcp vm start --vm "ubu-2404-lts" --yes

# Take a snapshot
vmcraft-mcp snapshot create --vm "ubu-2404-lts" --snap "clean-state" --yes

# Capture a screenshot from a running VM
vmcraft-mcp shot --vm "ubu-2404-lts" --out "screenshot.png"

# Execute code in the safe isolated sandbox
vmcraft-mcp sandbox run --lang python --code "print('hello from sandbox')"

# Explore ArtCraft command bus catalog
vmcraft-mcp commands --json
vmcraft-mcp describe pipeline.detonate

# Clean-room air-gapped detonation & disassembly of untrusted binary
vmcraft-mcp pipeline detonate `
  --vm "win11" `
  --keygen "C:\malware\keygen.exe" `
  --target "C:\bin\target.exe" `
  --disassembler "ida64.exe" `
  --host-output "output.asm" `
  --yes

# Immediately sever all network links and clipboard to airgap a VM
vmcraft-mcp airgap "win11" --disconnect

# Verify clean-room consumer install on pristine baseline
vmcraft-mcp pipeline naked-test --repo "my-org/my-app" --yes

# Probe WSL2 KVM acceleration for Firecracker micro-VMs
vmcraft-mcp firecracker probe

# Spawn a dedicated sub-10ms ephemeral micro-VM
vmcraft-mcp firecracker spawn --id "agent-worker-1" --kernel "/vmlinux" --rootfs "/rootfs.ext4" --yes

# Browse curated Linux & UEFI Shell ISOs, download with SHA-256 sidecar, and attach to a VM
vmcraft-mcp iso catalog --category linux
vmcraft-mcp iso fetch --id uefi-shell-2.2-release --yes
vmcraft-mcp iso list
vmcraft-mcp iso verify --path "$env:USERPROFILE\.vmcraft\isos\UEFI-Shell-2.2-24H2-RELEASE.iso"
vmcraft-mcp iso attach --vm "ubu-2404-lts" --iso "$env:USERPROFILE\.vmcraft\isos\UEFI-Shell-2.2-24H2-RELEASE.iso" --yes
```

---

## Safety & Containment Rules

- **Mutating Operations Gate**: Operations that modify system or VM state (`vm create/start/stop/delete`, `snapshot create/restore/delete`, `network create/remove`, `disk create`, `airgap`, `guest.exec`, `pipeline.detonate`, `pipeline.naked_install`, `firecracker spawn/kill`) require explicit confirmation via `--yes`, `VMCRAFT_ALLOW_MUTATE=1`, or the UI checkbox.
- **Zero-Unwrap Policy**: Workspace compiles under strict denial of `unwrap_used`, `expect_used`, `panic`, `todo`, and `unimplemented`.
- **Stdio Isolation**: The MCP server guarantees that `stdout` is exclusively reserved for valid JSON-RPC frames; all diagnostic logs are directed to `stderr`.

---

## Verification & Testing

```powershell
# Run full workspace test suite (60 tests)
cargo test --workspace

# Lint with strict zero-warning policy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Verify formatting
cargo fmt --all --check

# Test application smoke harness
cargo run -p vmcraft-app --release -- --smoke
```
