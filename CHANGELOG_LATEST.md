## 0.1.0 - 2026-11-22

First tagged pre-1.0 release of `vmcraft` (L1 `vmcraft-core`, L2 `vmcraft-mcp` CLI + stdio MCP, L3 `vmcraft-app` egui desktop workstation).

### Highlights
- **Multi-Hypervisor Management**: VirtualBox 7+ lifecycle (`vm_list`, `vm_info`, `vm_create`, `vm_start`, `vm_stop`, `vm_delete`), Hyper-V discovery and start/stop, snapshots, host-only networks, `.vdi` disks, and live framebuffer PNG capture (`vm_shot`).
- **Standard Craft Desktop UI**: Top menu bar (`Session`, `Edit`, `View`, `Command`, `Window`, `Help`) with `switch_on_hover`, hideable tab bar (`View -> Show Tab Bar` / `Ctrl+B`), switchable status bar (`View -> Status Bar -> Bottom/Top/Off`), `Ctrl+K` Command Palette, `F1` Keyboard Shortcuts, and 7-tab Help Center (`HELP.md`).
- **Human-Readable Views**: Structured tables and detail grids for VMs, VM Info, Networks, Disks, and Firecracker processes, with collapsible raw JSON output.
- **Clean-Room & Sandbox Pipelines**: Airgap isolation (`airgap_set`), VirtualBox `guestcontrol` (`guest_exec`, `guest_copyto`, `guest_copyfrom`), `pipeline.detonate`, `pipeline.naked_install`, WSL2 Firecracker KVM probing/management, and fixed Python/PowerShell local sandbox (`sandbox_run`).
- **Verification**: 54 workspace tests passing, `cargo clippy -- -D warnings` clean (zero `unwrap`/`expect`/`panic`), `cargo fmt --check` clean, GitHub Actions CI green on Windows and Linux.
