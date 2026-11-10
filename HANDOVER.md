# HANDOVER -- `vmcraft`

**Last Updated:** 2026-11-10T22:45:00+01:00  
**Current Phase:** Tier-4 Complete (v1.0.0) -> Clean-Room Detonation Pipeline + Hardware Airgap Isolation + Naked Install Verification + MCP stdio Green

## What Was Done
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
