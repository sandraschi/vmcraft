# Conversion Audit Log: `rust-vmcraft-mcp`

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