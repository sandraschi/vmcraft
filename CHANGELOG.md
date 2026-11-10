# Changelog — vmcraft

## [2026-11-10] v0.3.0 Tier-3 ArtCraft command bus and multi-hypervisor workstation
- L1 (`vmcraft-core`):
  - ArtCraft Command Bus: Added `commands_catalog()` (22 commands), `describe_command(id)`, and `parse_command(id, params)` conforming to `mcp-central-docs/standards/rust-artcraft/CRAFT_COMMAND_BUS.md`.
  - Multi-Hypervisor: Added native Windows Hyper-V integration (`hyperv_probe`, `hyperv_list`, `hyperv_start`, `hyperv_stop`) via PowerShell CIM querying without requiring external modules.
  - Multi-Hypervisor VM list: Aggregates both VirtualBox and Hyper-V machines with explicit provider tags (`virtualbox` vs `hyperv`).
  - Structured Snapshots: Parsed machine-readable snapshot hierarchy into structured records `[{name, uuid, description}]`.
- L2 (`vmcraft-mcp`):
  - CLI Command Bus: Added `commands --json`, `describe <id>`, `run --cmd <id> [--params <json>]`, and `batch --script <file>` subcommands.
  - CLI Hyper-V: Added `hyperv list|start|stop` subcommands with `--yes` safety confirmation.
  - MCP Tools: Added `vmcraft.commands`, `vmcraft.describe`, `vmcraft.run`, `vmcraft.hyperv_list`, `vmcraft.hyperv_start`, `vmcraft.hyperv_stop`, and backward-compatible `hyperv_management` routing.
- L3 (`vmcraft-app`):
  - Added Commands Bus Tab: Interactive registry browser and in-app command dispatcher.
  - Multi-Hypervisor Dashboard: VM table renders provider badges (`VirtualBox` in amber, `Hyper-V` in cyan) with provider-specific start/stop/info actions.
  - Top Bar Status: Displays presence and health of both VirtualBox and Hyper-V hypervisors.
  - Snapshots Tab: Structured interactive table with direct Restore and Delete buttons per snapshot.
  - Shot Tab: In-app framebuffer viewer using `image` decoding to render live guest screenshots directly inside the GUI without external viewers.
- Cleanups:
  - Removed stale installer NSIS configuration per ArtCraft application standard (no NSIS bundle required).
- Gates:
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: 0 warnings, zero unwraps/panics.
  - `cargo test --workspace`: 27 passed (19 core + 6 mcp + 2 app).
  - `cargo fmt --check`: clean.
  - Smoke binary passes in debug and release.

## [2026-11-10] v0.2.1 Fix startup freeze and auto-load VM list in L3 workstation app
- L3: Non-blocking async channels (`mpsc::channel`) decoupled from egui UI loop, eliminating thread blocks from synchronous `block_on`.
- L3: Auto-refresh on startup immediately queries `vmcraft.status`, `vmcraft.vm_list`, and `vmcraft.host_info`, ending the "stuck in starting… with empty VM list" condition.
- L3: Full interactive VM table on Dashboard with instant Select/Info/Start/Stop controls and quick VM selector on the VMs page.
- L1: `vboxmanage_candidates` checks `C:\Program Files\Oracle\VirtualBox\VBoxManage.exe` existence first before falling back to PATH, avoiding process spawning delays.
- Gates: `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean, `cargo test --workspace` 25 passed, `cargo fmt --check` clean.

## [2026-11-08] v0.2.0 Tier-2 mutating verbs + full L3 pages
- L1: `vm create|start|stop|delete`, `snapshot create|restore|delete`, `network create|remove` (hostonlyif), `disk list|create (.vdi)`, `vm shot` (screenshotpng + PNG IHDR validate), all mutating behind `confirm || VMCRAFT_ALLOW_MUTATE=1`, strict name/IP/path validators, injection tests.
- L2: CLI `--yes` on every mutating verb (`vm/network/snapshot/disk`), new `disk` + `shot` tops, 19 MCP tools, dual aliases (`vmcraft.*` + legacy portmanteaus), `mutate_without_confirm_is_refused` test.
- L3: 8 pages (Dashboard, VMs, Snapshots, Network, Disks, Shot, Sandbox, Help) + mutate checkbox (= --yes), `--smoke` ok.
- Bug bounty: `network list` used wrong verb `hostonlynets` (down receipt on Goliath); fixed to `hostonlyifs` → `status:ok`, caught by live Oracle, locked by real output.
- Gates: clippy clean, `cargo test --workspace` 23 passed (19 core + 4 mcp), fmt clean.
- Live Oracle Goliath: 3 VMs, real snapshot `clean-desktop-2026-10-10` on ubu-2404-lts, real VDI list, `vm start` without `--yes` correctly refused (no VM touched), `shot` on stopped VM errors gracefully.

## [2026-11-08] v0.1.0 Tier-1 skeleton
- Workspace stamp (Phase 0/1): kernel from `rust-devices-mcp`, `vmcraft-core/mcp/app` crates.
- L1 `Command::dispatch`: status, vm list/info, snapshot list, network list, host info, sandbox run (python/pwsh, 30s timeout).
- L2 `vmcraft-mcp`: CLI `--json` + `mcp` stdio (pure JSON-RPC), dual aliases (`vmcraft.*` + legacy `vm_management`/`snapshot_management`/`network_management`/`system_management`/`sandbox_management`).
- L3 `vmcraft-app`: Tier-1 egui shell (status, vm list/info, host info, sandbox runner), `--smoke` ok.
- Gates: `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean, `cargo test --workspace` 18 passed, `cargo fmt --check` clean.
- Live Oracle 2026-11-08 Goliath: `vbox_present:true`, `vm list` 3 VMs (win11-automated, ubu-2404-lts, batocera-clean), MCP `initialize` ok.
