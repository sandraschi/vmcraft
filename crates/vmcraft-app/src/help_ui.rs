//! Modular L3 egui 7-tab Help Center with detailed human-readable operator guides
//! and Dark/Light high-contrast styling.

use eframe::egui::{self, RichText};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    GetStarted,
    Functions,
    Usage,
    Tips,
    Safety,
    Planned,
    OtherCrafts,
}

impl HelpTab {
    pub fn label(self) -> &'static str {
        match self {
            HelpTab::GetStarted => "Get Started",
            HelpTab::Functions => "Functions",
            HelpTab::Usage => "Usage (GUI, CLI & MCP)",
            HelpTab::Tips => "Tips & Tricks",
            HelpTab::Safety => "Pitfalls & Safety",
            HelpTab::Planned => "Planned Extensions",
            HelpTab::OtherCrafts => "Other Crafts",
        }
    }
}

pub struct HelpThemeColors {
    pub card_bg: egui::Color32,
    pub card_border: egui::Color32,
    pub text_primary: egui::Color32,
    pub text_muted: egui::Color32,
    pub accent: egui::Color32,
    pub tab_active_bg: egui::Color32,
    pub warn_fg: egui::Color32,
}

fn section_card(
    ui: &mut egui::Ui,
    colors: &HelpThemeColors,
    title: &str,
    body: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::NONE
        .fill(colors.card_bg)
        .stroke(egui::Stroke::new(1.0, colors.card_border))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).strong().color(colors.accent));
            ui.add_space(4.0);
            body(ui);
        });
    ui.add_space(6.0);
}

fn bullet(ui: &mut egui::Ui, colors: &HelpThemeColors, lead: &str, text: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(format!("• {lead}:"))
                .strong()
                .color(colors.text_primary),
        );
        ui.label(RichText::new(text).color(colors.text_primary));
    });
}

