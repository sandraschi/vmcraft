# HANDOVER -- `vmcraft`

**Last Updated:** 2026-11-10T12:20:00+01:00  
**Current Phase:** Tier-3 Complete (v0.3.0) -> ArtCraft Command Bus + Multi-Hypervisor Desktop Workstation + MCP stdio Green

## What Was Done
- **L1 (`vmcraft-core`)**:
  - Implemented ArtCraft Command Bus with 22 operations per `mcp-central-docs/standards/rust-artcraft/CRAFT_COMMAND_BUS.md`: `commands_catalog()`, `describe_command(id)`, `parse_command(id, params)`, and `dispatch(cmd)`.
  - Added native Microsoft Hyper-V integration (`hyperv_probe`, `hyperv_list`, `hyperv_start`, `hyperv_stop`) using CIM query fallback.
  - Multi-hypervisor `vm_list()` combines VirtualBox and Hyper-V instances with explicit `provider` attribution.
  - Structured snapshot parsing into `{name, uuid, description}` objects.
  - Mutating verbs safely gated behind explicit confirmation (`--yes` flag or `VMCRAFT_ALLOW_MUTATE=1`).
  - Added Windows VirtualBox path pre-check in `vboxmanage_candidates` to directly discover `C:\Program Files\Oracle\VirtualBox\VBoxManage.exe`.
  - Zero unwrap/panic discipline (`-D warnings` enforced across workspace).
- **L2 (`vmcraft-mcp`)**:
  - Pure JSON-RPC stdio implementation on stdout, operational logs on stderr.
  - Full CLI command bus: `commands --json`, `describe <cmd>`, `run --cmd <id> [--params <json>]`, `batch --script <file>`.
  - Native CLI subcommands for Hyper-V: `hyperv list`, `hyperv start`, `hyperv stop`.
  - MCP tools: `vmcraft.commands`, `vmcraft.describe`, `vmcraft.run`, `vmcraft.hyperv_list`, `vmcraft.hyperv_start`, `vmcraft.hyperv_stop`, plus legacy compatibility aliases.
- **L3 (`vmcraft-app`)**:
  - Pure Rust eframe/egui workstation with 9 tabs: Dashboard, VMs, Snapshots, Network, Disks, Shot, Sandbox, Commands, Help.
  - Commands tab: Interactive ArtCraft Command Bus registry explorer and test dispatcher.
  - Multi-hypervisor support in Dashboard: Provider badges (`VirtualBox` in amber, `Hyper-V` in cyan) and provider-specific actions.
  - Snapshots tab: Structured snapshot tree table with instant `Restore` and `Delete` actions.
  - Shot tab: In-app framebuffer viewer using `image` decoding to render live guest screenshots directly inside the GUI.
  - Decoupled asynchronous architecture using `tokio::runtime` and `mpsc::channel`, eliminating UI thread blocking.
  - Startup polling: automatically detects both VirtualBox and Hyper-V hypervisors on launch.
- **Cleanups**:
  - Removed obsolete installer NSIS files (vmcraft is a pure ArtCraft application, not an installer-bundled utility).
- **Verification**:
  - Live Oracle + Hyper-V verified on Goliath: 3 VirtualBox VMs (`win11-automated`, `ubu-2404-lts`, `batocera-clean`) and 1 Hyper-V VM (`podman-machine-default`).
  - All 27 workspace tests green (`cargo test --workspace`).
  - Clippy check clean (`cargo clippy --workspace --all-targets --all-features -- -D warnings`).
  - Formatter clean (`cargo fmt --check`).
  - Smoke binary passes in both debug and release profiles.

## Next Concrete Actions
1. Add optional QEMU/KVM provider alongside VirtualBox and Hyper-V if requested.
2. Add live control channel port `--control PORT` bridge if live GUI attach is required.
