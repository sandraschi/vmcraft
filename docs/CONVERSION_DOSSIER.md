# Conversion Dossier: `rust-vmcraft-mcp` (from `vmcraft-mcp`)

**Generated:** 2026-11-08T02:57:27+01:00  
**Legacy Repo Path (100% READ-ONLY):** `D:\Dev\repos\virtualization-mcp`  
**Legacy Git Baseline:** branch `master` @ commit `c5c9a8d`  
**Target Rust Repo:** `D:\Dev\repos\vmcraft`

---

## 1. Bucket & Privilege-Split / Hidden Binary Audit
- **Subdirectories / Native Binaries Detected:** native, src-tauri, Cargo.toml (root)
- **Triage Bucket:** *(Agent: confirm Bucket A / B / C1 / D before writing domain code)*

## 2. Live Service, Port & Database Contention Audit
- **Legacy Running Service / Port Check:** *(Agent: verify if legacy server or shared SQLite DB is currently open)*
- **Contention Strategy:** Shared SQLite databases opened with `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX` + `busy_timeout(5000 ms)`. Legacy repo remains 100% untouched.

## 3. Complete 1:1 MCP Tool & Portmanteau Mapping Table

| Legacy Python Tool (`@mcp.tool`) | Rust `Command` Variant | CLI Subcommand | Rust MCP Tool + Legacy Alias |
|---|---|---|---|
| `virtualization_shutdown` | `Vmcraft::...` | `virtualization_shutdown` | `vmcraft.virtualization_shutdown` + alias `virtualization_shutdown` |
| `list_hyperv_vms` | `Vmcraft::...` | `list_hyperv_vms` | `vmcraft.list_hyperv_vms` + alias `list_hyperv_vms` |
| `get_hyperv_vm` | `Vmcraft::...` | `get_hyperv_vm` | `vmcraft.get_hyperv_vm` + alias `get_hyperv_vm` |
| `start_hyperv_vm` | `Vmcraft::...` | `start_hyperv_vm` | `vmcraft.start_hyperv_vm` + alias `start_hyperv_vm` |
| `stop_hyperv_vm` | `Vmcraft::...` | `stop_hyperv_vm` | `vmcraft.stop_hyperv_vm` + alias `stop_hyperv_vm` |
| `create_sandbox` | `Vmcraft::...` | `create_sandbox` | `vmcraft.create_sandbox` + alias `create_sandbox` |
| `list_sandboxes` | `Vmcraft::...` | `list_sandboxes` | `vmcraft.list_sandboxes` + alias `list_sandboxes` |
| `list_portfolios` | `Vmcraft::...` | `list_portfolios` | `vmcraft.list_portfolios` + alias `list_portfolios` |
| `list_common_programs` | `Vmcraft::...` | `list_common_programs` | `vmcraft.list_common_programs` + alias `list_common_programs` |
| `stop_sandbox` | `Vmcraft::...` | `stop_sandbox` | `vmcraft.stop_sandbox` + alias `stop_sandbox` |
| `vm_agentic_workflow` | `Vmcraft::...` | `vm_agentic_workflow` | `vmcraft.vm_agentic_workflow` + alias `vm_agentic_workflow` |
| `info_tools` | `Vmcraft::...` | `info_tools` | `vmcraft.info_tools` + alias `info_tools` |
| `hyperv_management` | `Vmcraft::...` | `hyperv_management` | `vmcraft.hyperv_management` + alias `hyperv_management` |
| `libvirt_management` | `Vmcraft::...` | `libvirt_management` | `vmcraft.libvirt_management` + alias `libvirt_management` |
| `network_management` | `Vmcraft::...` | `network_management` | `vmcraft.network_management` + alias `network_management` |
| `snapshot_management` | `Vmcraft::...` | `snapshot_management` | `vmcraft.snapshot_management` + alias `snapshot_management` |
| `storage_management` | `Vmcraft::...` | `storage_management` | `vmcraft.storage_management` + alias `storage_management` |
| `system_management` | `Vmcraft::...` | `system_management` | `vmcraft.system_management` + alias `system_management` |
| `vm_management` | `Vmcraft::...` | `vm_management` | `vmcraft.vm_management` + alias `vm_management` |
| `vmware_management` | `Vmcraft::...` | `vmware_management` | `vmcraft.vmware_management` + alias `vmware_management` |

