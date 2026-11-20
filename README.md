# vmcraft

![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![Rust 1.90+](https://img.shields.io/badge/rust-1.90%2B-orange?logo=rust)
![MCP stdio](https://img.shields.io/badge/MCP-stdio-purple)
![ArtCraft harness](https://img.shields.io/badge/ArtCraft-harness-green)

Pure-Rust ArtCraft Virtual Machine workstation for local virtualization management and agentic tooling. One unified Rust workspace providing an in-process core domain library (L1), a dual-mode human CLI and JSON-RPC Model Context Protocol (MCP) server (L2), and an interactive native egui desktop workstation (L3).

Status: **v1.1.0, Tier 4** (L1 core + L2 CLI/MCP + L3 egui desktop app, multi-hypervisor VirtualBox 7+, Hyper-V, and Firecracker KVM Micro-VMs, ArtCraft Command Bus, Clean-Room Detonation Pipeline & Airgap Isolation).

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

Installs the Windows app, the CLI, and the MCP bundle.

---

## Architecture

| Crate | Layer | Purpose |
|---|---|---|
| `crates/vmcraft-core` | L1 | In-process command dispatch bus (`Command::dispatch`), ArtCraft command bus registry (`commands_catalog`, `describe_command`, `parse_command`), VBoxManage orchestration, Hyper-V CIM probe, ephemeral sandbox execution, strict input validators. |
| `crates/vmcraft-mcp` | L2 | Binary providing human CLI subcommands (`commands --json`, `describe`, `run`, `batch`, `vm`, `hyperv`, `snapshot`, etc.) and standard Model Context Protocol (MCP) JSON-RPC stdio. Dual-aliased to support both modern `vmcraft.*` names and legacy portmanteaus (`vm_management`, `snapshot_management`, `hyperv_management`, etc.). |
| `crates/vmcraft-app` | L3 | Native eframe/egui desktop application with 9 functional tabs, non-blocking asynchronous message passing (`mpsc::channel`), live multi-hypervisor VM table, snapshot tree controls, in-app framebuffer image viewer, command bus explorer, and safety gating. |

---

## Desktop Workstation (`vmcraft-app`)

Launch via:
```powershell
.\start.ps1
# or via fleet launcher:
..\mcp-central-docs\starts\vmcraft-start.bat
```

### Features
- **Instant Background Discovery**: Automatically polls hypervisors (VirtualBox + Hyper-V), VM list, and host templates upon opening without blocking the UI thread.
- **Dashboard**: Live multi-hypervisor VM overview table showing VM Name, Provider (`VirtualBox` in amber, `Hyper-V` in cyan), UUID, and instant action buttons (`Select`, `Info`, `Start`, `Stop`), plus an activity log.
- **VMs Page**: Quick target selector bar, VM configuration form, start/stop/create/delete controls, and detailed hardware inspect output.
- **Snapshots Page**: Point-in-time snapshot listing in a structured table, with one-click `Restore` and `Delete` per snapshot.
- **Network Page**: VirtualBox host-only network interface management (`list`, `create`, `remove`).
- **Disks Page**: Virtual disk image (`.vdi`) inspection and dynamic provisioning.
- **Shot Page**: Framebuffer screenshot capture with live in-app PNG rendering directly inside the GUI.
- **Sandbox Page**: Safe, isolated 30-second script runner for Python and PowerShell.
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
```

---

## Safety & Containment Rules

- **Mutating Operations Gate**: Operations that modify system or VM state (`vm create/start/stop/delete`, `snapshot create/restore/delete`, `network create/remove`, `disk create`, `airgap`, `guest.exec`, `pipeline.detonate`, `pipeline.naked_install`, `firecracker spawn/kill`) require explicit confirmation via `--yes`, `VMCRAFT_ALLOW_MUTATE=1`, or the UI checkbox.
- **Zero-Unwrap Policy**: Workspace compiles under strict denial of `unwrap_used`, `expect_used`, `panic`, `todo`, and `unimplemented`.
- **Stdio Isolation**: The MCP server guarantees that `stdout` is exclusively reserved for valid JSON-RPC frames; all diagnostic logs are directed to `stderr`.

---

## Verification & Testing

```powershell
# Run full workspace test suite (30 tests)
cargo test --workspace

# Lint with strict zero-warning policy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Verify formatting
cargo fmt --all --check

# Test application smoke harness
cargo run -p vmcraft-app --release -- --smoke
```
