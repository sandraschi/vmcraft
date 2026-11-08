# Changelog — vmcraft

## [2026-11-08] v0.1.0 Tier-1 skeleton
- Workspace stamp (Phase 0/1): kernel from `rust-devices-mcp`, `vmcraft-core/mcp/app` crates.
- L1 `Command::dispatch`: status, vm list/info, snapshot list, network list, host info, sandbox run (python/pwsh, 30s timeout).
- L2 `vmcraft-mcp`: CLI `--json` + `mcp` stdio (pure JSON-RPC), dual aliases (`vmcraft.*` + legacy `vm_management`/`snapshot_management`/`network_management`/`system_management`/`sandbox_management`).
- L3 `vmcraft-app`: Tier-1 egui shell (status, vm list/info, host info, sandbox runner), `--smoke` ok.
- Gates: `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean, `cargo test --workspace` 18 passed, `cargo fmt --check` clean.
- Live Oracle 2026-11-08 Goliath: `vbox_present:true`, `vm list` 3 VMs (win11-automated, ubu-2404-lts, batocera-clean), MCP `initialize` ok.