## 4. Credential, `.env`, Config & Data-Path Inventory (`import.rs` Spec)
Discovered environment keys in `D:\Dev\repos\virtualization-mcp` (values redacted):
- `VBOXMANAGE_PATH`
- `VIRTUALBOX_HOME`
- `MCP_SERVER_NAME`
- `MCP_SERVER_VERSION`
- `DEBUG`
- `LOG_LEVEL`
- `LOG_FILE`
- `LOG_ROTATION_SIZE`
- `LOG_BACKUP_COUNT`
- `TEMPLATES_FILE`
- `DEFAULT_TEMPLATE`
- `AUTO_RELOAD_TEMPLATES`
- `DEFAULT_MEMORY_MB`
- `DEFAULT_DISK_GB`
- `DEFAULT_NETWORK`
- `DEFAULT_OS_TYPE`
- `PORT_FORWARDING_START`
- `PORT_FORWARDING_END`
- `ENABLE_SSH_FORWARDING`
- `DEFAULT_SSH_PORT`
- `MAX_SNAPSHOTS_PER_VM`
- `AUTO_CLEANUP_SNAPSHOTS`
- `SNAPSHOT_PREFIX`
- `MAX_CONCURRENT_OPERATIONS`
- `VM_INFO_CACHE_TTL`
- `TEMPLATE_CACHE_TTL`
- `REQUIRE_DELETE_CONFIRMATION`
- `ALLOW_FORCE_OPERATIONS`
- `BLOCK_PRIVILEGED_PORTS`
- `MIN_ALLOWED_PORT`
- `ENABLE_MOCK_MODE`
- `AUTO_CLEANUP_TEST_VMS`
- `TEST_VM_PREFIX`
- `TEST_VM_MAX_AGE_HOURS`
- `PREFER_LINKED_CLONES`
- `ENABLE_PARALLEL_CREATION`
- `PROVIDE_SOLUTION_SUGGESTIONS`
- `AUSTRIAN_DIRECTNESS`
- `ENABLE_RICH_RESPONSES`
- `ENABLE_METRICS_EXPORT`
- `PROMETHEUS_PORT`
- `CHECK_HOST_RESOURCES`
- `MIN_FREE_MEMORY_MB`
- `MIN_FREE_DISK_GB`
- `MAX_FILE_SIZE_MB`
- `ALLOWED_EXTENSIONS`
- `COMMAND_TIMEOUT`
- `VM_START_TIMEOUT`
- `VM_STOP_TIMEOUT`
- `SNAPSHOT_TIMEOUT`
- `CUSTOM_TEMPLATES_PATH`
- `CUSTOM_LOGS_PATH`
- `CUSTOM_CONFIG_PATH`

## 5. Legacy Python Source Files Discovered (`src/`)
- `src/virtualization_mcp/__init__.py (1655 bytes)`
- `src/virtualization_mcp/__main__.py (1277 bytes)`
- `src/virtualization_mcp/all_tools_server.py (6404 bytes)`
- `src/virtualization_mcp/async_wrapper.py (5780 bytes)`
- `src/virtualization_mcp/config.py (10167 bytes)`
- `src/virtualization_mcp/dual_server.py (3045 bytes)`
- `src/virtualization_mcp/exceptions.py (3544 bytes)`
- `src/virtualization_mcp/json_encoder.py (7047 bytes)`
- `src/virtualization_mcp/main.py (3450 bytes)`
- `src/virtualization_mcp/mcp_tools.py (8449 bytes)`
- `src/virtualization_mcp/settings.py (576 bytes)`
- `src/virtualization_mcp/vbox_compat.py (12817 bytes)`
- `src/virtualization_mcp/api/__init__.py (6524 bytes)`
- `src/virtualization_mcp/api/documentation.py (25328 bytes)`
- `src/virtualization_mcp/chat/__init__.py (193 bytes)`
- `src/virtualization_mcp/chat/memory.py (2110 bytes)`
- `src/virtualization_mcp/chat/service.py (14756 bytes)`
- `src/virtualization_mcp/models/__init__.py (70 bytes)`
- `src/virtualization_mcp/plugins/__init__.py (3778 bytes)`
- `src/virtualization_mcp/plugins/hyperv/__init__.py (198 bytes)`

## 6. Write-Safety & Destructive-Action Fence Design
- All mutating commands (`vm create|start|stop|delete`, `snapshot create|restore|delete`, `network create|remove`, `disk create`, `airgap`, `guest.exec`, `pipeline.detonate`, `pipeline.naked_install`) require explicit confirmation flag `--yes` in CLI, `confirm: true` in MCP JSON-RPC, or `VMCRAFT_ALLOW_MUTATE=1` in the environment.
- Read-only operations (`status`, `vm list`, `vm info`, `snapshot list`, `network list`, `disk list`, `shot`, `host info`, `commands`, `describe`) execute without prompt.
- Clean-room detonation pipeline enforces an unconditional pre-detonation snapshot `craft-detonate-<ts>` and rolls back via RAII, guaranteeing zero residual footprint on the guest VM even if payloads crash or fail.

## 7. L3 `egui` Desktop Pages
10 Domain Tabs implemented in `crates/vmcraft-app/src/app.rs`:
- `Dashboard`: Multi-hypervisor VM overview table (VirtualBox & Hyper-V) with quick actions and operational log.
- `VMs`: Detailed target selector, hardware configuration form, and state controls.
- `Snapshots`: Structured point-in-time snapshot tree with restore/delete actions.
- `Network`: VirtualBox host-only network management.
- `Disks`: Virtual disk image (`.vdi`) inspection and dynamic provisioning.
- `Shot`: Framebuffer screenshot capture with live in-app PNG decoding.
- `Sandbox`: Isolated 30-second script runner for Python and PowerShell.
- `Commands`: Interactive ArtCraft Command Bus catalog (28 verbs) and test dispatcher.
- `Detonate`: Clean-room air-gapped detonation & disassembly studio with emergency nuke/rollback.
- `Help`: Safety gates and reference documentation.