pub fn render_help_page(ui: &mut egui::Ui, active_tab: &mut HelpTab, colors: &HelpThemeColors) {
    ui.heading("vmcraft Help Center");
    ui.label(
        RichText::new(
            "Operator manual for VirtualBox, Hyper-V, Linux/UEFI/Fido ISO provisioning, Clean-Room Detonation, and Firecracker KVM.",
        )
        .small()
        .color(colors.text_muted),
    );
    ui.add_space(6.0);

    ui.horizontal_wrapped(|ui| {
        for tab in [
            HelpTab::GetStarted,
            HelpTab::Functions,
            HelpTab::Usage,
            HelpTab::Tips,
            HelpTab::Safety,
            HelpTab::Planned,
            HelpTab::OtherCrafts,
        ] {
            let selected = *active_tab == tab;
            let btn = if selected {
                egui::Button::new(RichText::new(tab.label()).strong().color(colors.accent))
                    .fill(colors.tab_active_bg)
                    .stroke(egui::Stroke::new(1.0, colors.accent))
            } else {
                egui::Button::new(RichText::new(tab.label()).color(colors.text_primary))
            };
            if ui.add(btn).clicked() {
                *active_tab = tab;
            }
        }
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("help_center_scroll")
        .show(ui, |ui| match *active_tab {
            HelpTab::GetStarted => {
                section_card(ui, colors, "1. Launch & Automatic Discovery", |ui| {
                    ui.label("On startup, vmcraft probes VirtualBox (VBoxManage), Hyper-V (CIM), host telemetry, and your local ISO library (~/.vmcraft/isos) asynchronously without blocking the UI. Press F5 anytime to refresh.");
                });
                section_card(ui, colors, "2. Select a Target VM", |ui| {
                    ui.label("Open Dashboard to see all registered VirtualBox and Hyper-V guests with live RUNNING / STOPPED pills. Click Select on any VM (for example ubu-2404-lts) to target it across VMs, Snapshots, ISOs, Shot, and Detonate.");
                });
                section_card(ui, colors, "3. Take a Clean Baseline Snapshot", |ui| {
                    ui.label("Before installing software or running untrusted binaries, open Snapshots, enter a clear name (such as clean-base), check 'allow mutate (== --yes)' in the status bar, and click Create.");
                });
                section_card(ui, colors, "4. Fetch & Attach a Linux or UEFI ISO", |ui| {
                    ui.label("Open the ISOs tab to browse curated Linux distros (Alpine, Ubuntu LTS, Debian, Fedora, Arch) and EDK2 UEFI Shell 2.2 boot images. Click Resolve URL to inspect the HTTPS link, Download ISO to stream it with automatic SHA-256 verification and .sha256 sidecar, then click Attach to VM to mount it on the selected VirtualBox optical drive.");
                });
                section_card(ui, colors, "5. Operate, Inspect & Roll Back", |ui| {
                    ui.label("Use VMs (Guest Runner & Clean-Room Linux Craft Test), Shot (live framebuffer PNG), Detonate (airgapped analysis), Swarm (Firecracker), or Commands (Ctrl+K). Restore your baseline snapshot from Snapshots when finished.");
                });
            }
            HelpTab::Functions => {
                section_card(ui, colors, "Core Hypervisor & Storage Tabs", |ui| {
                    bullet(ui, colors, "Dashboard & VMs", "4 KPI cards, live search & provider filters (All / Running / VirtualBox / Hyper-V), parsed Selected VM Details, VM create/start/stop/delete, and in-GUI Guest Runner & Clean-Room Linux Craft Test.");
                    bullet(ui, colors, "Snapshots", "List, create, restore, and delete VirtualBox snapshots for repeatable clean-room testing.");
                    bullet(ui, colors, "Network & Disks", "Structured tables for VirtualBox host-only interfaces and .vdi virtual disks (State, Format, Capacity, Location, UUID) with creation controls.");
                });
                section_card(ui, colors, "ISOs Tab (Linux, UEFI Shell & External Fido)", |ui| {
                    bullet(ui, colors, "Curated Catalog", "6 Linux distros (Alpine 3.21 Virt, Ubuntu 24.04.2 Server/Desktop, Debian 12 Netinst, Fedora 41, Arch Latest), 2 UEFI Shell 2.2 boot ISOs (Release/Debug), and 2 Windows entries.");
                    bullet(ui, colors, "Streaming SHA-256", "Downloads write to <file>.part over HTTPS, hash in 1 MiB chunks on the fly, promote atomically to .iso, and save a .sha256 sidecar for one-click Verify.");
                    bullet(ui, colors, "Optional External Fido.ps1", "vmcraft stays pure MIT/Apache-2.0 and never bundles GPLv3 Fido code. Point to an external Fido.ps1 (or place it in ~/.vmcraft/isos/) to resolve signed Microsoft CDN URLs with a 20-hour local cache to avoid Sentinel rate-limits.");
                    bullet(ui, colors, "One-Click Attach", "Mounts any local ISO onto the selected VirtualBox VM optical drive, auto-detecting IDE vs SATA storage controllers.");
                });
                section_card(ui, colors, "Isolation, Micro-VMs & Automation", |ui| {
                    bullet(ui, colors, "Shot", "Captures a live VirtualBox framebuffer PNG screenshot and renders it directly inside the app.");
                    bullet(ui, colors, "Detonate", "Clean-room airgap pipeline: severs network adapters 1..4, disables clipboard/DnD, runs inside guest via guestcontrol, extracts text/ASM artifacts only, and rolls back unconditionally.");
                    bullet(ui, colors, "Swarm (Firecracker)", "Probes WSL2 /dev/kvm and firecracker binary, lists active micro-VM processes, and spawns/kills sub-10ms ephemeral micro-VMs.");
                    bullet(ui, colors, "Sandbox & Commands", "30s local Python/PowerShell sandbox runner and interactive 37-verb ArtCraft Command Bus explorer.");
                });
            }
            HelpTab::Usage => {
                section_card(ui, colors, "Desktop Workstation (vmcraft-app)", |ui| {
                    ui.label("Navigate via the top menu bar (Session, Edit, View, Command, Window, Help) or the hideable tab bar (View -> Show Tab Bar / Ctrl+B). Move or hide the status bar via View -> Status Bar. Press Ctrl+K for the Command Palette, F1 for Shortcuts, F5 to refresh, and Ctrl+MouseWheel / Ctrl+= / Ctrl+- / Ctrl+1 to zoom.");
                });
                section_card(ui, colors, "CLI (vmcraft-mcp)", |ui| {
                    ui.monospace(
                        "vmcraft-mcp status\n\
                         vmcraft-mcp vm list\n\
                         vmcraft-mcp vm info --vm ubu-2404-lts\n\
                         vmcraft-mcp iso catalog --category linux\n\
                         vmcraft-mcp iso fetch --id uefi-shell-2.2-release --yes\n\
                         vmcraft-mcp iso verify --path \"$env:USERPROFILE\\.vmcraft\\isos\\UEFI-Shell-2.2-24H2-RELEASE.iso\"\n\
                         vmcraft-mcp iso attach --vm ubu-2404-lts --iso \"$env:USERPROFILE\\.vmcraft\\isos\\UEFI-Shell-2.2-24H2-RELEASE.iso\" --yes\n\
                         vmcraft-mcp pipeline naked-test --repo owner/repo --baseline ubu-2404-lts --snapshot clean-desktop-2026-11-22 --user user --pass user --linux --yes",
                    );
                });
                section_card(ui, colors, "MCP Stdio Server & Environment Variables", |ui| {
                    ui.monospace("vmcraft-mcp mcp");
                    ui.add_space(4.0);
                    ui.label("stdout is strictly reserved for JSON-RPC 2.0 frames; diagnostics go to stderr. Optional env vars: VMCRAFT_ALLOW_MUTATE=1, VMCRAFT_ISO_DIR, VMCRAFT_FIDO_SCRIPT, VBOXMANAGE_PATH.");
                });
            }
            HelpTab::Tips => {
                section_card(ui, colors, "High-Leverage Workflows", |ui| {
                    bullet(ui, colors, "Fast Smoke Guests", "Use alpine-3.21-virt (~62 MB) or uefi-shell-2.2-release (~12 MB) from the ISOs tab to verify boot and optical attachment in seconds.");
                    bullet(ui, colors, "Avoid Microsoft Sentinel Rejections", "Microsoft rejects rapid back-to-back Fido.ps1 -GetUrl requests from the same IP. vmcraft caches resolved Windows URLs for 20 hours in .fido-url-cache.json automatically.");
                    bullet(ui, colors, "Pre-fill VM OS Type", "Click 'Use OS Type in VMs' on any ISO catalog card to jump straight to the VMs tab with the matching VirtualBox ostype (Ubuntu_64, Debian_64, Linux_64, etc.) pre-filled.");
                    bullet(ui, colors, "Command Palette & Schemas", "Press Ctrl+K to search all 37 Command Bus verbs, or run `vmcraft-mcp describe <verb>` from CLI before scripting.");
                    bullet(ui, colors, "Hyphenated Guest Args", "In CLI `guest run`, pass hyphenated guest flags with `=` syntax: `--args=-lc`.");
                });
            }
            HelpTab::Safety => {
                section_card(ui, colors, "Mutating Operations Safety Gate", |ui| {
                    ui.label(
                        RichText::new(
                            "Every state-changing verb is blocked by default until you explicitly opt in.",
                        )
                        .strong()
                        .color(colors.warn_fg),
                    );
                    ui.label("Gated verbs: vm create/start/stop/delete, snapshot create/restore/delete, network create/remove, disk create, iso fetch (download), iso attach, airgap, guest.exec, pipeline.detonate, pipeline.naked_install, firecracker spawn/kill. Enable via the status-bar 'allow mutate' checkbox, CLI `--yes`, or `VMCRAFT_ALLOW_MUTATE=1`.");
                    ui.label("Read-only inspection (status, vm list, iso catalog, iso list, iso verify, iso fetch --url-only) always runs without confirmation.");
                });
                section_card(ui, colors, "Transport, Containment & Prerequisites", |ui| {
                    bullet(ui, colors, "HTTPS-Only ISO Downloads", "iso.fetch rejects http://, file://, and path-traversal filenames; partial transfers stay in .part files until complete.");
                    bullet(ui, colors, "Zero-Unwrap Containment", "unwrap_used, expect_used, panic, todo, and unimplemented are denied workspace-wide; absent hypervisors return structured `status: down` JSON receipts.");
                    bullet(ui, colors, "Guest Additions & WSL2 KVM", "guestcontrol and pipeline_naked_install require VirtualBox Guest Additions inside the guest. Firecracker requires wsl.exe with /dev/kvm and the firecracker binary in WSL2.");
                });
            }
            HelpTab::Planned => {
                section_card(ui, colors, "Roadmap (Unbuilt — Honest Status)", |ui| {
                    bullet(ui, colors, "QEMU/KVM & Apple Virtualization", "Native Linux QEMU/KVM and macOS Virtualization.framework providers alongside VirtualBox and Hyper-V.");
                    bullet(ui, colors, "Unattended Seed ISO Generator", "Pure-Rust autounattend.xml (Windows) and cloud-init NoCloud (Linux) seed ISO generator paired with the ISOs tab.");
                    bullet(ui, colors, "Live GUI Control Port", "Optional `--control PORT` bridge for external live UI automation.");
                    bullet(ui, colors, "Shared AI Core", "Migrate vmcraft-core/src/llm.rs to shared craft-ai-core.");
                });
            }
            HelpTab::OtherCrafts => {
                section_card(ui, colors, "Craft Fleet Integration", |ui| {
                    bullet(ui, colors, "devcraft", "Fleet orchestration and clean-room `naked-test` consumer install verification target.");
                    bullet(ui, colors, "wincraft", "Windows host diagnostics, RAM Weeder, and MFT storage treemaps; vmcraft handles VM/hypervisor isolation and ISO provisioning.");
                    bullet(ui, colors, "craft-launcher", "Launches and manages installed Craft workstations.");
                });
            }
        });
}
