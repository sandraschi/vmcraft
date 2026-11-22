//! Tier-3 L3: workstation pages (VMs, snapshots, network, disks, shot, sandbox, commands bus).

use eframe::egui::{self, RichText};
use std::sync::{mpsc, Arc};
use vmcraft_core::{dispatch, Command};

const ACCENT_CYAN: egui::Color32 = egui::Color32::from_rgb(56, 189, 248);
const ACCENT_AMBER: egui::Color32 = egui::Color32::from_rgb(251, 146, 60);
const ACCENT_EMERALD: egui::Color32 = egui::Color32::from_rgb(52, 211, 153);

/// Cyberpunk / Slate-950 high-contrast dark visuals.
fn harsh_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_rgb(241, 245, 249));
    visuals.window_fill = egui::Color32::from_rgb(15, 23, 42); // slate-900
    visuals.panel_fill = egui::Color32::from_rgb(15, 23, 42);
    visuals.extreme_bg_color = egui::Color32::from_rgb(2, 6, 23); // slate-950
    visuals.code_bg_color = egui::Color32::from_rgb(30, 41, 59); // slate-800
    visuals.faint_bg_color = egui::Color32::from_rgb(30, 41, 59);
    visuals.warn_fg_color = egui::Color32::from_rgb(251, 191, 36);
    visuals.error_fg_color = egui::Color32::from_rgb(248, 113, 113);

    visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(203, 213, 225));
    visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(226, 232, 240));
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);

    visuals.selection.bg_fill = egui::Color32::from_rgb(14, 116, 144);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.hyperlink_color = egui::Color32::from_rgb(56, 189, 248);

    visuals.window_corner_radius = egui::CornerRadius::same(8);
    visuals.menu_corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(6);

    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 65, 85));
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(71, 85, 105));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, ACCENT_CYAN);
    visuals
}

/// Crisp light visuals (slate-50 panel with clean border hairline).
fn crisp_light_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.override_text_color = Some(egui::Color32::BLACK);
    visuals.weak_text_color = Some(egui::Color32::BLACK);
    visuals.window_fill = egui::Color32::WHITE;
    visuals.panel_fill = egui::Color32::WHITE;
    visuals.extreme_bg_color = egui::Color32::WHITE;
    visuals.faint_bg_color = egui::Color32::WHITE;
    visuals.code_bg_color = egui::Color32::WHITE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, egui::Color32::BLACK);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::BLACK);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(226, 232, 240));
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(203, 213, 225));
    visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.5, egui::Color32::from_rgb(2, 132, 199));
    visuals.window_corner_radius = egui::CornerRadius::same(8);
    visuals.menu_corner_radius = egui::CornerRadius::same(6);
    visuals
}

/// Apply typography and visuals to context.
fn apply_theme(ctx: &egui::Context, dark_mode: bool) {
    ctx.set_visuals(if dark_mode {
        harsh_visuals()
    } else {
        crisp_light_visuals()
    });
    ctx.style_mut_of(
        if dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        },
        |style| {
            use egui::TextStyle::*;
            for (text_style, size) in [
                (Body, 14.5),
                (Button, 14.0),
                (Heading, 20.0),
                (Monospace, 13.0),
                (Small, 11.5),
            ] {
                if let Some(font) = style.text_styles.get_mut(&text_style) {
                    font.size = size;
                }
            }
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);
            style.spacing.button_padding = egui::vec2(8.0, 4.0);
        },
    );
}

/// Theme toggle button: renders vector sun / moon.
fn theme_toggle_button(ui: &mut egui::Ui, dark_mode: &mut bool) {
    let size = egui::vec2(28.0, 24.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&resp);
        ui.painter().rect(
            rect,
            egui::CornerRadius::same(6),
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let center = rect.center();
        let painter = ui.painter();
        if *dark_mode {
            let sun_color = egui::Color32::from_rgb(251, 191, 36);
            painter.circle_filled(center, 4.0, sun_color);
            for i in 0..8 {
                let angle = (i as f32) * std::f32::consts::TAU / 8.0;
                let dir = egui::vec2(angle.cos(), angle.sin());
                painter.line_segment(
                    [center + dir * 5.5, center + dir * 8.0],
                    egui::Stroke::new(1.5, sun_color),
                );
            }
        } else {
            let moon_color = egui::Color32::from_rgb(51, 65, 85);
            painter.circle_filled(center, 5.5, moon_color);
            painter.circle_filled(center + egui::vec2(2.5, -1.8), 4.8, visuals.bg_fill);
        }
    }
    if resp.on_hover_text("Toggle theme").clicked() {
        *dark_mode = !*dark_mode;
        apply_theme(ui.ctx(), *dark_mode);
    }
}

/// Ctrl+MouseWheel global UI zoom handler.
fn handle_zoom(ctx: &egui::Context) {
    let zoom = ctx.input(|input| input.zoom_delta());
    if (zoom - 1.0).abs() > f32::EPSILON {
        ctx.set_zoom_factor((ctx.zoom_factor() * zoom).clamp(0.6, 2.5));
    }
}

/// Render a glowing badge pill.
fn render_badge(ui: &mut egui::Ui, label: &str, bg: egui::Color32, fg: egui::Color32) {
    let padding = egui::vec2(8.0, 2.0);
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_string(), egui::FontId::proportional(11.5), fg);
    let size = galley.size() + padding * 2.0;
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().rect(
            rect,
            egui::CornerRadius::same(10),
            bg,
            egui::Stroke::new(1.0, fg),
            egui::StrokeKind::Inside,
        );
        let text_pos = rect.left_top() + padding;
        ui.painter().galley(text_pos, galley, fg);
    }
}

/// SOTA switch_on_hover implementation for egui top menu bars.
fn switch_on_hover(ctx: &egui::Context, titles: &[egui::Response]) {
    let is_open =
        |t: &egui::Response| egui::Popup::is_id_open(ctx, egui::Popup::default_response_id(t));
    if !titles.iter().any(is_open) {
        return;
    }
    if let Some(title) = titles
        .iter()
        .find(|t| t.hovered() && !t.clicked() && !is_open(t))
    {
        egui::Popup::open_id(ctx, egui::Popup::default_response_id(title));
    }
}

