# Changelog — vmcraft

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
