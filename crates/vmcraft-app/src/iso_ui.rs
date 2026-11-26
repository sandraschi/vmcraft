//! Modular L3 egui page for Linux, UEFI Shell, and optional external-Fido Windows ISOs.

use eframe::egui::{self, RichText};
use serde_json::Value;
use vmcraft_core::iso::{
    builtin_catalog, default_iso_dir, find_fido_script, IsoCatalogEntry, LocalIsoItem,
};
use vmcraft_core::Command;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IsoCategoryFilter {
    All,
    Linux,
    Uefi,
    Windows,
}

impl IsoCategoryFilter {
    pub fn label(self) -> &'static str {
        match self {
            IsoCategoryFilter::All => "All",
            IsoCategoryFilter::Linux => "Linux",
            IsoCategoryFilter::Uefi => "UEFI",
            IsoCategoryFilter::Windows => "Windows (Fido)",
        }
    }

    pub fn matches(self, category: &str) -> bool {
        match self {
            IsoCategoryFilter::All => true,
            IsoCategoryFilter::Linux => category == "linux",
            IsoCategoryFilter::Uefi => category == "uefi",
            IsoCategoryFilter::Windows => category == "windows",
        }
    }
}

pub struct IsoUiState {
    pub catalog: Vec<IsoCatalogEntry>,
    pub local_isos: Vec<LocalIsoItem>,
    pub category: IsoCategoryFilter,
    pub search: String,
    pub dest_dir: String,
    pub fido_script: String,
    pub custom_url: String,
    pub win_lang: String,
    pub last_receipt: String,
    pub last_resolved_url: String,
}

impl IsoUiState {
    pub fn new() -> Self {
        let detected_fido = find_fido_script(None)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        Self {
            catalog: builtin_catalog(),
            local_isos: Vec::new(),
            category: IsoCategoryFilter::All,
            search: String::new(),
            dest_dir: default_iso_dir().display().to_string(),
            fido_script: detected_fido,
            custom_url: String::new(),
            win_lang: "default".to_string(),
            last_receipt: String::new(),
            last_resolved_url: String::new(),
        }
    }

    pub fn apply_response(&mut self, title: &str, value: &Value) {
        if title == "ISO List" {
            if let Some(arr) = value.get("isos").and_then(|v| v.as_array()) {
                self.local_isos = arr
                    .iter()
                    .filter_map(|item| serde_json::from_value::<LocalIsoItem>(item.clone()).ok())
                    .collect();
            }
            if let Some(fido) = value.get("fido_detected").and_then(|v| v.as_str()) {
                if self.fido_script.trim().is_empty() {
                    self.fido_script = fido.to_string();
                }
            }
        } else if title == "ISO Fetch" || title == "ISO Resolve URL" {
            if let Some(u) = value.get("url").and_then(|v| v.as_str()) {
                self.last_resolved_url = u.to_string();
            }
            let status = value
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            if status == "ok" {
                if value.get("mode").and_then(|v| v.as_str()) == Some("url_only") {
                    let cached = value
                        .get("cached_url_hit")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let fname = value
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("iso");
                    self.last_receipt = format!(
                        "Resolved {fname} ({})",
                        if cached { "20h Fido cache hit" } else { "live" }
                    );
                } else {
                    let fname = value
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("iso");
                    let mb = value.get("size_mb").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let sha = value.get("sha256").and_then(|v| v.as_str()).unwrap_or("");
                    let short_sha: String = sha.chars().take(16).collect();
                    self.last_receipt =
                        format!("Downloaded {fname} ({mb:.1} MB)  SHA-256: {short_sha}...");
                }
            } else {
                let reason = value
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("failed");
                self.last_receipt = format!("{title} ({status}): {reason}");
            }
        } else if title == "ISO Verify" {
            let status = value
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let sha = value.get("sha256").and_then(|v| v.as_str()).unwrap_or("");
            let verified = value.get("verified").and_then(|v| v.as_bool());
            self.last_receipt = match verified {
                Some(true) => format!("SHA-256 VERIFIED: {sha}"),
                Some(false) => format!("SHA-256 MISMATCH! Actual: {sha}"),
                None if status == "ok" => format!("SHA-256 computed: {sha}"),
                _ => format!(
                    "Verify failed: {}",
                    value
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                ),
            };
        } else if title == "ISO Attach" {
            let status = value
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            if status == "ok" {
                let vm = value.get("vm").and_then(|v| v.as_str()).unwrap_or("");
                let ctl = value
                    .get("controller")
                    .and_then(|v| v.as_str())
                    .unwrap_or("IDE");
                self.last_receipt = format!("Attached ISO to VM '{vm}' ({ctl} optical drive).");
            } else {
                let reason = value
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("failed");
                self.last_receipt = format!("Attach failed: {reason}");
            }
        }
    }
}

pub enum IsoUiAction {
    Dispatch {
        title: &'static str,
        cmd: Box<Command>,
    },
    CreateVmFromEntry {
        ostype: String,
    },
}