/// Filter registered ArtCraft command bus entries for the Command Palette.
fn filter_palette_commands(query: &str) -> Vec<(String, String, String)> {
    let q = query.trim().to_lowercase();
    let catalog = vmcraft_core::commands_catalog();
    let Some(arr) = catalog.as_array() else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|item| {
            let id = item.get("id").and_then(|v| v.as_str())?.to_string();
            let label = item
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let desc = item
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if q.is_empty()
                || id.to_lowercase().contains(&q)
                || label.to_lowercase().contains(&q)
                || desc.to_lowercase().contains(&q)
            {
                Some((id, label, desc))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HelpTab {
    GetStarted,
    Functions,
    Usage,
    Tips,
    Safety,
    Planned,
    OtherCrafts,
}

impl HelpTab {
    fn label(self) -> &'static str {
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusBarPos {
    Bottom,
    Top,
    Off,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Dashboard,
    Vms,
    Snapshots,
    Network,
    Disks,
    Shot,
    Detonate,
    Swarm,
    Sandbox,
    Commands,
    Help,
}

impl Page {
    fn label(self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Vms => "VMs",
            Page::Snapshots => "Snapshots",
            Page::Network => "Network",
            Page::Disks => "Disks",
            Page::Shot => "Shot",
            Page::Detonate => "Detonate",
            Page::Swarm => "Swarm",
            Page::Sandbox => "Sandbox",
            Page::Commands => "Commands",
            Page::Help => "Help",
        }
    }
}

#[derive(Clone, Debug)]
pub struct VmItem {
    pub name: String,
    pub uuid: String,
    pub provider: String,
}

#[derive(Clone, Debug)]
pub struct SnapshotItem {
    pub name: String,
    pub uuid: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct FirecrackerInstance {
    pub pid: String,
    pub binary: String,
}

pub enum AppMessage {
    Status(serde_json::Value),
    VmList(serde_json::Value),
    HostInfo(serde_json::Value),
    CommandResult {
        title: String,
        value: serde_json::Value,
    },
}

pub struct VmApp {
    rt: Option<Arc<tokio::runtime::Runtime>>,
    rt_error: Option<String>,
    tx: mpsc::Sender<AppMessage>,
    rx: mpsc::Receiver<AppMessage>,
    pending_tasks: usize,
    page: Page,
    status: String,
    host_info: Option<String>,
    vms: Vec<VmItem>,
    snapshots: Vec<SnapshotItem>,
    shot_image: Option<egui::ColorImage>,
    shot_texture: Option<egui::TextureHandle>,
    bus_cmd: String,
    bus_params: String,
    vm_output: String,
    vm_name: String,
    ostype: String,
    memory_mb: String,
    snap_name: String,
    net_ip: String,
    netmask: String,
    net_name: String,
    disk_path: String,
    disk_gb: String,
    shot_out: String,
    sandbox_code: String,
    sandbox_output: String,
    mutate_confirm: bool,
    dark_mode: bool,
    airgap_active: bool,
    disasm_code: String,
    detonate_keygen: String,
    detonate_target: String,
    detonate_disasm_exe: String,
    detonate_user: String,
    detonate_pass: String,
    detonate_guest_out: String,
    detonate_host_out: String,
    detonate_airgap: bool,
    swarm_instances: Vec<FirecrackerInstance>,
    swarm_selected_idx: Option<usize>,
    swarm_spawn_id: String,
    swarm_kernel: String,
    swarm_rootfs: String,
    swarm_kvm_available: Option<bool>,
    swarm_fc_installed: Option<bool>,
    swarm_probe_reason: Option<String>,
    help_tab: HelpTab,
    show_palette: bool,
    palette_query: String,
    show_shortcuts: bool,
    show_about: bool,
    show_licenses: bool,
    show_tab_bar: bool,
    status_bar_pos: StatusBarPos,
}

impl VmApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::channel();
        let built = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .or_else(|_| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
            });
        let (rt, rt_error) = match built {
            Ok(r) => (Some(Arc::new(r)), None),
            Err(e) => (None, Some(format!("runtime init failed: {e}"))),
        };

        let mut app = Self {
            rt,
            rt_error,
            tx,
            rx,
            pending_tasks: 0,
            page: Page::Dashboard,
            status: "Connecting to hypervisors…".to_string(),
            host_info: None,
            vms: Vec::new(),
            snapshots: Vec::new(),
            shot_image: None,
            shot_texture: None,
            bus_cmd: "vm.list".to_string(),
            bus_params: "{}".to_string(),
            vm_output: String::new(),
            vm_name: String::new(),
            ostype: "Ubuntu_64".to_string(),
            memory_mb: "4096".to_string(),
            snap_name: String::new(),
            net_ip: "192.168.56.1".to_string(),
            netmask: "255.255.255.0".to_string(),
            net_name: String::new(),
            disk_path: String::new(),
            disk_gb: "20".to_string(),
            shot_out: String::new(),
            sandbox_code: "print('hi-vm')".to_string(),
            sandbox_output: String::new(),
            mutate_confirm: false,
            dark_mode: true,
            airgap_active: false,
            disasm_code: String::new(),
            detonate_keygen: String::new(),
            detonate_target: String::new(),
            detonate_disasm_exe: "ida64.exe".to_string(),
            detonate_user: String::new(),
            detonate_pass: String::new(),
            detonate_guest_out: "C:\\detonate\\out.asm".to_string(),
            detonate_host_out: "out.asm".to_string(),
            detonate_airgap: true,
            swarm_instances: Vec::new(),
            swarm_selected_idx: None,
            swarm_spawn_id: "microvm-1".to_string(),
            swarm_kernel: "/vmlinux".to_string(),
            swarm_rootfs: "/rootfs.ext4".to_string(),
            swarm_kvm_available: None,
            swarm_fc_installed: None,
            swarm_probe_reason: None,
            help_tab: HelpTab::GetStarted,
            show_palette: false,
            palette_query: String::new(),
            show_shortcuts: false,
            show_about: false,
            show_licenses: false,
            show_tab_bar: true,
            status_bar_pos: StatusBarPos::Bottom,
        };

        // Auto-fetch on startup so VM list and status are ready immediately
        app.refresh_all();
        app
    }

    fn refresh_all(&mut self) {
        self.spawn_status();
        self.spawn_vms();
        self.spawn_host_info();
    }

    fn spawn_status(&mut self) {
        self.spawn_task(|tx| async move {
            let res = dispatch(Command::Status).await;
            let _ = tx.send(AppMessage::Status(res));
        });
    }

    fn spawn_vms(&mut self) {
        self.spawn_task(|tx| async move {
            let res = dispatch(Command::VmList { limit: Some(50) }).await;
            let _ = tx.send(AppMessage::VmList(res));
        });
    }

    fn spawn_host_info(&mut self) {
        self.spawn_task(|tx| async move {
            let res = dispatch(Command::HostInfo).await;
            let _ = tx.send(AppMessage::HostInfo(res));
        });
    }

    fn spawn_command(&mut self, title: impl Into<String>, cmd: Command) {
        let t = title.into();
        self.spawn_task(move |tx| async move {
            let res = dispatch(cmd).await;
            let _ = tx.send(AppMessage::CommandResult {
                title: t,
                value: res,
            });
        });
    }

    fn spawn_task<F, Fut>(&mut self, make: F)
    where
        F: FnOnce(mpsc::Sender<AppMessage>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        if let Some(ref rt) = self.rt {
            let tx = self.tx.clone();
            self.pending_tasks += 1;
            rt.spawn(async move {
                make(tx).await;
            });
        }
    }

    fn drain(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            if self.pending_tasks > 0 {
                self.pending_tasks -= 1;
            }
            match msg {
                AppMessage::Status(val) => {
                    let vbox = val
                        .get("vbox_present")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let hv = val
                        .get("hyperv_present")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let mut parts = Vec::new();
                    if vbox {
                        parts.push("VirtualBox: active");
                    } else {
                        parts.push("VirtualBox: absent");
                    }
                    if hv {
                        parts.push("Hyper-V: active");
                    } else {
                        parts.push("Hyper-V: absent");
                    }
                    self.status = parts.join(" | ");
                }
                AppMessage::VmList(val) => {
                    if let Some(arr) = val.get("vms").and_then(|v| v.as_array()) {
                        self.vms.clear();
                        for item in arr {
                            let name = item
                                .get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("")
                                .to_string();
                            let uuid = item
                                .get("uuid")
                                .and_then(|u| u.as_str())
                                .unwrap_or("")
                                .to_string();
                            let provider = item
                                .get("provider")
                                .and_then(|p| p.as_str())
                                .unwrap_or("virtualbox")
                                .to_string();
                            if !name.is_empty() {
                                self.vms.push(VmItem {
                                    name,
                                    uuid,
                                    provider,
                                });
                            }
                        }
                        if self.vm_name.is_empty() && !self.vms.is_empty() {
                            self.vm_name = self.vms[0].name.clone();
                        }
                    }
                    self.vm_output =
                        serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string());
                }
                AppMessage::HostInfo(val) => {
                    if let Some(ver) = val.get("vbox_version").and_then(|s| s.as_str()) {
                        let count = val
                            .get("ostype_count")
                            .and_then(|c| c.as_u64())
                            .unwrap_or(0);
                        self.host_info = Some(format!("VBox {ver} ({count} templates)"));
                    }
                }
                AppMessage::CommandResult { title, value } => {
                    if title == "Snapshot List" {
                        self.snapshots.clear();
                        if let Some(arr) = value.get("snapshots").and_then(|s| s.as_array()) {
                            for s in arr {
                                let name = s
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let uuid = s
                                    .get("uuid")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let description = s
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                if !name.is_empty() {
                                    self.snapshots.push(SnapshotItem {
                                        name,
                                        uuid,
                                        description,
                                    });
                                }
                            }
                        }
                    } else if title == "VM Screenshot" && !self.shot_out.is_empty() {
                        if let Ok(bytes) = std::fs::read(&self.shot_out) {
                            if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                                let size = [dyn_img.width() as usize, dyn_img.height() as usize];
                                let rgba = dyn_img.to_rgba8();
                                let color_image =
                                    egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                                self.shot_image = Some(color_image);
                            }
                        }
                    } else if (title == "Pipeline Detonate" || title == "Detonate & Disassemble")
                        && !self.detonate_host_out.is_empty()
                    {
                        if let Ok(content) = std::fs::read_to_string(&self.detonate_host_out) {
                            self.disasm_code = content;
                        }
                        if let Some(shot) = value.get("screenshot").and_then(|v| v.as_str()) {
                            if let Ok(bytes) = std::fs::read(shot) {
                                if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                                    let size =
                                        [dyn_img.width() as usize, dyn_img.height() as usize];
                                    let rgba = dyn_img.to_rgba8();
                                    self.shot_image =
                                        Some(egui::ColorImage::from_rgba_unmultiplied(
                                            size,
                                            rgba.as_raw(),
                                        ));
                                }
                            }
                        }
                    } else if title == "Airgap Sever" {
                        self.airgap_active = true;
                    } else if title == "Airgap Reconnect" {
                        self.airgap_active = false;
                    } else if title == "Firecracker Probe" {
                        if value.get("status").and_then(|v| v.as_str()) == Some("ok") {
                            self.swarm_kvm_available =
                                value.get("kvm_available").and_then(|v| v.as_bool());
                            self.swarm_fc_installed =
                                value.get("installed").and_then(|v| v.as_bool());
                            self.swarm_probe_reason = None;
                        } else {
                            self.swarm_kvm_available = Some(false);
                            self.swarm_fc_installed = Some(false);
                            self.swarm_probe_reason = value
                                .get("reason")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                    } else if title == "Firecracker List" {
                        self.swarm_instances.clear();
                        if let Some(arr) = value.get("instances").and_then(|v| v.as_array()) {
                            for item in arr {
                                let pid = item
                                    .get("pid")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let binary = item
                                    .get("binary")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                if !pid.is_empty() {
                                    self.swarm_instances
                                        .push(FirecrackerInstance { pid, binary });
                                }
                            }
                        }
                        if let Some(idx) = self.swarm_selected_idx {
                            if idx >= self.swarm_instances.len() {
                                self.swarm_selected_idx = None;
                            }
                        }
                    }
                    let formatted =
                        serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
                    self.vm_output = format!("=== {title} ===\n{formatted}");
                }
            }
        }
    }

    fn confirm(&self) -> bool {
        self.mutate_confirm
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("vmcraft").strong().color(ACCENT_CYAN));
            render_badge(
                ui,
                "v0.1.0 (T1)",
                egui::Color32::from_rgb(14, 116, 144),
                egui::Color32::WHITE,
            );

            if ui.button("⟳ Refresh").clicked() {
                self.refresh_all();
            }

            if self.pending_tasks > 0 {
                ui.spinner();
                ui.label(RichText::new(format!("Loading ({}…)", self.pending_tasks)).weak());
            } else {
                ui.label(RichText::new(&self.status).color(ACCENT_EMERALD));
            }

            if let Some(ref host) = self.host_info {
                ui.separator();
                ui.label(RichText::new(host).weak());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                theme_toggle_button(ui, &mut self.dark_mode);
            });
        });

        ui.horizontal(|ui| {
            if self.airgap_active {
                render_badge(
                    ui,
                    "✂ HARD AIRGAP ENFORCED — ALL VIRTUAL NICS SEVERED — CLIPBOARD LOCKED",
                    egui::Color32::from_rgb(120, 53, 15),
                    ACCENT_AMBER,
                );
            } else {
                render_badge(
                    ui,
                    "🌐 NETWORK ADAPTERS CONNECTED",
                    egui::Color32::from_rgb(6, 78, 59),
                    ACCENT_EMERALD,
                );
            }

            ui.separator();
            ui.checkbox(&mut self.mutate_confirm, "allow mutate (== --yes)");
            ui.label(
                RichText::new("mutating verbs need this checkbox or VMCRAFT_ALLOW_MUTATE=1").weak(),
            );
        });
    }
}

