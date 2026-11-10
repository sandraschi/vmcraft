# Changelog — vmcraft

## [2026-11-10] v1.1.0 Firecracker Micro-VM KVM Provider & Sub-10ms Agent Isolation
- L1 (`vmcraft-core`):
  - Added `firecracker` provider module (`crates/vmcraft-core/src/firecracker.rs`):
    - `firecracker_probe`: Probes WSL2 environment for `/dev/kvm` hardware acceleration and `firecracker` binary presence.
    - `firecracker_list`: Inspects running Firecracker micro-VM processes and UNIX domain sockets.
    - `firecracker_spawn`: Bootstraps sub-10ms ephemeral micro-VMs with dedicated guest kernel (`vmlinux`) and `rootfs.ext4` via Unix domain socket REST API.
    - `firecracker_kill`: Forcefully terminates micro-VM instances and purges domain sockets.
  - Command Bus: Registered `firecracker.probe`, `firecracker.list`, `firecracker.spawn`, `firecracker.kill` (32 operations cataloged).
- L2 (`vmcraft-mcp`):
  - CLI: Added `firecracker probe`, `firecracker list`, `firecracker spawn`, `firecracker kill` subcommands.
  - MCP Tools: Added `vmcraft.firecracker_probe`, `vmcraft.firecracker_list`, `vmcraft.firecracker_spawn`, `vmcraft.firecracker_kill`.
  - Mutating confirmation: All micro-VM spawn/kill operations safely gated behind `--yes` / `VMCRAFT_ALLOW_MUTATE=1`.
- L3 (`vmcraft-app`):
  - Visual overhaul: Cyberpunk Slate-950 high-contrast palette, vector theme toggle (sun/moon), glowing status badge pills, persistent airgap status banner.
  - Disassembly Studio: 2-column split-view in Detonate tab with direct `.asm` code viewer and host output loader.
- Gates:
  - Clippy: 0 warnings, zero unwraps/panics (`-D warnings`).
  - Tests: 32 tests passing (`23 core + 7 mcp + 2 app`).
  - Smoke: `vmcraft-app --smoke` passing.
  - Live probe on Goliath: `kvm_available: true`, `host_mode: wsl2_kvm`.

## [2026-11-10] v1.0.0 Tier-4 Clean-Room Detonation Pipeline & Airgap Isolation
- L1 (`vmcraft-core`):
  - Airgap Engine (`airgap_set`): Disconnects all virtual network adapter link cables in real-time (`setlinkstate1..4 off`), disables shared clipboard (`clipboard mode disabled`), and blocks drag-and-drop (`draganddrop disabled`).
  - Guest Control Execution: Added `guest_exec` (`VBoxManage guestcontrol run`), `guest_copyto` (`guestcontrol copyto`), and `guest_copyfrom` (`guestcontrol copyfrom`) for direct guest orchestration without network shares.
  - Reverse-Engineering Detonation Workflow (`pipeline_detonate`):
    - Automates clean-room detonation for untrusted patchers/keygens (e.g., IDA Pro keygen).
    - 1. Takes pre-detonation snapshot `craft-detonate-<ts>`.
    - 2. Powers on VM headlessly and settles guest additions.
    - 3. Hard severs virtual network link cables, clipboard, and drag-and-drop before any payload execution.
    - 4. Copies untrusted keygen/patcher into guest and detonates payload.
    - 5. Injects user-specified target binary into the guest.
    - 6. Executes the potentially tainted disassembler (`ida64.exe` batch mode) *strictly inside* the air-gapped guest.
    - 7. Extracts only safe disassembly output (`.asm` / text artifacts) back to the host.
    - 8. Captures post-analysis screenshot for inspection.
    - 9. RAII unconditional nuke: Powers down guest and rolls back VM state to pristine pre-detonation snapshot.
  - Consumer Naked Install Verification (`pipeline_naked_install`):
    - Clones/tests target repository in a pristine Windows baseline VM to verify zero-dependency consumer install experiences.
  - Command Bus: Registered `airgap.set`, `guest.exec`, `guest.copyto`, `guest.copyfrom`, `pipeline.detonate`, `pipeline.naked_install` (28 commands total in catalog).
- L2 (`vmcraft-mcp`):
  - CLI Subcommands: Added `pipeline detonate`, `pipeline naked-test`, `airgap`, and `guest` subcommands.
  - MCP Tools: Added `vmcraft.pipeline_detonate`, `vmcraft.pipeline_naked_install`, `vmcraft.airgap_set`, `vmcraft.guest_exec`, `vmcraft.guest_copyto`, and `vmcraft.guest_copyfrom`.
  - Safety Gates: Enforced `--yes` / `VMCRAFT_ALLOW_MUTATE=1` confirmation for all detonation, airgap, and guest operations.
- L3 (`vmcraft-app`):
  - Added **Detonate Tab** (10th tab in desktop app):
    - Interactive clean-room detonation studio.
    - Live target VM selection, guest credentials, keygen path, target binary, disassembler executable, and guest/host output paths.
    - Direct buttons for "💥 Detonate & Disassemble in Airgap", "✂ Sever Network Link", "🔌 Reconnect Network", and "🚨 Emergency Nuke & Rollback".
- Gates:
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: 0 warnings, zero unwraps/panics.
  - `cargo test --workspace`: 30 passed (21 core + 7 mcp + 2 app).
  - `cargo fmt --check`: clean.
  - Smoke binary passes in debug and release.

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