pub struct IsoThemeColors {
    pub card_bg: egui::Color32,
    pub card_border: egui::Color32,
    pub text_primary: egui::Color32,
    pub text_muted: egui::Color32,
    pub accent: egui::Color32,
    pub ok_bg: egui::Color32,
    pub ok_fg: egui::Color32,
    pub warn_bg: egui::Color32,
    pub warn_fg: egui::Color32,
    pub info_bg: egui::Color32,
    pub info_fg: egui::Color32,
}

fn pill(ui: &mut egui::Ui, label: &str, bg: egui::Color32, fg: egui::Color32) {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(label).small().strong().color(fg));
        });
}

pub fn render_iso_page(
    ui: &mut egui::Ui,
    state: &mut IsoUiState,
    colors: &IsoThemeColors,
    selected_vm: &str,
    confirm: bool,
) -> Vec<IsoUiAction> {
    let mut actions = Vec::new();

    ui.horizontal(|ui| {
        ui.heading("ISO Library & Fetcher");
        ui.add_space(8.0);
        pill(ui, "LINUX", colors.ok_bg, colors.ok_fg);
        pill(ui, "UEFI SHELL", colors.info_bg, colors.info_fg);
        pill(
            ui,
            "WINDOWS (EXTERNAL FIDO)",
            colors.warn_bg,
            colors.warn_fg,
        );
    });
    ui.label(
        RichText::new(
            "Direct HTTPS download + streaming SHA-256 verification for Linux & UEFI ISOs. Windows ISOs resolve via optional external Fido.ps1 (20h URL cache).",
        )
        .small()
        .color(colors.text_muted),
    );
    ui.add_space(6.0);

    // Top configuration & custom URL card
    egui::Frame::NONE
        .fill(colors.card_bg)
        .stroke(egui::Stroke::new(1.0, colors.card_border))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("ISO Dir:");
                ui.add(egui::TextEdit::singleline(&mut state.dest_dir).desired_width(280.0));
                if ui.button("Scan Local ISOs").clicked() {
                    actions.push(IsoUiAction::Dispatch {
                        title: "ISO List",
                        cmd: Box::new(Command::IsoList {
                            dir: Some(state.dest_dir.clone()),
                        }),
                    });
                }
                ui.separator();
                ui.label("External Fido.ps1:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.fido_script)
                        .hint_text("Optional path to Fido.ps1 for Windows ISOs")
                        .desired_width(240.0),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Custom HTTPS ISO URL:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.custom_url)
                        .hint_text("https://.../image.iso")
                        .desired_width(420.0),
                );
                if ui
                    .add_enabled(
                        !state.custom_url.trim().is_empty(),
                        egui::Button::new("Download URL"),
                    )
                    .clicked()
                {
                    actions.push(IsoUiAction::Dispatch {
                        title: "ISO Fetch",
                        cmd: Box::new(Command::IsoFetch {
                            id: None,
                            url: Some(state.custom_url.trim().to_string()),
                            dest_dir: Some(state.dest_dir.clone()),
                            fido_script: None,
                            win_version: None,
                            lang: None,
                            arch: None,
                            url_only: false,
                            confirm,
                        }),
                    });
                }
                if !confirm {
                    ui.label(
                        RichText::new(
                            "Enable 'Allow mutating ops' in status bar to download/attach",
                        )
                        .small()
                        .color(colors.warn_fg),
                    );
                }
            });
        });

    if !state.last_receipt.is_empty() {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(&state.last_receipt)
                    .strong()
                    .color(colors.accent),
            );
            if !state.last_resolved_url.is_empty() && ui.small_button("Copy URL").clicked() {
                ui.ctx().copy_text(state.last_resolved_url.clone());
            }
        });
    }

    ui.add_space(6.0);

    // Filter bar for Catalog
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Curated Catalog:").strong());
        for cat in [
            IsoCategoryFilter::All,
            IsoCategoryFilter::Linux,
            IsoCategoryFilter::Uefi,
            IsoCategoryFilter::Windows,
        ] {
            let selected = state.category == cat;
            if ui.selectable_label(selected, cat.label()).clicked() {
                state.category = cat;
            }
        }
        ui.separator();
        ui.label("Filter:");
        ui.add(
            egui::TextEdit::singleline(&mut state.search)
                .hint_text("ubuntu, alpine, uefi, win11...")
                .desired_width(180.0),
        );
    });

    ui.add_space(4.0);
    let q = state.search.trim().to_ascii_lowercase();
    let entries: Vec<IsoCatalogEntry> = state
        .catalog
        .iter()
        .filter(|e| state.category.matches(&e.category))
        .filter(|e| {
            q.is_empty()
                || e.name.to_ascii_lowercase().contains(&q)
                || e.id.to_ascii_lowercase().contains(&q)
                || e.notes.to_ascii_lowercase().contains(&q)
        })
        .cloned()
        .collect();

    egui::ScrollArea::vertical()
        .id_salt("iso_catalog_scroll")
        .max_height(265.0)
        .show(ui, |ui| {
            for entry in entries {
                egui::Frame::NONE
                    .fill(colors.card_bg)
                    .stroke(egui::Stroke::new(1.0, colors.card_border))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let (bg, fg, tag) = match entry.category.as_str() {
                                "linux" => (colors.ok_bg, colors.ok_fg, "LINUX"),
                                "uefi" => (colors.info_bg, colors.info_fg, "UEFI"),
                                _ => (colors.warn_bg, colors.warn_fg, "WINDOWS · FIDO"),
                            };
                            pill(ui, tag, bg, fg);
                            ui.label(
                                RichText::new(&entry.name)
                                    .strong()
                                    .color(colors.text_primary),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "{} · {} · ~{} MB · VBox: {}",
                                    entry.version,
                                    entry.arch,
                                    entry.size_hint_mb,
                                    entry.vbox_ostype
                                ))
                                .small()
                                .color(colors.text_muted),
                            );
                        });
                        ui.label(RichText::new(&entry.notes).small().color(colors.text_muted));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let fido_opt = if state.fido_script.trim().is_empty() {
                                None
                            } else {
                                Some(state.fido_script.trim().to_string())
                            };
                            if ui.small_button("Resolve URL").clicked() {
                                actions.push(IsoUiAction::Dispatch {
                                    title: "ISO Resolve URL",
                                    cmd: Box::new(Command::IsoFetch {
                                        id: Some(entry.id.clone()),
                                        url: None,
                                        dest_dir: Some(state.dest_dir.clone()),
                                        fido_script: fido_opt.clone(),
                                        win_version: None,
                                        lang: Some(state.win_lang.clone()),
                                        arch: Some(entry.arch.clone()),
                                        url_only: true,
                                        confirm: false,
                                    }),
                                });
                            }
                            if ui.small_button("Download ISO").clicked() {
                                actions.push(IsoUiAction::Dispatch {
                                    title: "ISO Fetch",
                                    cmd: Box::new(Command::IsoFetch {
                                        id: Some(entry.id.clone()),
                                        url: None,
                                        dest_dir: Some(state.dest_dir.clone()),
                                        fido_script: fido_opt,
                                        win_version: None,
                                        lang: Some(state.win_lang.clone()),
                                        arch: Some(entry.arch.clone()),
                                        url_only: false,
                                        confirm,
                                    }),
                                });
                            }
                            if ui.small_button("Use OS Type in VMs").clicked() {
                                actions.push(IsoUiAction::CreateVmFromEntry {
                                    ostype: entry.vbox_ostype.clone(),
                                });
                            }
                        });
                    });
                ui.add_space(4.0);
            }
        });

    ui.separator();

    // Local ISO Library
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("Local ISO Library ({})", state.local_isos.len())).strong());
        if !selected_vm.trim().is_empty() {
            ui.label(
                RichText::new(format!("Target VM for Attach: {selected_vm}"))
                    .small()
                    .color(colors.accent),
            );
        } else {
            ui.label(
                RichText::new("Select a VM on Dashboard/VMs to enable one-click Attach")
                    .small()
                    .color(colors.text_muted),
            );
        }
    });

    if state.local_isos.is_empty() {
        ui.label(
            RichText::new("No local .iso / .img files in this directory yet. Download one above or click 'Scan Local ISOs'.")
                .small()
                .color(colors.text_muted),
        );
    } else {
        egui::ScrollArea::vertical()
            .id_salt("local_iso_scroll")
            .max_height(170.0)
            .show(ui, |ui| {
                egui::Grid::new("local_iso_grid")
                    .striped(true)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Filename").strong());
                        ui.label(RichText::new("Size").strong());
                        ui.label(RichText::new("Checksum").strong());
                        ui.label(RichText::new("Actions").strong());
                        ui.end_row();
                        for iso in &state.local_isos {
                            ui.label(&iso.filename);
                            ui.label(format!("{:.1} MB", iso.size_mb));
                            if iso.sha256_sidecar.is_some() {
                                pill(ui, "SHA-256", colors.ok_bg, colors.ok_fg);
                            } else {
                                ui.label(RichText::new("none").small().color(colors.text_muted));
                            }
                            ui.horizontal(|ui| {
                                if ui.small_button("Verify").clicked() {
                                    actions.push(IsoUiAction::Dispatch {
                                        title: "ISO Verify",
                                        cmd: Box::new(Command::IsoVerify {
                                            path: iso.path.clone(),
                                            expected_sha256: iso.sha256_sidecar.clone(),
                                        }),
                                    });
                                }
                                let can_attach = !selected_vm.trim().is_empty();
                                if ui
                                    .add_enabled(
                                        can_attach,
                                        egui::Button::new("Attach to VM").small(),
                                    )
                                    .clicked()
                                {
                                    actions.push(IsoUiAction::Dispatch {
                                        title: "ISO Attach",
                                        cmd: Box::new(Command::IsoAttach {
                                            vm: selected_vm.to_string(),
                                            iso_path: iso.path.clone(),
                                            confirm,
                                        }),
                                    });
                                }
                            });
                            ui.end_row();
                        }
                    });
            });
    }

    actions
}