impl eframe::App for VmApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.drain();

        // Handle Ctrl+MouseWheel zooming and global shortcuts
        handle_zoom(ui.ctx());
        ui.ctx().input(|i| {
            if (i.modifiers.ctrl && i.key_pressed(egui::Key::K))
                || (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::P))
            {
                self.show_palette = !self.show_palette;
            }
            if i.key_pressed(egui::Key::F1) || (i.modifiers.ctrl && i.key_pressed(egui::Key::Slash))
            {
                self.show_shortcuts = !self.show_shortcuts;
            }
            if i.key_pressed(egui::Key::F5) {
                self.refresh_all();
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Num1) {
                ui.ctx().set_zoom_factor(1.0);
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Q) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::B) {
                self.show_tab_bar = !self.show_tab_bar;
            }
        });

        if self.pending_tasks > 0 {
            ui.ctx().request_repaint();
        }

        if let Some(err) = self.rt_error.clone() {
            ui.colored_label(egui::Color32::RED, err);
            return;
        }

        // Standard Craft Menu Bar (Session, Edit, View, Command, Window, Help)
        egui::MenuBar::new().ui(ui, |ui| {
            let mut titles = Vec::with_capacity(6);

            let r_session = ui.menu_button("Session", |ui| {
                if ui.button("Refresh Hypervisors\tF5").clicked() {
                    self.refresh_all();
                    ui.close();
                }
                if ui
                    .button(if self.mutate_confirm {
                        "Disable Mutate Gate"
                    } else {
                        "Enable Mutate Gate"
                    })
                    .clicked()
                {
                    self.mutate_confirm = !self.mutate_confirm;
                    ui.close();
                }
                ui.separator();
                if ui.button("Quit\tCtrl+Q").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close();
                }
            });
            titles.push(r_session.response);

            let r_edit = ui.menu_button("Edit", |ui| {
                if ui.button("Copy Output").clicked() {
                    ui.ctx().copy_text(self.vm_output.clone());
                    ui.close();
                }
                if ui.button("Clear Output").clicked() {
                    self.vm_output.clear();
                    ui.close();
                }
            });
            titles.push(r_edit.response);

            let r_view = ui.menu_button("View", |ui| {
                if ui.button("Zoom In").clicked() {
                    let z = (ui.ctx().zoom_factor() * 1.1).clamp(0.6, 2.5);
                    ui.ctx().set_zoom_factor(z);
                    ui.close();
                }
                if ui.button("Zoom Out").clicked() {
                    let z = (ui.ctx().zoom_factor() / 1.1).clamp(0.6, 2.5);
                    ui.ctx().set_zoom_factor(z);
                    ui.close();
                }
                if ui.button("Reset Zoom (100%)\tCtrl+1").clicked() {
                    ui.ctx().set_zoom_factor(1.0);
                    ui.close();
                }
                ui.separator();
                if ui
                    .checkbox(&mut self.show_tab_bar, "Show Tab Bar\tCtrl+B")
                    .changed()
                {
                    ui.close();
                }
                ui.menu_button("Status Bar", |ui| {
                    if ui
                        .selectable_label(self.status_bar_pos == StatusBarPos::Bottom, "Bottom")
                        .clicked()
                    {
                        self.status_bar_pos = StatusBarPos::Bottom;
                        ui.close();
                    }
                    if ui
                        .selectable_label(self.status_bar_pos == StatusBarPos::Top, "Top")
                        .clicked()
                    {
                        self.status_bar_pos = StatusBarPos::Top;
                        ui.close();
                    }
                    if ui
                        .selectable_label(self.status_bar_pos == StatusBarPos::Off, "Off")
                        .clicked()
                    {
                        self.status_bar_pos = StatusBarPos::Off;
                        ui.close();
                    }
                });
                ui.separator();
                ui.menu_button("Interface Theme", |ui| {
                    if ui.selectable_label(self.dark_mode, "Dark").clicked() {
                        self.dark_mode = true;
                        apply_theme(ui.ctx(), true);
                        ui.close();
                    }
                    if ui.selectable_label(!self.dark_mode, "Light").clicked() {
                        self.dark_mode = false;
                        apply_theme(ui.ctx(), false);
                        ui.close();
                    }
                });
            });
            titles.push(r_view.response);

            let r_cmd = ui.menu_button("Command", |ui| {
                if ui.button("Command Palette...\tCtrl+K").clicked() {
                    self.show_palette = true;
                    ui.close();
                }
                if ui.button("Command Bus Tab").clicked() {
                    self.page = Page::Commands;
                    ui.close();
                }
                ui.separator();
                if ui.button("Probe Hypervisors").clicked() {
                    self.spawn_status();
                    ui.close();
                }
                if ui.button("Probe Firecracker KVM").clicked() {
                    self.spawn_command("Firecracker Probe", Command::FirecrackerProbe);
                    ui.close();
                }
            });
            titles.push(r_cmd.response);

            let r_win = ui.menu_button("Window", |ui| {
                for p in [
                    Page::Dashboard,
                    Page::Vms,
                    Page::Snapshots,
                    Page::Network,
                    Page::Disks,
                    Page::Shot,
                    Page::Detonate,
                    Page::Swarm,
                    Page::Sandbox,
                    Page::Commands,
                    Page::Help,
                ] {
                    if ui.selectable_label(self.page == p, p.label()).clicked() {
                        self.page = p;
                        ui.close();
                    }
                }
            });
            titles.push(r_win.response);

            let r_help = ui.menu_button("Help", |ui| {
                if ui.button("Command Palette...\tCtrl+K").clicked() {
                    self.show_palette = true;
                    ui.close();
                }
                if ui.button("Keyboard Shortcuts...\tF1").clicked() {
                    self.show_shortcuts = true;
                    ui.close();
                }
                ui.separator();
                for tab in [
                    HelpTab::GetStarted,
                    HelpTab::Usage,
                    HelpTab::Tips,
                    HelpTab::Safety,
                    HelpTab::Planned,
                    HelpTab::OtherCrafts,
                ] {
                    if ui.button(format!("{}...", tab.label())).clicked() {
                        self.help_tab = tab;
                        self.page = Page::Help;
                        ui.close();
                    }
                }
                if ui.button("Project Files & Docs...").clicked() {
                    self.help_tab = HelpTab::Functions;
                    self.page = Page::Help;
                    ui.close();
                }
                ui.separator();
                if ui.button("Licenses & Notices...").clicked() {
                    self.show_licenses = true;
                    ui.close();
                }
                if ui.button("About vmcraft").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });
            titles.push(r_help.response);

            switch_on_hover(ui.ctx(), &titles);
        });
        ui.separator();

        if self.status_bar_pos == StatusBarPos::Top {
            self.render_status_bar(ui);
            ui.separator();
        }

        if self.status_bar_pos == StatusBarPos::Bottom {
            egui::Panel::bottom("vmcraft_status_bar").show(ui, |ui| {
                self.render_status_bar(ui);
            });
        }

        // Page navigation bar (hideable single-click alternative to Window menu)
        if self.show_tab_bar {
            ui.horizontal(|ui| {
                for p in [
                    Page::Dashboard,
                    Page::Vms,
                    Page::Snapshots,
                    Page::Network,
                    Page::Disks,
                    Page::Shot,
                    Page::Detonate,
                    Page::Swarm,
                    Page::Sandbox,
                    Page::Commands,
                    Page::Help,
                ] {
                    let is_selected = self.page == p;
                    let btn = if is_selected {
                        ui.button(RichText::new(p.label()).strong().color(ACCENT_CYAN))
                    } else {
                        ui.button(p.label())
                    };
                    if btn.clicked() {
                        self.page = p;
                    }
                }
            });
            ui.separator();
        }

        match self.page {
            Page::Dashboard => {
                ui.horizontal(|ui| {
                    ui.heading("VM Overview");
                    ui.label(format!("({} VMs detected)", self.vms.len()));
                });

                if self.vms.is_empty() {
                    if self.pending_tasks > 0 {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading VMs from hypervisors…");
                        });
                    } else {
                        ui.label("No virtual machines registered. Click '⟳ Refresh' or create a VM in the 'VMs' tab.");
                    }
                } else {
                    let mut action_cmd: Option<(&'static str, Command)> = None;
                    let mut select_vm: Option<String> = None;

                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .show(ui, |ui| {
                            egui::Grid::new("vm_dashboard_grid")
                                .striped(true)
                                .spacing([16.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("VM Name").strong());
                                    ui.label(RichText::new("Provider").strong());
                                    ui.label(RichText::new("UUID").strong());
                                    ui.label(RichText::new("Actions").strong());
                                    ui.end_row();

                                    for vm in &self.vms {
                                        ui.label(
                                            RichText::new(&vm.name)
                                                .color(egui::Color32::from_rgb(220, 230, 245))
                                                .strong(),
                                        );
                                        if vm.provider == "hyperv" {
                                            render_badge(
                                                ui,
                                                "Hyper-V",
                                                egui::Color32::from_rgb(8, 51, 68),
                                                ACCENT_CYAN,
                                            );
                                        } else {
                                            render_badge(
                                                ui,
                                                "VirtualBox",
                                                egui::Color32::from_rgb(67, 20, 7),
                                                ACCENT_AMBER,
                                            );
                                        }
                                        ui.label(RichText::new(&vm.uuid).weak());
                                        ui.horizontal(|ui| {
                                            if ui.button("Select").clicked() {
                                                select_vm = Some(vm.name.clone());
                                            }
                                            if vm.provider == "hyperv" {
                                                if ui.button("Start").clicked() {
                                                    select_vm = Some(vm.name.clone());
                                                    action_cmd = Some((
                                                        "Hyper-V Start",
                                                        Command::HypervStart {
                                                            vm: vm.name.clone(),
                                                            confirm: self.confirm(),
                                                        },
                                                    ));
                                                }
                                                if ui.button("Stop").clicked() {
                                                    select_vm = Some(vm.name.clone());
                                                    action_cmd = Some((
                                                        "Hyper-V Stop",
                                                        Command::HypervStop {
                                                            vm: vm.name.clone(),
                                                            force: false,
                                                            confirm: self.confirm(),
                                                        },
                                                    ));
                                                }
                                            } else {
                                                if ui.button("Info").clicked() {
                                                    select_vm = Some(vm.name.clone());
                                                    action_cmd = Some((
                                                        "VM Info",
                                                        Command::VmInfo {
                                                            vm: vm.name.clone(),
                                                        },
                                                    ));
                                                }
                                                if ui.button("Start").clicked() {
                                                    select_vm = Some(vm.name.clone());
                                                    action_cmd = Some((
                                                        "VM Start",
                                                        Command::VmStart {
                                                            vm: vm.name.clone(),
                                                            confirm: self.confirm(),
                                                        },
                                                    ));
                                                }
                                                if ui.button("Stop").clicked() {
                                                    select_vm = Some(vm.name.clone());
                                                    action_cmd = Some((
                                                        "VM Stop",
                                                        Command::VmStop {
                                                            vm: vm.name.clone(),
                                                            force: false,
                                                            confirm: self.confirm(),
                                                        },
                                                    ));
                                                }
                                            }
                                        });
                                        ui.end_row();
                                    }
                                });
                        });

                    if let Some(target) = select_vm {
                        self.vm_name = target;
                    }
                    if let Some((title, cmd)) = action_cmd {
                        self.spawn_command(title, cmd);
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Activity / Output").strong());
                    if ui.button("Clear").clicked() {
                        self.vm_output.clear();
                    }
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.vm_output)
                            .desired_rows(12)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
            Page::Vms => {
                ui.heading("Virtual Machines Management");

                if !self.vms.is_empty() {
                    let mut quick_select: Option<String> = None;
                    ui.horizontal(|ui| {
                        ui.label("Quick Select:");
                        for vm in &self.vms {
                            let is_selected = self.vm_name == vm.name;
                            if ui.selectable_label(is_selected, &vm.name).clicked() {
                                quick_select = Some(vm.name.clone());
                            }
                        }
                    });
                    if let Some(target) = quick_select {
                        self.vm_name = target;
                    }
                    ui.separator();
                }

                ui.horizontal(|ui| {
                    ui.label("Target VM:");
                    ui.text_edit_singleline(&mut self.vm_name);
                    ui.label("OS Type:");
                    ui.text_edit_singleline(&mut self.ostype);
                    ui.label("Memory (MB):");
                    ui.text_edit_singleline(&mut self.memory_mb);
                });

                ui.horizontal(|ui| {
                    if ui.button("⟳ List VMs").clicked() {
                        self.spawn_vms();
                    }
                    if ui.button("ℹ Info").clicked() {
                        let name = self.vm_name.clone();
                        self.spawn_command("VM Info", Command::VmInfo { vm: name });
                    }
                    if ui.button("➕ Create").clicked() {
                        let mem = self.memory_mb.trim().parse::<u32>().unwrap_or(4096);
                        self.spawn_command(
                            "VM Create",
                            Command::VmCreate {
                                vm: self.vm_name.clone(),
                                ostype: self.ostype.clone(),
                                memory_mb: mem,
                                confirm: self.confirm(),
                            },
                        );
                    }
                    if ui.button("▶ Start").clicked() {
                        self.spawn_command(
                            "VM Start",
                            Command::VmStart {
                                vm: self.vm_name.clone(),
                                confirm: self.confirm(),
                            },
                        );
                    }
                    if ui.button("⏹ Stop").clicked() {
                        self.spawn_command(
                            "VM Stop",
                            Command::VmStop {
                                vm: self.vm_name.clone(),
                                force: false,
                                confirm: self.confirm(),
                            },
                        );
                    }
                    if ui.button("🗑 Delete").clicked() {
                        self.spawn_command(
                            "VM Delete",
                            Command::VmDelete {
                                vm: self.vm_name.clone(),
                                confirm: self.confirm(),
                            },
                        );
                    }
                });

                ui.separator();
                ui.label(RichText::new("Command Output:").strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.vm_output)
                            .desired_rows(14)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
            Page::Snapshots => {
                ui.heading("Snapshot Management");
                ui.horizontal(|ui| {
                    ui.label("Target VM:");
                    ui.text_edit_singleline(&mut self.vm_name);
                    ui.label("Snapshot Name:");
                    ui.text_edit_singleline(&mut self.snap_name);
                });
                ui.horizontal(|ui| {
                    if ui.button("List Snapshots").clicked() {
                        let name = self.vm_name.clone();
                        self.spawn_command("Snapshot List", Command::SnapshotList { vm: name });
                    }
                    if ui.button("Take Snapshot").clicked() {
                        self.spawn_command(
                            "Snapshot Create",
                            Command::SnapshotCreate {
                                vm: self.vm_name.clone(),
                                snap: self.snap_name.clone(),
                                confirm: self.confirm(),
                            },
                        );
                    }
                    if ui.button("Restore Snapshot").clicked() {
                        self.spawn_command(
                            "Snapshot Restore",
                            Command::SnapshotRestore {
                                vm: self.vm_name.clone(),
                                snap: self.snap_name.clone(),
                                confirm: self.confirm(),
                            },
                        );
                    }
                    if ui.button("Delete Snapshot").clicked() {
                        self.spawn_command(
                            "Snapshot Delete",
                            Command::SnapshotDelete {
                                vm: self.vm_name.clone(),
                                snap: self.snap_name.clone(),
                                confirm: self.confirm(),
                            },
                        );
                    }
                });

                if !self.snapshots.is_empty() {
                    ui.separator();
                    ui.label(RichText::new(format!("Snapshots for '{}':", self.vm_name)).strong());
                    let mut snap_cmd: Option<(&'static str, Command)> = None;
                    egui::ScrollArea::vertical()
                        .max_height(160.0)
                        .show(ui, |ui| {
                            egui::Grid::new("snapshots_grid")
                                .striped(true)
                                .spacing([16.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Name").strong());
                                    ui.label(RichText::new("UUID").strong());
                                    ui.label(RichText::new("Description").strong());
                                    ui.label(RichText::new("Actions").strong());
                                    ui.end_row();

                                    for s in &self.snapshots {
                                        ui.label(
                                            RichText::new(&s.name)
                                                .color(egui::Color32::from_rgb(220, 230, 245)),
                                        );
                                        ui.label(RichText::new(&s.uuid).weak());
                                        ui.label(RichText::new(&s.description).weak());
                                        ui.horizontal(|ui| {
                                            if ui.button("Restore").clicked() {
                                                snap_cmd = Some((
                                                    "Snapshot Restore",
                                                    Command::SnapshotRestore {
                                                        vm: self.vm_name.clone(),
                                                        snap: s.name.clone(),
                                                        confirm: self.confirm(),
                                                    },
                                                ));
                                            }
                                            if ui.button("Delete").clicked() {
                                                snap_cmd = Some((
                                                    "Snapshot Delete",
                                                    Command::SnapshotDelete {
                                                        vm: self.vm_name.clone(),
                                                        snap: s.name.clone(),
                                                        confirm: self.confirm(),
                                                    },
                                                ));
                                            }
                                        });
                                        ui.end_row();
                                    }
                                });
                        });
                    if let Some((title, cmd)) = snap_cmd {
                        self.spawn_command(title, cmd);
                    }
                }

                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.vm_output)
                            .desired_rows(12)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
            Page::Network => {
                ui.heading("Host-Only Network Management");
                ui.horizontal(|ui| {
                    ui.label("IP:");
                    ui.text_edit_singleline(&mut self.net_ip);
                    ui.label("Mask:");
                    ui.text_edit_singleline(&mut self.netmask);
                    ui.label("Interface:");
                    ui.text_edit_singleline(&mut self.net_name);
                });
                ui.horizontal(|ui| {
                    if ui.button("List Networks").clicked() {
                        self.spawn_command("Network List", Command::NetworkList);
                    }
                    if ui.button("Create Host-Only Interface").clicked() {
                        self.spawn_command(
                            "Network Create",
                            Command::NetworkCreate {
                                ip: self.net_ip.clone(),
                                netmask: self.netmask.clone(),
                                confirm: self.confirm(),
                            },
                        );
                    }
                    if ui.button("Remove Interface").clicked() {
                        self.spawn_command(
                            "Network Remove",
                            Command::NetworkRemove {
                                net: self.net_name.clone(),
                                confirm: self.confirm(),
                            },
                        );
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.vm_output)
                            .desired_rows(14)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
            Page::Disks => {
                ui.heading("Disk Management");
                ui.horizontal(|ui| {
                    ui.label("VDI Path:");
                    ui.text_edit_singleline(&mut self.disk_path);
                    ui.label("Size (GB):");
                    ui.text_edit_singleline(&mut self.disk_gb);
                });
                ui.horizontal(|ui| {
                    if ui.button("List Hard Disks").clicked() {
                        self.spawn_command("Disk List", Command::DiskList);
                    }
                    if ui.button("Create VDI Disk").clicked() {
                        let gb = self.disk_gb.trim().parse::<u32>().unwrap_or(20);
                        self.spawn_command(
                            "Disk Create",
                            Command::DiskCreate {
                                path: self.disk_path.clone(),
                                size_gb: gb,
                                confirm: self.confirm(),
                            },
                        );
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.vm_output)
                            .desired_rows(14)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
            Page::Shot => {
                ui.heading("VM Screenshot Capture");
                ui.horizontal(|ui| {
                    ui.label("Target VM:");
                    ui.text_edit_singleline(&mut self.vm_name);
                    ui.label("Output PNG Path:");
                    ui.text_edit_singleline(&mut self.shot_out);
                    if ui.button("Capture Screenshot").clicked() {
                        let out_path = if self.shot_out.trim().is_empty() {
                            let p = std::env::temp_dir()
                                .join("vmcraft-screenshot.png")
                                .to_string_lossy()
                                .to_string();
                            self.shot_out = p.clone();
                            p
                        } else {
                            self.shot_out.clone()
                        };
                        self.spawn_command(
                            "VM Screenshot",
                            Command::VmShot {
                                vm: self.vm_name.clone(),
                                out: out_path,
                            },
                        );
                    }
                });
                ui.separator();
                if let Some(ref img) = self.shot_image {
                    ui.label(RichText::new("Captured Framebuffer:").strong());
                    if self.shot_texture.is_none() {
                        self.shot_texture = Some(ui.ctx().load_texture(
                            "vm_framebuffer",
                            img.clone(),
                            Default::default(),
                        ));
                    } else if let Some(ref mut tex) = self.shot_texture {
                        tex.set(img.clone(), Default::default());
                    }
                    if let Some(ref tex) = self.shot_texture {
                        egui::ScrollArea::both().max_height(320.0).show(ui, |ui| {
                            ui.image(tex);
                        });
                    }
                    ui.separator();
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.vm_output)
                            .desired_rows(8)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
            Page::Detonate => {
                ui.heading("Air-Gapped Clean-Room Detonation & Triage");
                ui.label(
                    "Safely detonate untrusted keygens/patches inside an air-gapped guest (network cut), execute disassembler inside guest on target binary, extract clean disassembly, and auto-nuke & revert."
                );
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Target VM:");
                    ui.text_edit_singleline(&mut self.vm_name);
                    ui.checkbox(&mut self.detonate_airgap, "Enforce Hard Airgap (0 egress)");
                });

                ui.horizontal(|ui| {
                    ui.label("Untrusted Keygen / Sample:");
                    ui.text_edit_singleline(&mut self.detonate_keygen);
                });

                ui.horizontal(|ui| {
                    ui.label("Target Binary to Disassemble:");
                    ui.text_edit_singleline(&mut self.detonate_target);
                });

                ui.horizontal(|ui| {
                    ui.label("In-Guest Disassembler:");
                    ui.text_edit_singleline(&mut self.detonate_disasm_exe);
                    ui.label("Guest Output:");
                    ui.text_edit_singleline(&mut self.detonate_guest_out);
                });

                ui.horizontal(|ui| {
                    ui.label("Guest User:");
                    ui.text_edit_singleline(&mut self.detonate_user);
                    ui.label("Guest Password:");
                    ui.text_edit_singleline(&mut self.detonate_pass);
                    ui.label("Host Destination:");
                    ui.text_edit_singleline(&mut self.detonate_host_out);
                });

                ui.horizontal(|ui| {
                    if ui.button("💥 Detonate & Disassemble in Airgap").clicked() {
                        let shot_path = std::env::temp_dir()
                            .join("vmcraft-detonate.png")
                            .to_string_lossy()
                            .to_string();
                        self.shot_out = shot_path.clone();
                        self.spawn_command(
                            "Pipeline Detonate",
                            Command::PipelineDetonate {
                                vm: self.vm_name.clone(),
                                user: self.detonate_user.clone(),
                                pass: self.detonate_pass.clone(),
                                keygen: if self.detonate_keygen.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.detonate_keygen.clone())
                                },
                                target_binary: self.detonate_target.clone(),
                                disassembler_exe: self.detonate_disasm_exe.clone(),
                                disassembler_args: vec![
                                    "-B".to_string(),
                                    format!("-o{}", self.detonate_guest_out),
                                    "C:\\detonate\\target.bin".to_string(),
                                ],
                                guest_output: self.detonate_guest_out.clone(),
                                host_output: self.detonate_host_out.clone(),
                                screenshot: Some(shot_path),
                                confirm: self.confirm(),
                            },
                        );
                    }

                    if ui.button("✂ Sever Network (Airgap Now)").clicked() {
                        self.spawn_command(
                            "Airgap Sever",
                            Command::AirgapSet {
                                vm: self.vm_name.clone(),
                                disconnect: true,
                            },
                        );
                    }

                    if ui.button("🔌 Reconnect Network").clicked() {
                        self.spawn_command(
                            "Airgap Reconnect",
                            Command::AirgapSet {
                                vm: self.vm_name.clone(),
                                disconnect: false,
                            },
                        );
                    }

                    if ui.button("🚨 Emergency Nuke & Purge VM").clicked() {
                        self.spawn_command(
                            "Emergency Nuke",
                            Command::VmStop {
                                vm: self.vm_name.clone(),
                                force: true,
                                confirm: self.confirm(),
                            },
                        );
                    }
                });

                ui.separator();
                // Interactive split: Telemetry on left/top, Disassembly Output on right/bottom
                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.label(RichText::new("Detonation Progress & Telemetry:").strong());
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.vm_output)
                                    .desired_rows(14)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
                    });

                    cols[1].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Extracted Disassembly Studio:").strong());
                            if ui.button("📂 Load from Host Output").clicked() {
                                if let Ok(content) =
                                    std::fs::read_to_string(&self.detonate_host_out)
                                {
                                    self.disasm_code = content;
                                }
                            }
                        });
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.disasm_code)
                                    .desired_rows(14)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
                    });
                });
            }
            Page::Swarm => {
                ui.heading("Firecracker Micro-VMs (WSL2 /dev/kvm)");
                ui.label(
                    "Probes, spawns, lists, and terminates Firecracker micro-VM processes inside WSL2.",
                );
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("🔍 Probe KVM & Firecracker").clicked() {
                        self.spawn_command("Firecracker Probe", Command::FirecrackerProbe);
                    }
                    if ui.button("📋 List Active Micro-VMs").clicked() {
                        self.spawn_command("Firecracker List", Command::FirecrackerList);
                    }
                    if let Some(kvm) = self.swarm_kvm_available {
                        let (bg, fg, label) = if kvm {
                            (
                                egui::Color32::from_rgb(6, 78, 59),
                                ACCENT_EMERALD,
                                "/dev/kvm: available",
                            )
                        } else {
                            (
                                egui::Color32::from_rgb(127, 29, 29),
                                egui::Color32::from_rgb(254, 202, 202),
                                "/dev/kvm: unavailable",
                            )
                        };
                        render_badge(ui, label, bg, fg);
                    }
                    if let Some(installed) = self.swarm_fc_installed {
                        let (bg, fg, label) = if installed {
                            (
                                egui::Color32::from_rgb(8, 51, 68),
                                ACCENT_CYAN,
                                "firecracker: installed",
                            )
                        } else {
                            (
                                egui::Color32::from_rgb(67, 20, 7),
                                ACCENT_AMBER,
                                "firecracker: not found",
                            )
                        };
                        render_badge(ui, label, bg, fg);
                    }
                });

                if let Some(ref reason) = self.swarm_probe_reason {
                    ui.colored_label(ACCENT_AMBER, format!("Probe note: {reason}"));
                }

                ui.horizontal(|ui| {
                    ui.label("VM ID:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.swarm_spawn_id).desired_width(120.0),
                    );
                    ui.label("Kernel:");
                    ui.text_edit_singleline(&mut self.swarm_kernel);
                    ui.label("Rootfs:");
                    ui.text_edit_singleline(&mut self.swarm_rootfs);
                    if ui.button("🚀 Spawn Micro-VM").clicked() {
                        self.spawn_command(
                            "Firecracker Spawn",
                            Command::FirecrackerSpawn {
                                vm_id: self.swarm_spawn_id.clone(),
                                vcpu_count: 1,
                                mem_size_mib: 128,
                                kernel_path: self.swarm_kernel.clone(),
                                rootfs_path: self.swarm_rootfs.clone(),
                                boot_args: None,
                                confirm: self.confirm(),
                            },
                        );
                    }
                    if ui.button("🛑 Kill VM ID").clicked() {
                        self.spawn_command(
                            "Firecracker Kill",
                            Command::FirecrackerKill {
                                vm_id: self.swarm_spawn_id.clone(),
                                confirm: self.confirm(),
                            },
                        );
                    }
                });

                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "Active Firecracker Processes ({}):",
                        self.swarm_instances.len()
                    ))
                    .strong(),
                );

                if self.swarm_instances.is_empty() {
                    ui.label(
                        "No active Firecracker processes reported. Click 'List Active Micro-VMs' to query WSL2.",
                    );
                } else {
                    let mut clicked_idx: Option<usize> = None;
                    egui::Grid::new("firecracker_instances_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(RichText::new("PID").strong());
                            ui.label(RichText::new("Binary").strong());
                            ui.label(RichText::new("Action").strong());
                            ui.end_row();
                            for (idx, inst) in self.swarm_instances.iter().enumerate() {
                                let selected = self.swarm_selected_idx == Some(idx);
                                if ui.selectable_label(selected, &inst.pid).clicked() {
                                    clicked_idx = Some(idx);
                                }
                                ui.label(&inst.binary);
                                ui.label("running");
                                ui.end_row();
                            }
                        });
                    if let Some(idx) = clicked_idx {
                        self.swarm_selected_idx = Some(idx);
                    }
                }

                ui.separator();
                ui.label(RichText::new("Telemetry & Swarm Audit Output:").strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.vm_output)
                            .desired_rows(6)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
            Page::Sandbox => {
                ui.heading("Ephemeral Sandbox Execution");
                ui.label("Executes Python or PowerShell in an isolated 30s timeout sandbox.");
                ui.add(
                    egui::TextEdit::multiline(&mut self.sandbox_code)
                        .desired_rows(6)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
                if ui.button("▶ Run Sandbox").clicked() {
                    let code = self.sandbox_code.clone();
                    self.spawn_command(
                        "Sandbox Run",
                        Command::SandboxRun {
                            lang: "python".to_string(),
                            code,
                        },
                    );
                }
                ui.separator();
                ui.label(RichText::new("Sandbox Output:").strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.sandbox_output)
                            .desired_rows(8)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
            Page::Commands => {
                ui.heading("Craft Command Bus Registry & Interactive Runner");
                ui.label(
                    "Deterministic semantic command bus per mcp-central-docs/standards/rust-artcraft/CRAFT_COMMAND_BUS.md",
                );
                ui.separator();

                let mut run_cmd_opt: Option<(&'static str, Command)> = None;

                ui.collapsing("Interactive Bus Dispatch", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Command ID:");
                        ui.text_edit_singleline(&mut self.bus_cmd);
                        ui.label("Params (JSON):");
                        ui.text_edit_singleline(&mut self.bus_params);
                        if ui.button("▶ Dispatch Bus").clicked() {
                            let params_val: serde_json::Value =
                                serde_json::from_str(&self.bus_params)
                                    .unwrap_or(serde_json::json!({}));
                            match vmcraft_core::parse_command(&self.bus_cmd, &params_val) {
                                Ok(cmd) => {
                                    run_cmd_opt = Some(("Bus Command", cmd));
                                }
                                Err(e) => {
                                    self.vm_output = format!("=== Bus Error ===\n{e}");
                                }
                            }
                        }
                    });
                });

                ui.separator();
                ui.label(RichText::new("Registered Commands Catalog:").strong());

                let catalog = vmcraft_core::commands_catalog();
                if let Some(arr) = catalog.as_array() {
                    egui::ScrollArea::vertical()
                        .max_height(280.0)
                        .show(ui, |ui| {
                            egui::Grid::new("bus_catalog_grid")
                                .striped(true)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("ID").strong());
                                    ui.label(RichText::new("Label").strong());
                                    ui.label(RichText::new("Description").strong());
                                    ui.label(RichText::new("Action").strong());
                                    ui.end_row();

                                    for item in arr {
                                        let id =
                                            item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                        let label = item
                                            .get("label")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let desc = item
                                            .get("description")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");

                                        ui.label(
                                            RichText::new(id)
                                                .monospace()
                                                .color(egui::Color32::from_rgb(180, 210, 255)),
                                        );
                                        ui.label(label);
                                        ui.label(RichText::new(desc).weak());

                                        let id_string = id.to_string();
                                        if ui.button("Populate").clicked() {
                                            self.bus_cmd = id_string;
                                            self.bus_params = "{}".to_string();
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                }

                if let Some((title, cmd)) = run_cmd_opt {
                    self.spawn_command(title, cmd);
                }

                ui.separator();
                ui.label(RichText::new("Execution Output:").strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.vm_output)
                            .desired_rows(8)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
            Page::Help => {
                ui.heading("vmcraft Help Center");
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
                        let selected = self.help_tab == tab;
                        let btn = if selected {
                            ui.button(RichText::new(tab.label()).strong().color(ACCENT_CYAN))
                        } else {
                            ui.button(tab.label())
                        };
                        if btn.clicked() {
                            self.help_tab = tab;
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| match self.help_tab {
                    HelpTab::GetStarted => {
                        ui.label(RichText::new("1. Launch & Discovery").strong());
                        ui.label("Status auto-polls VirtualBox and Hyper-V on startup. Click Refresh (F5) anytime.");
                        ui.label(RichText::new("2. Select a VM").strong());
                        ui.label("Open Dashboard and click Select on any VM (e.g. ubu-2404-lts) to target it across tabs.");
                        ui.label(RichText::new("3. Create a Baseline Snapshot").strong());
                        ui.label("Open Snapshots, enter a name (e.g. clean-base), enable 'allow mutate', and click Create.");
                        ui.label(RichText::new("4. Run & Inspect").strong());
                        ui.label("Use VMs, Shot, Detonate, Swarm, Sandbox, or Commands (Ctrl+K) to operate the guest.");
                        ui.label(RichText::new("5. Roll Back").strong());
                        ui.label("Restore your baseline snapshot when done to keep the VM pristine.");
                    }
                    HelpTab::Functions => {
                        ui.label("• Dashboard & VMs: VirtualBox 7+ and Hyper-V discovery, inspect, start, stop, create, delete.");
                        ui.label("• Snapshots: List, create, restore, and delete VirtualBox snapshots.");
                        ui.label("• Network & Disks: Host-only interfaces and .vdi image inspection/creation.");
                        ui.label("• Shot: Live framebuffer PNG capture rendered in-app.");
                        ui.label("• Detonate: Airgap link severing + guestcontrol execution + snapshot rollback.");
                        ui.label("• Swarm: WSL2 /dev/kvm Firecracker probe, process list, spawn, and kill.");
                        ui.label("• Sandbox: Isolated 30s Python/PowerShell runner.");
                        ui.label("• Commands: 32-operation ArtCraft Command Bus explorer and dispatcher.");
                    }
                    HelpTab::Usage => {
                        ui.label(RichText::new("GUI").strong());
                        ui.label("Use the top menu bar (Session, Edit, View, Command, Window, Help) or tab bar. Ctrl+K opens the Command Palette; F1 opens Keyboard Shortcuts.");
                        ui.separator();
                        ui.label(RichText::new("CLI (vmcraft-mcp)").strong());
                        ui.monospace("vmcraft-mcp status\nvmcraft-mcp vm list\nvmcraft-mcp vm info --vm ubu-2404-lts\nvmcraft-mcp pipeline naked-test --repo owner/repo --baseline ubu-2404-lts --snapshot clean-desktop-2026-11-22 --user user --pass user --linux --yes");
                        ui.separator();
                        ui.label(RichText::new("MCP stdio").strong());
                        ui.monospace("vmcraft-mcp mcp");
                    }
                    HelpTab::Tips => {
                        ui.label("• Always create a clean snapshot before running untrusted binaries or installers.");
                        ui.label("• Use Ctrl+K to search and populate any of the 32 Command Bus verbs immediately.");
                        ui.label("• In CLI `guest exec`, pass hyphenated guest args with `=` syntax: `--args=-lc`.");
                        ui.label("• Press Ctrl+1 (or View -> Reset Zoom) to reset UI scaling to 100%.");
                    }
                    HelpTab::Safety => {
                        ui.label(RichText::new("Mutating Operations Safety Gate").strong());
                        ui.label("State-changing verbs (vm create/start/stop/delete, snapshot create/restore/delete, network create/remove, airgap, guest.exec, pipeline.detonate, pipeline.naked_install, firecracker spawn/kill) require checking 'allow mutate' in the toolbar, passing `--yes`, or setting `VMCRAFT_ALLOW_MUTATE=1`.");
                        ui.separator();
                        ui.label("• Guest Additions must be running inside the guest for `guestcontrol` and `pipeline.naked_install`.");
                        ui.label("• Firecracker commands require `wsl.exe` with `/dev/kvm` and `firecracker` installed inside WSL2.");
                    }
                    HelpTab::Planned => {
                        ui.label("• Native QEMU/KVM (Linux) and Apple Virtualization.framework (macOS) providers.");
                        ui.label("• Live GUI `--control PORT` bridge.");
                        ui.label("• Migrate `vmcraft-core/src/llm.rs` to shared `craft-ai-core`.");
                    }
                    HelpTab::OtherCrafts => {
                        ui.label("• devcraft: Fleet orchestration and clean-room `naked-test` consumer install verification.");
                        ui.label("• wincraft: Windows host diagnostics; vmcraft handles VM/hypervisor isolation.");
                        ui.label("• craft-launcher: Launches and manages installed Craft workstations.");
                    }
                });
            }
        }

        // Command Palette modal (Ctrl+K / Ctrl+Shift+P)
        if self.show_palette {
            let mut open = self.show_palette;
            let mut exec_cmd: Option<(String, Command)> = None;
            egui::Window::new("Command Palette")
                .open(&mut open)
                .collapsible(false)
                .resizable(true)
                .default_width(520.0)
                .show(ui.ctx(), |ui| {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.palette_query)
                            .hint_text("Type a command ID or keyword (e.g. vm.list, firecracker, snapshot)...")
                            .desired_width(f32::INFINITY),
                    );
                    resp.request_focus();
                    let matches = filter_palette_commands(&self.palette_query);
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if enter_pressed {
                        if let Some((id, _, _)) = matches.first() {
                            if let Ok(cmd) = vmcraft_core::parse_command(id, &serde_json::json!({}))
                            {
                                exec_cmd = Some((format!("Palette: {id}"), cmd));
                                self.show_palette = false;
                            } else {
                                self.bus_cmd = id.clone();
                                self.bus_params = "{}".to_string();
                                self.page = Page::Commands;
                                self.show_palette = false;
                            }
                        }
                    }
                    ui.separator();
                    egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                        for (id, label, desc) in matches.into_iter().take(20) {
                            if ui
                                .selectable_label(false, format!("{id}  -  {label} ({desc})"))
                                .clicked()
                            {
                                self.bus_cmd = id;
                                self.bus_params = "{}".to_string();
                                self.page = Page::Commands;
                                self.show_palette = false;
                            }
                        }
                    });
                });
            if !open {
                self.show_palette = false;
            }
            if let Some((title, cmd)) = exec_cmd {
                self.spawn_command(title, cmd);
            }
        }

        // Keyboard Shortcuts modal (F1 / Ctrl+/)
        if self.show_shortcuts {
            let mut open = self.show_shortcuts;
            egui::Window::new("Keyboard Shortcuts")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    egui::Grid::new("shortcuts_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Ctrl+K / Ctrl+Shift+P").strong());
                            ui.label("Open Command Palette");
                            ui.end_row();
                            ui.label(RichText::new("F1 / Ctrl+/").strong());
                            ui.label("Open Keyboard Shortcuts");
                            ui.end_row();
                            ui.label(RichText::new("F5").strong());
                            ui.label("Refresh Hypervisors & VM List");
                            ui.end_row();
                            ui.label(RichText::new("Ctrl+MouseWheel").strong());
                            ui.label("Zoom In / Out");
                            ui.end_row();
                            ui.label(RichText::new("Ctrl+1").strong());
                            ui.label("Reset Zoom (100%)");
                            ui.end_row();
                            ui.label(RichText::new("Ctrl+B").strong());
                            ui.label("Show / Hide Tab Bar");
                            ui.end_row();
                            ui.label(RichText::new("Ctrl+Q").strong());
                            ui.label("Quit Application");
                            ui.end_row();
                        });
                });
            self.show_shortcuts = open;
        }

        // Licenses & Notices modal
        if self.show_licenses {
            let mut open = self.show_licenses;
            egui::Window::new("Licenses & Notices")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Dual-licensed under MIT OR Apache-2.0.");
                    ui.label("Zero-unwrap / zero-panic workspace clippy policy enforced.");
                    ui.label("Independent community ArtCraft workstation (not affiliated with Storyteller / Storytold).");
                });
            self.show_licenses = open;
        }

        // About modal
        if self.show_about {
            let mut open = self.show_about;
            egui::Window::new("About vmcraft")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.heading("vmcraft v0.1.0");
                    ui.label("Pure-Rust ArtCraft VM Workstation (L1 Core + L2 CLI/MCP + L3 egui).");
                    ui.label("Supports VirtualBox 7+, Windows Hyper-V, and WSL2 Firecracker KVM.");
                });
            self.show_about = open;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_labels_are_distinct() {
        let pages = [
            Page::Dashboard,
            Page::Vms,
            Page::Snapshots,
            Page::Network,
            Page::Disks,
            Page::Shot,
            Page::Detonate,
            Page::Swarm,
            Page::Sandbox,
            Page::Commands,
            Page::Help,
        ];
        for (i, p1) in pages.iter().enumerate() {
            for (j, p2) in pages.iter().enumerate() {
                if i != j {
                    assert_ne!(p1.label(), p2.label());
                }
            }
        }
    }

    #[test]
    fn vmlist_message_parses_into_items() {
        let (tx, rx) = mpsc::channel();
        let mut app = VmApp {
            rt: None,
            rt_error: None,
            tx: tx.clone(),
            rx,
            pending_tasks: 1,
            page: Page::Dashboard,
            status: "starting…".to_string(),
            host_info: None,
            vms: Vec::new(),
            snapshots: Vec::new(),
            shot_image: None,
            shot_texture: None,
            bus_cmd: "vm.list".to_string(),
            bus_params: "{}".to_string(),
            vm_output: String::new(),
            vm_name: String::new(),
            ostype: "Ubuntu_64".to_string(),
            memory_mb: "4096".to_string(),
            snap_name: String::new(),
            net_ip: "192.168.56.1".to_string(),
            netmask: "255.255.255.0".to_string(),
            net_name: String::new(),
            disk_path: String::new(),
            disk_gb: "20".to_string(),
            shot_out: String::new(),
            sandbox_code: String::new(),
            sandbox_output: String::new(),
            mutate_confirm: false,
            dark_mode: true,
            airgap_active: false,
            disasm_code: String::new(),
            detonate_keygen: String::new(),
            detonate_target: String::new(),
            detonate_disasm_exe: "ida64.exe".to_string(),
            detonate_user: String::new(),
            detonate_pass: String::new(),
            detonate_guest_out: "C:\\detonate\\out.asm".to_string(),
            detonate_host_out: "out.asm".to_string(),
            detonate_airgap: true,
            swarm_instances: Vec::new(),
            swarm_selected_idx: None,
            swarm_spawn_id: "microvm-1".to_string(),
            swarm_kernel: "/vmlinux".to_string(),
            swarm_rootfs: "/rootfs.ext4".to_string(),
            swarm_kvm_available: None,
            swarm_fc_installed: None,
            swarm_probe_reason: None,
            help_tab: HelpTab::GetStarted,
            show_palette: false,
            palette_query: String::new(),
            show_shortcuts: false,
            show_about: false,
            show_licenses: false,
            show_tab_bar: true,
            status_bar_pos: StatusBarPos::Bottom,
        };

        let sample_json = serde_json::json!({
            "status": "ok",
            "vms": [
                { "name": "ubu-2404-lts", "uuid": "{ef4f7178-1087-42a5-9539-e6d37a0c5616}", "provider": "virtualbox" },
                { "name": "podman-machine-default", "uuid": "9154eea3-78ed-4191-91fa-b16dc8597949", "provider": "hyperv" }
            ]
        });

        let _ = tx.send(AppMessage::VmList(sample_json));
        app.drain();

        assert_eq!(app.vms.len(), 2);
        assert_eq!(app.vms[0].name, "ubu-2404-lts");
        assert_eq!(app.vms[0].provider, "virtualbox");
        assert_eq!(app.vms[1].name, "podman-machine-default");
        assert_eq!(app.vms[1].provider, "hyperv");
        assert_eq!(app.vm_name, "ubu-2404-lts");
        assert_eq!(app.pending_tasks, 0);
    }

    #[test]
    fn firecracker_probe_and_list_populate_state() {
        let (tx, rx) = mpsc::channel();
        let mut app = VmApp {
            rt: None,
            rt_error: None,
            tx: tx.clone(),
            rx,
            pending_tasks: 2,
            page: Page::Swarm,
            status: String::new(),
            host_info: None,
            vms: Vec::new(),
            snapshots: Vec::new(),
            shot_image: None,
            shot_texture: None,
            bus_cmd: "vm.list".to_string(),
            bus_params: "{}".to_string(),
            vm_output: String::new(),
            vm_name: String::new(),
            ostype: "Ubuntu_64".to_string(),
            memory_mb: "4096".to_string(),
            snap_name: String::new(),
            net_ip: "192.168.56.1".to_string(),
            netmask: "255.255.255.0".to_string(),
            net_name: String::new(),
            disk_path: String::new(),
            disk_gb: "20".to_string(),
            shot_out: String::new(),
            sandbox_code: String::new(),
            sandbox_output: String::new(),
            mutate_confirm: false,
            dark_mode: true,
            airgap_active: false,
            disasm_code: String::new(),
            detonate_keygen: String::new(),
            detonate_target: String::new(),
            detonate_disasm_exe: "ida64.exe".to_string(),
            detonate_user: String::new(),
            detonate_pass: String::new(),
            detonate_guest_out: "C:\\detonate\\out.asm".to_string(),
            detonate_host_out: "out.asm".to_string(),
            detonate_airgap: true,
            swarm_instances: Vec::new(),
            swarm_selected_idx: None,
            swarm_spawn_id: "microvm-1".to_string(),
            swarm_kernel: "/vmlinux".to_string(),
            swarm_rootfs: "/rootfs.ext4".to_string(),
            swarm_kvm_available: None,
            swarm_fc_installed: None,
            swarm_probe_reason: None,
            help_tab: HelpTab::GetStarted,
            show_palette: false,
            palette_query: String::new(),
            show_shortcuts: false,
            show_about: false,
            show_licenses: false,
            show_tab_bar: true,
            status_bar_pos: StatusBarPos::Bottom,
        };

        let _ = tx.send(AppMessage::CommandResult {
            title: "Firecracker Probe".to_string(),
            value: serde_json::json!({
                "status": "ok",
                "kvm_available": true,
                "installed": false,
            }),
        });
        let _ = tx.send(AppMessage::CommandResult {
            title: "Firecracker List".to_string(),
            value: serde_json::json!({
                "status": "ok",
                "instances": [
                    { "pid": "4242", "binary": "/usr/local/bin/firecracker", "provider": "firecracker" }
                ]
            }),
        });
        app.drain();

        assert_eq!(app.swarm_kvm_available, Some(true));
        assert_eq!(app.swarm_fc_installed, Some(false));
        assert_eq!(app.swarm_instances.len(), 1);
        assert_eq!(app.swarm_instances[0].pid, "4242");
        assert_eq!(app.pending_tasks, 0);
    }

    #[test]
    fn help_tabs_have_distinct_labels() {
        let tabs = [
            HelpTab::GetStarted,
            HelpTab::Functions,
            HelpTab::Usage,
            HelpTab::Tips,
            HelpTab::Safety,
            HelpTab::Planned,
            HelpTab::OtherCrafts,
        ];
        for (i, t1) in tabs.iter().enumerate() {
            for (j, t2) in tabs.iter().enumerate() {
                if i != j {
                    assert_ne!(t1.label(), t2.label());
                }
            }
        }
    }

    #[test]
    fn filter_palette_commands_matches_and_lists_all() {
        let all = filter_palette_commands("");
        assert!(all.len() >= 30);

        let fc = filter_palette_commands("firecracker");
        assert!(!fc.is_empty());
        assert!(fc.iter().all(|(id, label, desc)| {
            id.to_lowercase().contains("firecracker")
                || label.to_lowercase().contains("firecracker")
                || desc.to_lowercase().contains("firecracker")
        }));
    }

    #[test]
    fn airgap_messages_toggle_state() {
        let (tx, rx) = mpsc::channel();
        let mut app = VmApp {
            rt: None,
            rt_error: None,
            tx: tx.clone(),
            rx,
            pending_tasks: 2,
            page: Page::Dashboard,
            status: String::new(),
            host_info: None,
            vms: Vec::new(),
            snapshots: Vec::new(),
            shot_image: None,
            shot_texture: None,
            bus_cmd: "vm.list".to_string(),
            bus_params: "{}".to_string(),
            vm_output: String::new(),
            vm_name: String::new(),
            ostype: "Ubuntu_64".to_string(),
            memory_mb: "4096".to_string(),
            snap_name: String::new(),
            net_ip: "192.168.56.1".to_string(),
            netmask: "255.255.255.0".to_string(),
            net_name: String::new(),
            disk_path: String::new(),
            disk_gb: "20".to_string(),
            shot_out: String::new(),
            sandbox_code: String::new(),
            sandbox_output: String::new(),
            mutate_confirm: false,
            dark_mode: true,
            airgap_active: false,
            disasm_code: String::new(),
            detonate_keygen: String::new(),
            detonate_target: String::new(),
            detonate_disasm_exe: "ida64.exe".to_string(),
            detonate_user: String::new(),
            detonate_pass: String::new(),
            detonate_guest_out: "C:\\detonate\\out.asm".to_string(),
            detonate_host_out: "out.asm".to_string(),
            detonate_airgap: true,
            swarm_instances: Vec::new(),
            swarm_selected_idx: None,
            swarm_spawn_id: "microvm-1".to_string(),
            swarm_kernel: "/vmlinux".to_string(),
            swarm_rootfs: "/rootfs.ext4".to_string(),
            swarm_kvm_available: None,
            swarm_fc_installed: None,
            swarm_probe_reason: None,
            help_tab: HelpTab::GetStarted,
            show_palette: false,
            palette_query: String::new(),
            show_shortcuts: false,
            show_about: false,
            show_licenses: false,
            show_tab_bar: true,
            status_bar_pos: StatusBarPos::Bottom,
        };

        let _ = tx.send(AppMessage::CommandResult {
            title: "Airgap Sever".to_string(),
            value: serde_json::json!({"status": "ok"}),
        });
        app.drain();
        assert!(app.airgap_active);

        let _ = tx.send(AppMessage::CommandResult {
            title: "Airgap Reconnect".to_string(),
            value: serde_json::json!({"status": "ok"}),
        });
        app.drain();
        assert!(!app.airgap_active);
    }
}
