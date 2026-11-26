# `vmcraft-mcp` (L2 Dual-Mode CLI & MCP Stdio Server)

Unified headless binary for **vmcraft**. The same executable serves both human/script CLI workflows and the Model Context Protocol (MCP) JSON-RPC 2.0 `stdio` server for AI agents (Cursor, Claude Desktop, Antigravity, OpenCode).

## Stdio Isolation Guarantee

When invoked as `vmcraft-mcp mcp`, `stdout` is strictly reserved for newline-delimited JSON-RPC 2.0 responses. All warnings, subprocess errors, and diagnostics are isolated from `stdout` so MCP clients never suffer framing corruption.

## CLI Subcommands

```powershell
# Hypervisor & Command Bus introspection
vmcraft-mcp status
vmcraft-mcp commands --json
vmcraft-mcp describe iso.fetch
vmcraft-mcp run iso.catalog '{"category":"linux"}'

# VirtualBox & Hyper-V management
vmcraft-mcp vm list
vmcraft-mcp vm info --vm "ubu-2404-lts"
vmcraft-mcp vm start --vm "ubu-2404-lts" --yes
vmcraft-mcp snapshot list --vm "ubu-2404-lts"
vmcraft-mcp snapshot create --vm "ubu-2404-lts" --snap "clean-base" --yes
vmcraft-mcp hyperv list

# ISO Library, Streaming Fetcher & Optical Attach
vmcraft-mcp iso catalog --category all       # all | linux | uefi | windows
vmcraft-mcp iso list                         # scans ~/.vmcraft/isos or VMCRAFT_ISO_DIR
vmcraft-mcp iso fetch --id alpine-3.21-virt --url-only
vmcraft-mcp iso fetch --id uefi-shell-2.2-release --yes
vmcraft-mcp iso verify --path "$env:USERPROFILE\.vmcraft\isos\UEFI-Shell-2.2-24H2-RELEASE.iso"
vmcraft-mcp iso attach --vm "ubu-2404-lts" --iso "$env:USERPROFILE\.vmcraft\isos\UEFI-Shell-2.2-24H2-RELEASE.iso" --yes

# Clean-room detonation, guest control & Firecracker KVM
vmcraft-mcp airgap --vm "win11" --disconnect
vmcraft-mcp guest run --vm "ubu-2404-lts" --user user --pass user --exe /bin/bash --args=-lc --args "uname -a"
vmcraft-mcp pipeline naked-test --repo "sandraschi/vmcraft" --baseline "ubu-2404-lts" --user user --pass user --linux --yes
vmcraft-mcp firecracker probe
```

## MCP Registration

```json
{
  "mcpServers": {
    "vmcraft": {
      "command": "D:/Dev/repos/vmcraft/target/release/vmcraft-mcp.exe",
      "args": ["mcp"]
    }
  }
}
```

## Environment Variables

- `VMCRAFT_ALLOW_MUTATE=1` — Globally allows mutating operations without passing `--yes` per call.
- `VMCRAFT_ISO_DIR` — Overrides the default `~/.vmcraft/isos` local ISO library path.
- `VMCRAFT_FIDO_SCRIPT` — Path to an optional external `Fido.ps1` script for Windows 10/11 ISO URL resolution.
- `VBOXMANAGE_PATH` — Explicit path to `VBoxManage.exe` if not in the default Oracle VirtualBox install directory or `PATH`.

## Verification

```powershell
cargo test -p vmcraft-mcp
cargo clippy -p vmcraft-mcp --all-targets --all-features -- -D warnings
```
