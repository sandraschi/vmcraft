# HANDOVER -- `vmcraft`

**Last Updated:** 2026-11-24T18:07:00+01:00  
**Current Phase:** `v0.1.0` published on GitHub (`origin/master` @ post-v0.1.0 UI & live-state polish), 54 workspace tests passing, GitHub Actions CI green on Windows and Linux.

## Recent Updates (2026-11-22 to 2026-11-24)
- **Release & CI**: Tagged and published `v0.1.0` via `scripts/release.ps1 -Publish`; added `.github/workflows/ci.yml` (`windows-latest` + `ubuntu-latest`).
- **Core (`vmcraft-core`)**: Wired `pipeline_naked_install` to real VirtualBox snapshot restore + headless start + `guestcontrol` execution + rollback; fixed `sandbox_run` (`python`/`py`/`python3` fallback + `powershell.exe -File`); added live `state` (`running`/`off`) to `vm_list` via `VBoxManage list runningvms`.
- **Desktop UI (`vmcraft-app`)**: Added standard Craft menu bar (`switch_on_hover`), `View -> Show Tab Bar` (`Ctrl+B`), `View -> Status Bar` (`Bottom`/`Top`/`Off`), `Ctrl+K` Command Palette, `F1` Shortcuts, 7-tab Help Center + `HELP.md`, theme-aware Dark/Light `Palette`, Dashboard KPI cards + VM search/filter + `Recent Activity` feed, human-readable `VMs`/`Network`/`Disks`/`Shot`/`Swarm` views, and in-GUI Guest Runner & Clean-Room Linux Craft Test card.
- **Linux Test VM**: Reset `ubu-2404-lts` (`user`/`user`) via GRUB recovery, verified `vmcraft-mcp guest exec` end-to-end, and saved clean baseline snapshot `clean-desktop-2026-11-22`.

## Earlier Milestone (2026-11-10)
- **L1 (`vmcraft-core`)**:
  - Implemented Airgap Isolation (`airgap_set`): Dynamically cuts physical/virtual link cables across adapters 1..4 (`setlinkstateN off`), disables shared clipboard (`clipboard mode disabled`), and blocks drag-and-drop (`draganddrop disabled`).
  - Added Guest Control API: `guest_exec`, `guest_copyto`, and `guest_copyfrom` using native `VBoxManage guestcontrol` without relying on network shares.
  - Implemented Reverse-Engineering Clean-Room Detonation Workflow (`pipeline_detonate`):
    - Addresses operational security risk of poisoned/tainted patchers and keygens with delayed payloads (e.g. IDA Pro keygen).
    - Hard air-gap disconnects network *before* running keygen.
    - Copies disassembly target binary *into* the guest.
    - Runs disassembler *inside* the isolated guest on the target binary.
    - Extracts only non-executable disassembly output (`.asm` / text artifacts) back to the host.
    - Unconditionally powers off and nukes the VM back to pre-detonation snapshot `craft-detonate-<ts>`.
  - Implemented Naked Install Testing (`pipeline_naked_install`) to verify consumer installer experience on clean baseline VMs.
  - Expanded ArtCraft Command Bus to 28 operations.
  - Zero unwrap/panic discipline maintained across workspace.
- **L2 (`vmcraft-mcp`)**:
  - Added CLI subcommands: `pipeline detonate`, `pipeline naked-test`, `airgap`, `guest run`, `guest copyto`, `guest copyfrom`.
  - Added MCP tools: `vmcraft.pipeline_detonate`, `vmcraft.pipeline_naked_install`, `vmcraft.airgap_set`, `vmcraft.guest_exec`, `vmcraft.guest_copyto`, `vmcraft.guest_copyfrom`.
  - Enforced strict confirmation gate (`--yes` or `VMCRAFT_ALLOW_MUTATE=1`) on all mutating operations.
- **L3 (`vmcraft-app`)**:
  - Added 10th tab **Detonate**: Clean-room detonation interface for untrusted patchers/keygens with input fields for guest credentials, keygen path, target binary, disassembler executable, and output paths.
  - Action buttons: "💥 Detonate & Disassemble in Airgap", "✂ Sever Network Link", "🔌 Reconnect Network", and "🚨 Emergency Nuke & Rollback".
- **Verification**:
  - 30 unit/integration tests passing (`cargo test --workspace`).
  - Clippy check clean (`cargo clippy --workspace --all-targets --all-features -- -D warnings`).
  - Formatter clean (`cargo fmt --check`).
  - Desktop smoke test passed (`cargo run -p vmcraft-app -- --smoke`).


## Next Concrete Actions
1. Add optional QEMU/KVM provider alongside VirtualBox and Hyper-V if requested.
2. Add live control channel port `--control PORT` bridge if live GUI attach is required.
