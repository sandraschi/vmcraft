# `vmcraft-app` (L3 Native egui Desktop Workstation)

Native Windows/cross-platform desktop workstation built on `eframe`/`egui` (`0.37`), calling `vmcraft_core::dispatch` in-process over a non-blocking Tokio runtime + `mpsc::channel` message bus so hypervisor probes and ISO downloads never freeze the UI.

## 12 Workstation Tabs

| Tab | Purpose |
|---|---|
| **Dashboard** | 4 KPI cards, live VM search & filter chips (`All`, `Running`, `VirtualBox`, `Hyper-V`), one-click VM selection, and `Recent Activity` feed. |
| **VMs** | VM provisioning form, live state table, parsed `Selected VM Details` grid, and in-GUI Guest Runner & Clean-Room Linux Craft Test card. |
| **Snapshots** | Point-in-time snapshot listing with create, restore, and delete controls. |
| **Network** | VirtualBox host-only network interface table and provisioning. |
| **Disks** | `.vdi` hard disk inspection (`State`, `Format`, `Capacity`, `Location`, `UUID`) and creation. |
| **ISOs** | Curated Linux (Alpine, Ubuntu, Debian, Fedora, Arch), UEFI Shell 2.2, and external-Fido Windows ISO catalog with category filters, search, custom HTTPS download, SHA-256 verification, and one-click attach to the selected VM. |
| **Shot** | Live VirtualBox framebuffer PNG capture rendered directly inside the app. |
| **Detonate** | Clean-room airgapped execution & disassembly pipeline with emergency network sever and rollback. |
| **Swarm** | WSL2 `/dev/kvm` Firecracker micro-VM probe, running instance list, spawn, and kill. |
| **Sandbox** | Isolated 30-second local Python / PowerShell runner with formatted output. |
| **Commands** | Interactive 37-verb ArtCraft Command Bus explorer and JSON dispatcher. |
| **Help** | 7-tab built-in Help Center (`Get Started`, `Functions`, `Usage`, `Tips & Tricks`, `Pitfalls & Safety`, `Planned Extensions`, `Other Crafts`). |

## Modules

- `src/main.rs` — Entry point, `--smoke` headless verification gate, native viewport config.
- `src/app.rs` — `VmApp` state machine, Craft standard menu bar (`Session`, `Edit`, `View`, `Command`, `Window`, `Help`), Dark/Light `Palette`, Command Palette (`Ctrl+K`), Keyboard Shortcuts (`F1`), and page routing.
- `src/iso_ui.rs` — Modular **ISOs** tab (`IsoUiState`, catalog cards, local library grid, Fido/custom URL bar).
- `src/help_ui.rs` — Modular 7-tab **Help Center** with detailed human-readable operator guides and theme-aware high-contrast styling.

## Shortcuts & Running

- `Ctrl+K` / `Ctrl+Shift+P` — Command Palette
- `Ctrl+B` — Toggle Tab Bar
- `F1` — Keyboard Shortcuts
- `F5` — Refresh hypervisors & local ISO library
- `Ctrl+MouseWheel` / `Ctrl+=` / `Ctrl+-` / `Ctrl+1` — Zoom in / out / reset 100%

```powershell
cargo run -p vmcraft-app
cargo run -p vmcraft-app -- --smoke
```
