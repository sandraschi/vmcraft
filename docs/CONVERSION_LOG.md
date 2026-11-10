# Conversion Audit Log: `rust-vmcraft-mcp`

## [2026-11-10] Phase Tier-4 v1.0.0 Clean-Room Detonation Pipeline & Airgap Isolation (agent)
- L1 (`vmcraft-core`):
  - Airgap Engine (`airgap_set`): Dynamic severance of virtual network link cables across adapters 1..4 (`setlinkstateN off`), clipboard disabled, drag-and-drop disabled.
  - In-guest execution via guest additions: `guest_exec`, `guest_copyto`, `guest_copyfrom` (no network shares required).
  - Clean-room detonation pipeline (`pipeline_detonate`): RAII automated snapshotting, headless poweron, hard airgap severance, untrusted keygen/patcher injection & explosion, target injection, in-guest batch disassembler (`ida64.exe`) execution, non-executable output extraction (`.asm`), and unconditional auto-rollback to pre-detonation state.
  - Naked consumer install validation (`pipeline_naked_install`): Automated pristine baseline verification.
  - ArtCraft Command Bus expanded to 28 registered commands.
- L2 (`vmcraft-mcp`):
  - Added CLI subcommands: `pipeline detonate`, `pipeline naked-test`, `airgap`, `guest run`, `guest copyto`, `guest copyfrom`.
  - Added MCP tools: `vmcraft.pipeline_detonate`, `vmcraft.pipeline_naked_install`, `vmcraft.airgap_set`, `vmcraft.guest_exec`, `vmcraft.guest_copyto`, `vmcraft.guest_copyfrom`.
  - Safety gate: All mutating verbs strictly require `--yes` or `VMCRAFT_ALLOW_MUTATE=1`.
- L3 (`vmcraft-app`):
  - Added 10th tab **Detonate**: Dedicated clean-room detonation studio with parameters for target VM, credentials, keygen, target binary, disassembler executable, output paths, and emergency rollback.
- Gates:
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: 0 warnings, zero unwraps/panics.
  - `cargo test --workspace`: 30 passed (21 core + 7 mcp + 2 app).
  - `cargo fmt --check`: clean.
  - Smoke binary passes in debug and release.

## [2026-11-10] Phase Tier-3 v0.3.0 ArtCraft Command Bus & Multi-Hypervisor Workstation (agent)
- L1: ArtCraft Command Bus with 22 operations, native Windows Hyper-V CIM probe/list/start/stop, multi-hypervisor combined list, structured snapshots.
- L2: CLI command bus (`commands`, `describe`, `run`, `batch`), CLI Hyper-V (`hyperv list|start|stop`), MCP tools.
- L3: 9 tabs, Command Bus browser tab, multi-hypervisor badges on Dashboard, live snapshot table with restore/delete, in-app framebuffer PNG viewer.
- Gates: clippy clean, 27 tests passed, fmt clean, smoke ok.

## [2026-11-10] Phase Tier-2 v0.2.1 Fix startup freeze and auto-load VM list in L3 workstation app (agent)
- L3: Non-blocking async channels (`mpsc::channel`) decoupled from egui UI loop, eliminating thread freezes from synchronous `block_on`.
- L3: Auto-refresh on startup immediately queries `vmcraft.status`, `vmcraft.vm_list`, and `vmcraft.host_info`, ending the "stuck in starting… with empty VM list" condition.
- L3: Full interactive VM table on Dashboard with instant Select/Info/Start/Stop controls and quick VM selector on the VMs page.
- L1: `vboxmanage_candidates` checks `C:\Program Files\Oracle\VirtualBox\VBoxManage.exe` existence first before falling back to PATH, avoiding process spawning delays.
- Gates: `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean, `cargo test --workspace` 25 passed, `cargo fmt --check` clean.
- Live Oracle Goliath: 3 VMs resolved instantly, non-blocking UI responsive.

## [2026-11-08] Phase Tier-2 (agent)
- L1 mutating verbs with gate (`--yes` or env), validators, PNG IHDR check for shot.
- L2 19 tools, `disk` + `shot` CLI tops, gate test.
- L3 8 pages + mutate checkbox, `--smoke` ok.
- Gates: clippy clean, 23 tests passed, fmt clean.
- Live Oracle: 3 VMs, snapshot clean-desktop-2026-10-10, VDI list real, mutate-without-yes refused, shot-stopped-VM graceful error, network verb bug (`hostonlynets`→`hostonlyifs`) found + fixed live.
- Next: Tier-3/4 (goldens, NSIS bundle, `run --script` batch for democraft), then `revecraft`.

## [2026-11-08] Phase 2 Tier-1 L1+L2+L3 skeleton (agent)
- L1 `vm.rs` + `import.rs` + `lib.rs` (`Command::dispatch`, `dispatch_future_is_send`).
- L2 CLI + dual-aliased MCP stdio (`vmcraft.*` + `vm_management` etc), 3 L2 tests.
- L3 egui Tier-1 shell (`app.rs` + `main.rs` + `--smoke`), eframe 0.37 `App::ui`.
- Gates: clippy clean, `cargo test --workspace` 18 passed (15 core + 3 mcp), fmt clean.
- Live Oracle Goliath: `status` vbox_present:true, `vm list` 3 VMs, MCP `initialize` ok. Legacy `virtualization-mcp` untouched.
- Next: Phase 2 full verbs (mutating gated), Phase 4 full L3 pages + goldens, then `revecraft`.

## [2026-11-08T02:57:27+01:00] Phase 0 & Phase 1 (Automated `New-RustArtcraftRepo.ps1` Stamp)
- Verified legacy repo `D:\Dev\repos\virtualization-mcp` at `master@c5c9a8d` (kept 100% read-only).
- Copied reusable kernel modules from `D:\Dev\repos\rust-devices-mcp`:
  - `crates/vmcraft-core/src/{bridge.rs, llm.rs (gemma4:12b default), error.rs, creds.rs}`
  - `crates/vmcraft-app/src/{vault.rs, registry.rs}`
- Generated pre-populated `docs/CONVERSION_DOSSIER.md` with 20 discovered `@mcp.tool` definitions and 53 `.env` keys.
- Next step: Complete `docs/CONVERSION_DOSSIER.md` review, then implement Phase 2 (`crates/vmcraft-core`).