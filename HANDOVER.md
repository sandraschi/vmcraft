# HANDOVER -- `vmcraft`

**Last Updated:** 2026-11-10T10:52:45+01:00  
**Current Phase:** Tier-2 Complete (v0.2.1) -> L3 Desktop Workstation + MCP stdio Green

## What Was Done
- **L1 (`vmcraft-core`)**:
  - Implemented command dispatch bus with 19 domain operations: `status`, `vm_list`, `vm_info`, `vm_create`, `vm_start`, `vm_stop`, `vm_delete`, `snapshot_list`, `snapshot_create`, `snapshot_restore`, `snapshot_delete`, `network_list`, `network_create`, `network_remove`, `disk_list`, `disk_create`, `vm_shot`, `host_info`, `sandbox_run`.
  - Mutating verbs safely gated behind explicit confirmation (`--yes` flag or `VMCRAFT_ALLOW_MUTATE=1`).
  - Added Windows VirtualBox path pre-check in `vboxmanage_candidates` to directly discover `C:\Program Files\Oracle\VirtualBox\VBoxManage.exe` without subprocess spawn latency.
  - Zero unwrap/panic discipline (`-D warnings` enforced across workspace).
- **L2 (`vmcraft-mcp`)**:
  - Pure JSON-RPC stdio implementation on stdout, operational logs on stderr.
  - 19 tools with dual aliases (`vmcraft.*` + legacy portmanteaus e.g. `vm_management`, `snapshot_management`, etc.).
  - Human CLI tops for all domain commands (`status`, `vm`, `snapshot`, `network`, `disk`, `shot`, `sandbox`, `host`).
- **L3 (`vmcraft-app`)**:
  - Pure Rust eframe/egui workstation with 8 tabs: Dashboard, VMs, Snapshots, Network, Disks, Shot, Sandbox, Help.
  - Decoupled asynchronous architecture using `tokio::runtime` and `mpsc::channel`, eliminating UI thread blocking.
  - Automatic background discovery on startup: instantly polls status, VM list, and host templates upon launch.
  - Interactive live VM table on Dashboard with quick-action controls (`Select`, `Info`, `Start`, `Stop`).
  - Quick VM target selector on the VMs management page.
- **Verification**:
  - Live Oracle verified on Goliath: 3 real VMs (`win11-automated`, `ubu-2404-lts`, `batocera-clean`), VirtualBox 7.x detected.
  - All 25 workspace tests green (`cargo test --workspace`).
  - Clippy check clean (`cargo clippy --workspace --all-targets --all-features -- -D warnings`).
  - Formatter clean (`cargo fmt --check`).
  - Smoke binary passes in both debug and release profiles.

## Next Concrete Actions
1. Add optional QEMU/KVM provider alongside VirtualBox and Hyper-V if requested.
2. Build NSIS installer bundle via `installer/installer.nsi`.
