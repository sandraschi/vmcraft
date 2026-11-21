# vmcraft — Pure-Rust ArtCraft VM workstation (Bucket A)

**Status:** Tier-4 complete (v1.0.0 stamped 2026-11-10), multi-hypervisor (VirtualBox + Hyper-V), ArtCraft command bus (28 verbs), clean-room airgap detonation pipeline & consumer naked install flows.
**Legacy (READ-ONLY):** `D:\Dev\repos\virtualization-mcp` @ `master@c5c9a8d`.
**Target:** `D:\Dev\repos\vmcraft` — `vmcraft-core` (L1) + `vmcraft-mcp` (L2 CLI+stdio) + `vmcraft-app` (L3 egui).

## Rules (ArtCraft containment)

- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — zero warnings, `unwrap_used`/`expect_used`/`panic`/`todo`/`unimplemented` deny.
- `cargo test --workspace` green, `cargo fmt --all --check` clean.
- MCP stdio: `stdout` is 100% JSON-RPC, logs on `stderr` only.
- Mutating verbs (`vm create|start|stop|delete`, `snapshot create|restore|delete`, `network create|remove`, `airgap.set`, `guest.exec`, `pipeline.detonate`, `pipeline.naked_install`) require `--yes` or `VMCRAFT_ALLOW_MUTATE=1`. Default CI runs read-only + `sandbox run` only.
- Hyper-V absent / VBox absent: return `status: down` receipt, never fake green, never fail CI.
- Legacy repo is 100% read-only. No edits, no stops, no DB locks.

## Layout

- `crates/vmcraft-core` — VBoxManage + Hyper-V + guest control + airgap + detonation pipeline + sandbox protocols, `Command::dispatch`, ArtCraft command bus registry (`commands_catalog`, `describe_command`, `parse_command`), `import.rs` zero-config import.
- `crates/vmcraft-mcp` — `vmcraft-mcp` binary: human CLI + `mcp` stdio, ArtCraft command bus (`commands`, `describe`, `run`, `batch`), pipeline (`detonate`, `naked-test`), airgap, guest control, dual-aliased tools (`vmcraft.*` + legacy `vm_management` etc).
- `crates/vmcraft-app` — `vmcraft-app` egui desktop with 10 tabs: Dashboard, VMs, Snapshots, Network, Disks, Shot (framebuffer viewer), Sandbox, Commands (bus browser), Detonate (air-gapped reverse engineering studio), Help.

## Fleet context

Before any task, read `D:\Dev\repos\mcp-central-docs\standards\AGENT_READING_ORDER.md`.
Never invent dates/locations. Max 10 files per commit.


## Documentation preservation and hooks

On 2026-11-20 a README "standardization" sweep across the fleet deleted project descriptions,
status sections and user warnings in nine repos. Full rules:
`mcp-central-docs/standards/CONTENT_PRESERVATION_STANDARD.md`.

- Applying any standard or template restructures; it never deletes. Move content (within the file
  or to `docs/` with a link), do not drop it. Never remove what the project is, what it is based on,
  its status (what works, what does not, what is not built) or warnings to users.
- Remove content only when Sandra explicitly asks for that removal, in this repo.
- Never add claims you have not verified (release downloads, versions, test counts, features).
- Only standards with `status: active` may be applied across repos. A change across several repos is
  done on one repo first and approved by Sandra; its commits name the tool and carry a `Sweep-Id:` line.
- `.githooks/commit-msg` blocks a commit that deletes a Markdown file, removes 20% of its lines
  (15 or more) or removes 3 or more headings. A removal Sandra asked for passes with
  `Docs-Removal-Approved: <who and why>` in the message. Enable once per clone:
  `git config core.hooksPath .githooks`. Never use `--no-verify`; if a hook blocks you, stop and ask.
