# `vmcraft-core` (L1 Domain Library)

Pure-Rust, UI-agnostic, CLI-agnostic domain engine for **vmcraft**. Both the L2 CLI/MCP binary (`vmcraft-mcp`) and the L3 desktop workstation (`vmcraft-app`) execute all operations through this single crate so behavior never diverges.

## Design Guarantees

- **Single Command Bus**: Every capability is modeled as a variant of `vmcraft_core::Command`, cataloged in `commands_catalog()` (37 operations), parsed via `parse_command(id, params)`, and executed by `dispatch(Command) -> serde_json::Value`.
- **Zero-Panic / Zero-Unwrap**: Inherits workspace Clippy denies for `unwrap_used`, `expect_used`, `panic`, `todo`, and `unimplemented`. Absent hypervisors (`VBoxManage`, Hyper-V, WSL2 Firecracker) or missing optional scripts (`Fido.ps1`) return structured `{"status": "down" | "error", "reason": "..."}` JSON receipts.
- **Explicit Mutation Gate**: Every state-changing operation checks `confirm || VMCRAFT_ALLOW_MUTATE=1` before running.
- **MIT OR Apache-2.0 Clean Boundary**: Never embeds GPLv3 code. Windows ISO URL resolution invokes an optional external `Fido.ps1` subprocess if provided by the user.

## Modules

| Module | Responsibility |
|---|---|
| `lib.rs` | `Command` enum, `dispatch()`, `commands_catalog()`, `describe_command()`, `parse_command()`. |
| `vm.rs` | `VBoxManage` discovery & execution, VM lifecycle, snapshots, host-only networks, `.vdi` disks, screenshots, Hyper-V CIM probe/control, and 30s ephemeral sandbox runner. |
| `iso.rs` | Curated Linux + UEFI Shell + external-Fido Windows ISO catalog, `~/.vmcraft/isos` library scanner, 1 MiB streaming SHA-256 verifier, HTTPS-only streaming fetcher (`.part` atomic rename + `.sha256` sidecar + 20h Fido URL cache), and VirtualBox optical-drive attach (`IDE`/`SATA` auto-detect). |
| `pipeline.rs` | `airgap_set` (cuts link states + clipboard/DnD), `guest_exec`, `guest_copyto`, `guest_copyfrom`, `pipeline_detonate` (airgapped clean-room disassembly + unconditional rollback), and `pipeline_naked_install`. |
| `firecracker.rs` | WSL2 `/dev/kvm` probe, active Firecracker process/socket listing, and micro-VM spawn/kill. |
| `error.rs` | Typed `DeviceError` variants and input fences. |
| `creds.rs` | Timeout wrapper (`with_timeout_secs`) and credential source resolution. |
| `bridge.rs` / `import.rs` / `llm.rs` | Event bridge, zero-config env import, and local/cloud LLM client helpers. |

## Verification

```powershell
cargo test -p vmcraft-core
cargo clippy -p vmcraft-core --all-targets --all-features -- -D warnings
```
