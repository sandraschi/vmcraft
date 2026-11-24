//! Tier-3 L3: workstation pages (VMs, snapshots, network, disks, shot, sandbox, commands bus).

use eframe::egui::{self, RichText};
use std::sync::{mpsc, Arc};
use vmcraft_core::{dispatch, Command};

const ACCENT_CYAN: egui::Color32 = egui::Color32::from_rgb(56, 189, 248);
const ACCENT_AMBER: egui::Color32 = egui::Color32::from_rgb(251, 146, 60);
const ACCENT_EMERALD: egui::Color32 = egui::Color32::from_rgb(52, 211, 153);

#[derive(Clone, Copy)]
struct Palette {
    card_bg: egui::Color32,
    card_border: egui::Color32,
    text_primary: egui::Color32,
    text_muted: egui::Color32,
    accent: egui::Color32,
    ok_bg: egui::Color32,
    ok_fg: egui::Color32,
    warn_bg: egui::Color32,
    warn_fg: egui::Color32,
    muted_pill_bg: egui::Color32,
    muted_pill_fg: egui::Color32,
    info_bg: egui::Color32,
    info_fg: egui::Color32,
    tab_active_bg: egui::Color32,
}

impl Palette {
    fn for_mode(dark: bool) -> Self {
        if dark {
            Self {
                card_bg: egui::Color32::from_rgb(17, 24, 39),
                card_border: egui::Color32::from_rgb(30, 41, 59),
                text_primary: egui::Color32::from_rgb(241, 245, 249),
                text_muted: egui::Color32::from_rgb(148, 163, 184),
                accent: ACCENT_CYAN,
                ok_bg: egui::Color32::from_rgb(6, 78, 59),
                ok_fg: ACCENT_EMERALD,
                warn_bg: egui::Color32::from_rgb(67, 20, 7),
                warn_fg: ACCENT_AMBER,
                muted_pill_bg: egui::Color32::from_rgb(30, 41, 59),
                muted_pill_fg: egui::Color32::from_rgb(148, 163, 184),
                info_bg: egui::Color32::from_rgb(8, 51, 68),
                info_fg: ACCENT_CYAN,
                tab_active_bg: egui::Color32::from_rgb(12, 74, 110),
            }
        } else {
            Self {
                card_bg: egui::Color32::WHITE,
                card_border: egui::Color32::from_rgb(203, 213, 225),
                text_primary: egui::Color32::from_rgb(15, 23, 42),
                text_muted: egui::Color32::from_rgb(71, 85, 105),
                accent: egui::Color32::from_rgb(3, 105, 161),
                ok_bg: egui::Color32::from_rgb(209, 250, 229),
                ok_fg: egui::Color32::from_rgb(4, 120, 87),
                warn_bg: egui::Color32::from_rgb(254, 243, 199),
                warn_fg: egui::Color32::from_rgb(180, 83, 9),
                muted_pill_bg: egui::Color32::from_rgb(226, 232, 240),
                muted_pill_fg: egui::Color32::from_rgb(51, 65, 85),
                info_bg: egui::Color32::from_rgb(224, 242, 254),
                info_fg: egui::Color32::from_rgb(3, 105, 161),
                tab_active_bg: egui::Color32::from_rgb(224, 242, 254),
            }
        }
    }
}

fn card_frame(pal: Palette) -> egui::Frame {
    egui::Frame::NONE
        .fill(pal.card_bg)
        .stroke(egui::Stroke::new(1.0, pal.card_border))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(12))
}

fn render_kpi_card(
    ui: &mut egui::Ui,
    pal: Palette,
    label: &str,
    value: &str,
    value_color: egui::Color32,
    sub: &str,
) {
    card_frame(pal).show(ui, |ui| {
        ui.set_min_width(150.0);
        ui.label(RichText::new(label).small().color(pal.text_muted));
        ui.add_space(2.0);
        ui.label(RichText::new(value).size(20.0).strong().color(value_color));
        ui.add_space(2.0);
        ui.label(RichText::new(sub).small().color(pal.text_muted));
    });
}

fn render_state_pill(ui: &mut egui::Ui, pal: Palette, state: &str) {
    if state.eq_ignore_ascii_case("running") {
        render_badge(ui, "RUNNING", pal.ok_bg, pal.ok_fg);
    } else {
        render_badge(ui, "STOPPED", pal.muted_pill_bg, pal.muted_pill_fg);
    }
}

fn render_provider_pill(ui: &mut egui::Ui, pal: Palette, provider: &str) {
    if provider == "hyperv" {
        render_badge(ui, "Hyper-V", pal.info_bg, pal.info_fg);
    } else {
        render_badge(ui, "VirtualBox", pal.warn_bg, pal.warn_fg);
    }
}

/// Deep Slate-950 workstation dark visuals.
fn harsh_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_rgb(241, 245, 249));
    visuals.window_fill = egui::Color32::from_rgb(11, 17, 32);
    visuals.panel_fill = egui::Color32::from_rgb(11, 17, 32);
    visuals.extreme_bg_color = egui::Color32::from_rgb(2, 6, 23);
    visuals.code_bg_color = egui::Color32::from_rgb(17, 24, 39);
    visuals.faint_bg_color = egui::Color32::from_rgb(23, 33, 52);
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

/// Crisp light visuals (soft cool-slate canvas with pure white elevated cards).
fn crisp_light_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    let ink = egui::Color32::from_rgb(15, 23, 42);
    visuals.override_text_color = Some(ink);
    visuals.weak_text_color = Some(egui::Color32::from_rgb(71, 85, 105));
    visuals.window_fill = egui::Color32::from_rgb(248, 250, 252);
    visuals.panel_fill = egui::Color32::from_rgb(248, 250, 252);
    visuals.extreme_bg_color = egui::Color32::WHITE;
    visuals.faint_bg_color = egui::Color32::from_rgb(241, 245, 249);
    visuals.code_bg_color = egui::Color32::from_rgb(241, 245, 249);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, ink);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, ink);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, ink);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, ink);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(203, 213, 225));
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(148, 163, 184));
    visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.5, egui::Color32::from_rgb(3, 105, 161));
    visuals.window_corner_radius = egui::CornerRadius::same(8);
    visuals.menu_corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
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

/// Parse `VM Info` JSON response into human-readable `(label, value)` pairs.
fn parse_vm_info_fields(val: &serde_json::Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(reason) = val.get("reason").and_then(|v| v.as_str()) {
        out.push(("Error".to_string(), reason.to_string()));
        return out;
    }
    let Some(raw) = val.get("info").and_then(|v| v.as_str()) else {
        if let Some(vm) = val.get("vm").and_then(|v| v.as_str()) {
            out.push(("VM".to_string(), vm.to_string()));
        }
        return out;
    };

    let mut kv: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim().trim_matches('"').to_string();
            let val_str = v.trim().trim_matches('"').to_string();
            kv.insert(key, val_str);
        }
    }

    let mappings: &[(&str, &str, &str)] = &[
        ("name", "Name", ""),
        ("ostype", "OS Type", ""),
        ("VMState", "State", ""),
        ("cpus", "CPUs", ""),
        ("memory", "Memory", " MB"),
        ("vram", "VRAM", " MB"),
        ("graphicscontroller", "Graphics", ""),
        ("nic1", "Network (NIC 1)", ""),
        ("vrde", "VRDE Remote", ""),
        ("vrdeports", "VRDE Port", ""),
        ("CurrentSnapshotName", "Current Snapshot", ""),
        ("UUID", "UUID", ""),
        ("CfgFile", "Config File", ""),
    ];

    for (key, label, suffix) in mappings {
        if let Some(v) = kv.get(*key) {
            if !v.is_empty() && v != "none" {
                out.push(((*label).to_string(), format!("{v}{suffix}")));
            }
        }
    }
    out
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum VmFilterMode {
    All,
    Running,
    VirtualBox,
    HyperV,
}

#[derive(Clone, Debug)]
pub struct VmItem {
    pub name: String,
    pub uuid: String,
    pub provider: String,
    pub state: String,
}

#[derive(Clone, Debug)]
pub struct SnapshotItem {
    pub name: String,
    pub uuid: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct NetworkItem {
    pub name: String,
    pub ip: String,
    pub mask: String,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct DiskItem {
    pub uuid: String,
    pub state: String,
    pub format: String,
    pub capacity: String,
    pub location: String,
}

fn parse_kv_blocks(raw: &str) -> Vec<std::collections::HashMap<String, String>> {
    let mut blocks = Vec::new();
    let mut cur = std::collections::HashMap::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !cur.is_empty() {
                blocks.push(cur);
                cur = std::collections::HashMap::new();
            }
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            cur.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    if !cur.is_empty() {
        blocks.push(cur);
    }
    blocks
}

fn parse_network_blocks(raw: &str) -> Vec<NetworkItem> {
    parse_kv_blocks(raw)
        .into_iter()
        .filter_map(|m| {
            let name = m.get("Name")?.clone();
            Some(NetworkItem {
                name,
                ip: m.get("IPAddress").cloned().unwrap_or_default(),
                mask: m.get("NetworkMask").cloned().unwrap_or_default(),
                status: m.get("Status").cloned().unwrap_or_default(),
            })
        })
        .collect()
}

fn parse_disk_blocks(raw: &str) -> Vec<DiskItem> {
    parse_kv_blocks(raw)
        .into_iter()
        .filter_map(|m| {
            let uuid = m.get("UUID")?.clone();
            Some(DiskItem {
                uuid,
                state: m.get("State").cloned().unwrap_or_default(),
                format: m.get("Storage format").cloned().unwrap_or_default(),
                capacity: m.get("Capacity").cloned().unwrap_or_default(),
                location: m.get("Location").cloned().unwrap_or_default(),
            })
        })
        .collect()
}

fn format_sandbox_output(val: &serde_json::Value) -> String {
    let status = val
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    if let Some(reason) = val.get("reason").and_then(|v| v.as_str()) {
        return format!("Status: {status}\nError: {reason}");
    }
    let exit = val.get("exit").and_then(|v| v.as_i64()).unwrap_or(-1);
    let stdout = val.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
    let stderr = val.get("stderr").and_then(|v| v.as_str()).unwrap_or("");
    let mut out = format!("Status: {status} (exit {exit})");
    if !stdout.trim().is_empty() {
        out.push_str("\n\n--- stdout ---\n");
        out.push_str(stdout.trim_end());
    }
    if !stderr.trim().is_empty() {
        out.push_str("\n\n--- stderr ---\n");
        out.push_str(stderr.trim_end());
    }
    out
}

fn format_action_summary(title: &str, val: &serde_json::Value) -> String {
    let status = val
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    if let Some(reason) = val.get("reason").and_then(|v| v.as_str()) {
        return format!("{title}: {status} - {reason}");
    }
    if let Some(path) = val.get("path").and_then(|v| v.as_str()) {
        return format!("{title}: {status} ({path})");
    }
    format!("{title}: {status}")
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
    vm_info_fields: Vec<(String, String)>,
    networks: Vec<NetworkItem>,
    disks: Vec<DiskItem>,
    sandbox_lang: String,
    last_action_summary: String,
    vm_filter: String,
    vm_filter_mode: VmFilterMode,
    activity_log: Vec<(String, bool)>,
    guest_user: String,
    guest_pass: String,
    guest_cmd: String,
    guest_output: String,
    naked_repo: String,
    naked_snapshot: String,
    naked_linux: bool,
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
            vm_info_fields: Vec::new(),
            networks: Vec::new(),
            disks: Vec::new(),
            sandbox_lang: "python".to_string(),
            last_action_summary: String::new(),
            vm_filter: String::new(),
            vm_filter_mode: VmFilterMode::All,
            activity_log: Vec::new(),
            guest_user: "user".to_string(),
            guest_pass: "user".to_string(),
            guest_cmd: "uname -a; lsb_release -ds 2>/dev/null || true".to_string(),
            guest_output: String::new(),
            naked_repo: "sandraschi/vmcraft".to_string(),
            naked_snapshot: "clean-desktop-2026-11-22".to_string(),
            naked_linux: true,
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
                            let state = item
                                .get("state")
                                .and_then(|s| s.as_str())
                                .unwrap_or("off")
                                .to_string();
                            if !name.is_empty() {
                                self.vms.push(VmItem {
                                    name,
                                    uuid,
                                    provider,
                                    state,
                                });
                            }
                        }
                        if self.vm_name.is_empty() && !self.vms.is_empty() {
                            self.vm_name = self.vms[0].name.clone();
                        }
                    }
                    let summary: Vec<String> = self
                        .vms
                        .iter()
                        .map(|v| format!("{} ({})", v.name, v.provider))
                        .collect();
                    self.vm_output = if summary.is_empty() {
                        "No VMs found.".to_string()
                    } else {
                        format!("Found {} VMs: {}", summary.len(), summary.join(", "))
                    };
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
                    } else if title == "VM Info" {
                        self.vm_info_fields = parse_vm_info_fields(&value);
                    } else if title == "Network List" {
                        let raw = value.get("raw").and_then(|v| v.as_str()).unwrap_or("");
                        self.networks = parse_network_blocks(raw);
                    } else if title == "Disk List" {
                        let raw = value.get("raw").and_then(|v| v.as_str()).unwrap_or("");
                        self.disks = parse_disk_blocks(raw);
                    } else if title == "Sandbox Run" {
                        self.sandbox_output = format_sandbox_output(&value);
                    } else if title == "Guest Exec" || title == "Naked Test" {
                        let status = value
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let mut lines = vec![format!("{title}: {status}")];
                        if let Some(steps) = value.get("steps").and_then(|v| v.as_array()) {
                            for step in steps {
                                if let Some(s) = step.as_str() {
                                    lines.push(format!("  - {s}"));
                                }
                            }
                        }
                        if let Some(out) = value.get("output").and_then(|v| v.as_str()) {
                            if !out.trim().is_empty() {
                                lines.push(String::new());
                                lines.push(out.trim_end().to_string());
                            }
                        }
                        if let Some(reason) = value.get("reason").and_then(|v| v.as_str()) {
                            lines.push(format!("Error: {reason}"));
                        }
                        self.guest_output = lines.join("\n");
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
                    let is_ok = value.get("status").and_then(|v| v.as_str()) == Some("ok");
                    self.last_action_summary = format_action_summary(&title, &value);
                    self.activity_log
                        .insert(0, (self.last_action_summary.clone(), is_ok));
                    self.activity_log.truncate(40);
                    if is_ok
                        && matches!(
                            title.as_str(),
                            "VM Start" | "VM Stop" | "Hyper-V Start" | "Hyper-V Stop"
                        )
                    {
                        self.spawn_vms();
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

    fn filtered_vms(&self) -> Vec<VmItem> {
        let q = self.vm_filter.trim().to_lowercase();
        self.vms
            .iter()
            .filter(|vm| {
                let mode_ok = match self.vm_filter_mode {
                    VmFilterMode::All => true,
                    VmFilterMode::Running => vm.state.eq_ignore_ascii_case("running"),
                    VmFilterMode::VirtualBox => vm.provider == "virtualbox",
                    VmFilterMode::HyperV => vm.provider == "hyperv",
                };
                let text_ok = q.is_empty()
                    || vm.name.to_lowercase().contains(&q)
                    || vm.uuid.to_lowercase().contains(&q);
                mode_ok && text_ok
            })
            .cloned()
            .collect()
    }

    fn render_vm_filter_bar(&mut self, ui: &mut egui::Ui, pal: Palette) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Filter:").color(pal.text_muted));
            ui.add(
                egui::TextEdit::singleline(&mut self.vm_filter)
                    .hint_text("Search VM name or UUID...")
                    .desired_width(200.0),
            );
            for (mode, label) in [
                (VmFilterMode::All, "All"),
                (VmFilterMode::Running, "Running"),
                (VmFilterMode::VirtualBox, "VirtualBox"),
                (VmFilterMode::HyperV, "Hyper-V"),
            ] {
                if ui
                    .selectable_label(self.vm_filter_mode == mode, label)
                    .clicked()
                {
                    self.vm_filter_mode = mode;
                }
            }
            if !self.vm_filter.is_empty() && ui.button("Clear").clicked() {
                self.vm_filter.clear();
            }
        });
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
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Equals) {
                let z = (ui.ctx().zoom_factor() * 1.1).clamp(0.6, 2.5);
                ui.ctx().set_zoom_factor(z);
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Minus) {
                let z = (ui.ctx().zoom_factor() / 1.1).clamp(0.6, 2.5);
                ui.ctx().set_zoom_factor(z);
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
                if ui.button("Zoom In\tCtrl+=").clicked() {
                    let z = (ui.ctx().zoom_factor() * 1.1).clamp(0.6, 2.5);
                    ui.ctx().set_zoom_factor(z);
                    ui.close();
                }
                if ui.button("Zoom Out\tCtrl+-").clicked() {
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

        let pal = Palette::for_mode(self.dark_mode);

        // Page navigation bar (hideable single-click alternative to Window menu)
        if self.show_tab_bar {
            ui.horizontal_wrapped(|ui| {
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
                        egui::Button::new(RichText::new(p.label()).strong().color(pal.accent))
                            .fill(pal.tab_active_bg)
                            .stroke(egui::Stroke::new(1.0, pal.accent))
                    } else {
                        egui::Button::new(RichText::new(p.label()).color(pal.text_primary))
                    };
                    if ui.add(btn).clicked() {
                        self.page = p;
                    }
                }
            });
            ui.separator();
        }

        match self.page {
            Page::Dashboard => {
                let total_vms = self.vms.len();
                let running_vms = self
                    .vms
                    .iter()
                    .filter(|v| v.state.eq_ignore_ascii_case("running"))
                    .count();
                let vbox_count = self.vms.iter().filter(|v| v.provider != "hyperv").count();
                let hv_count = total_vms.saturating_sub(vbox_count);
                let selected_label = if self.vm_name.is_empty() {
                    "None".to_string()
                } else {
                    self.vm_name.clone()
                };

                // KPI Summary Cards
                ui.horizontal_wrapped(|ui| {
                    render_kpi_card(
                        ui,
                        pal,
                        "TOTAL VMS",
                        &total_vms.to_string(),
                        pal.accent,
                        &format!("{vbox_count} VirtualBox, {hv_count} Hyper-V"),
                    );
                    render_kpi_card(
                        ui,
                        pal,
                        "RUNNING NOW",
                        &running_vms.to_string(),
                        if running_vms > 0 {
                            pal.ok_fg
                        } else {
                            pal.text_muted
                        },
                        if running_vms > 0 {
                            "Active guest(s)"
                        } else {
                            "All guests powered off"
                        },
                    );
                    render_kpi_card(
                        ui,
                        pal,
                        "SELECTED TARGET",
                        &selected_label,
                        pal.text_primary,
                        "Click any VM row to switch",
                    );
                    card_frame(pal).show(ui, |ui| {
                        ui.set_min_width(170.0);
                        ui.label(RichText::new("QUICK JUMP").small().color(pal.text_muted));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button("Snapshots").clicked() {
                                self.page = Page::Snapshots;
                            }
                            if ui.button("Shot").clicked() {
                                self.page = Page::Shot;
                            }
                            if ui.button("Sandbox").clicked() {
                                self.page = Page::Sandbox;
                            }
                        });
                    });
                });
                ui.add_space(6.0);

                // Virtual Machines Card
                card_frame(pal).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Virtual Machines");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("⟳ Refresh").clicked() {
                                self.refresh_all();
                            }
                        });
                    });
                    self.render_vm_filter_bar(ui, pal);
                    ui.add_space(4.0);

                    let visible = self.filtered_vms();
                    if self.vms.is_empty() {
                        if self.pending_tasks > 0 {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Loading VMs from hypervisors…");
                            });
                        } else {
                            ui.label("No virtual machines registered. Click '⟳ Refresh' or create a VM in the 'VMs' tab.");
                        }
                    } else if visible.is_empty() {
                        ui.label(
                            RichText::new("No VMs match the current filter.").color(pal.text_muted),
                        );
                    } else {
                        let mut action_cmd: Option<(&'static str, Command)> = None;
                        let mut select_vm: Option<String> = None;

                        egui::ScrollArea::vertical()
                            .max_height(240.0)
                            .show(ui, |ui| {
                                egui::Grid::new("vm_dashboard_grid")
                                    .striped(true)
                                    .spacing([14.0, 8.0])
                                    .show(ui, |ui| {
                                        ui.label(RichText::new("VM Name").strong());
                                        ui.label(RichText::new("State").strong());
                                        ui.label(RichText::new("Provider").strong());
                                        ui.label(RichText::new("UUID").strong());
                                        ui.label(RichText::new("Actions").strong());
                                        ui.end_row();

                                        for vm in &visible {
                                            let is_sel = self.vm_name == vm.name;
                                            let name_text = if is_sel {
                                                RichText::new(&vm.name).strong().color(pal.accent)
                                            } else {
                                                RichText::new(&vm.name)
                                                    .strong()
                                                    .color(pal.text_primary)
                                            };
                                            if ui.selectable_label(is_sel, name_text).clicked() {
                                                select_vm = Some(vm.name.clone());
                                            }
                                            render_state_pill(ui, pal, &vm.state);
                                            render_provider_pill(ui, pal, &vm.provider);
                                            ui.label(
                                                RichText::new(&vm.uuid)
                                                    .monospace()
                                                    .small()
                                                    .color(pal.text_muted),
                                            );
                                            ui.horizontal(|ui| {
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
                                                        self.page = Page::Vms;
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
                });

                ui.add_space(6.0);
                // Recent Activity Card (human-readable feed)
                card_frame(pal).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Recent Activity").strong());
                        if !self.activity_log.is_empty() && ui.button("Clear").clicked() {
                            self.activity_log.clear();
                        }
                    });
                    if self.activity_log.is_empty() {
                        ui.label(
                            RichText::new("No actions run yet in this session.")
                                .color(pal.text_muted),
                        );
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(130.0)
                            .show(ui, |ui| {
                                for (msg, is_ok) in self.activity_log.iter().take(12) {
                                    ui.horizontal(|ui| {
                                        let dot = if *is_ok { pal.ok_fg } else { pal.warn_fg };
                                        ui.label(RichText::new("●").color(dot));
                                        ui.label(msg);
                                    });
                                }
                            });
                    }
                    ui.add_space(4.0);
                    egui::CollapsingHeader::new("Raw Command Output (JSON)")
                        .default_open(false)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(160.0)
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.vm_output)
                                            .desired_rows(8)
                                            .desired_width(f32::INFINITY)
                                            .font(egui::TextStyle::Monospace),
                                    );
                                });
                        });
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
                if !self.vms.is_empty() {
                    let mut vms_action: Option<(String, Option<(&'static str, Command)>)> = None;
                    card_frame(pal).show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("Registered VMs ({})", self.vms.len())).strong(),
                        );
                        ui.add_space(4.0);
                        egui::Grid::new("vms_page_table")
                            .striped(true)
                            .spacing([12.0, 6.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("Name").strong());
                                ui.label(RichText::new("State").strong());
                                ui.label(RichText::new("Provider").strong());
                                ui.label(RichText::new("UUID").strong());
                                ui.label(RichText::new("Actions").strong());
                                ui.end_row();
                                for vm in &self.vms {
                                    let selected = self.vm_name == vm.name;
                                    if ui.selectable_label(selected, &vm.name).clicked() {
                                        vms_action = Some((vm.name.clone(), None));
                                    }
                                    render_state_pill(ui, pal, &vm.state);
                                    render_provider_pill(ui, pal, &vm.provider);
                                    ui.label(
                                        RichText::new(&vm.uuid)
                                            .monospace()
                                            .small()
                                            .color(pal.text_muted),
                                    );
                                    ui.horizontal(|ui| {
                                        if ui.button("Info").clicked() {
                                            vms_action = Some((
                                                vm.name.clone(),
                                                Some((
                                                    "VM Info",
                                                    Command::VmInfo {
                                                        vm: vm.name.clone(),
                                                    },
                                                )),
                                            ));
                                        }
                                        if ui.button("Start").clicked() {
                                            vms_action = Some((
                                                vm.name.clone(),
                                                Some((
                                                    "VM Start",
                                                    Command::VmStart {
                                                        vm: vm.name.clone(),
                                                        confirm: self.confirm(),
                                                    },
                                                )),
                                            ));
                                        }
                                        if ui.button("Stop").clicked() {
                                            vms_action = Some((
                                                vm.name.clone(),
                                                Some((
                                                    "VM Stop",
                                                    Command::VmStop {
                                                        vm: vm.name.clone(),
                                                        force: false,
                                                        confirm: self.confirm(),
                                                    },
                                                )),
                                            ));
                                        }
                                    });
                                    ui.end_row();
                                }
                            });
                    });
                    if let Some((name, cmd_opt)) = vms_action {
                        self.vm_name = name;
                        if let Some((title, cmd)) = cmd_opt {
                            self.spawn_command(title, cmd);
                        }
                    }
                    ui.add_space(6.0);
                }

                if !self.vm_info_fields.is_empty() {
                    card_frame(pal).show(ui, |ui| {
                        ui.label(RichText::new("Selected VM Details").strong());
                        ui.add_space(4.0);
                        egui::Grid::new("vm_info_details_grid")
                            .striped(true)
                            .spacing([16.0, 4.0])
                            .show(ui, |ui| {
                                for (k, v) in &self.vm_info_fields {
                                    ui.label(RichText::new(k).strong().color(pal.text_muted));
                                    ui.label(RichText::new(v).color(pal.text_primary));
                                    ui.end_row();
                                }
                            });
                    });
                    ui.add_space(6.0);
                }

                // Guest Shell & Clean-Room Linux Craft Test Card
                card_frame(pal).show(ui, |ui| {
                    ui.label(RichText::new("Guest Runner & Clean-Room Linux Craft Test").strong());
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("User:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.guest_user).desired_width(90.0),
                        );
                        ui.label("Pass:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.guest_pass)
                                .password(true)
                                .desired_width(90.0),
                        );
                        ui.label("Guest Cmd:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.guest_cmd).desired_width(260.0),
                        );
                        if ui.button("▶ Run in Guest").clicked() {
                            self.spawn_command(
                                "Guest Exec",
                                Command::GuestExec {
                                    vm: self.vm_name.clone(),
                                    user: self.guest_user.clone(),
                                    pass: self.guest_pass.clone(),
                                    exe: "/bin/bash".to_string(),
                                    args: vec!["-lc".to_string(), self.guest_cmd.clone()],
                                    timeout_secs: Some(60),
                                },
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Repo:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.naked_repo).desired_width(180.0),
                        );
                        ui.label("Snapshot:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.naked_snapshot)
                                .desired_width(180.0),
                        );
                        ui.checkbox(&mut self.naked_linux, "Linux guest");
                        if ui.button("🧪 Run Naked Test").clicked() {
                            self.spawn_command(
                                "Naked Test",
                                Command::PipelineNakedInstall {
                                    repo: self.naked_repo.clone(),
                                    baseline_vm: Some(self.vm_name.clone()),
                                    user: self.guest_user.clone(),
                                    pass: self.guest_pass.clone(),
                                    snapshot: if self.naked_snapshot.trim().is_empty() {
                                        None
                                    } else {
                                        Some(self.naked_snapshot.clone())
                                    },
                                    command: None,
                                    linux: self.naked_linux,
                                    confirm: self.confirm(),
                                },
                            );
                        }
                    });
                    if !self.guest_output.is_empty() {
                        ui.add_space(4.0);
                        egui::ScrollArea::vertical()
                            .max_height(120.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.guest_output)
                                        .desired_rows(5)
                                        .desired_width(f32::INFINITY)
                                        .font(egui::TextStyle::Monospace),
                                );
                            });
                    }
                });
                ui.add_space(4.0);

                egui::CollapsingHeader::new("Raw Command Output (JSON)")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.vm_output)
                                    .desired_rows(10)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
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
                                            RichText::new(&s.name).strong().color(pal.text_primary),
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
                if !self.last_action_summary.is_empty() {
                    ui.label(RichText::new(&self.last_action_summary).strong());
                }
                if self.networks.is_empty() {
                    ui.label("No host-only networks reported. Click 'List Networks' to query VirtualBox.");
                } else {
                    let mut picked_net: Option<String> = None;
                    egui::Grid::new("network_list_grid")
                        .striped(true)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Name").strong());
                            ui.label(RichText::new("IP").strong());
                            ui.label(RichText::new("Mask").strong());
                            ui.label(RichText::new("Status").strong());
                            ui.label(RichText::new("Action").strong());
                            ui.end_row();
                            for n in &self.networks {
                                ui.label(&n.name);
                                ui.label(&n.ip);
                                ui.label(&n.mask);
                                ui.label(&n.status);
                                if ui.button("Select").clicked() {
                                    picked_net = Some(n.name.clone());
                                }
                                ui.end_row();
                            }
                        });
                    if let Some(name) = picked_net {
                        self.net_name = name;
                    }
                }
                ui.separator();
                egui::CollapsingHeader::new("Raw Command Output (JSON)")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.vm_output)
                                    .desired_rows(8)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
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
                if !self.last_action_summary.is_empty() {
                    ui.label(RichText::new(&self.last_action_summary).strong());
                }
                if self.disks.is_empty() {
                    ui.label(
                        "No virtual disks listed yet. Click 'List Hard Disks' to query VirtualBox.",
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(260.0)
                        .show(ui, |ui| {
                            egui::Grid::new("disk_list_grid")
                                .striped(true)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("State").strong());
                                    ui.label(RichText::new("Format").strong());
                                    ui.label(RichText::new("Capacity").strong());
                                    ui.label(RichText::new("Location").strong());
                                    ui.label(RichText::new("UUID").strong());
                                    ui.end_row();
                                    for d in &self.disks {
                                        ui.label(&d.state);
                                        ui.label(&d.format);
                                        ui.label(&d.capacity);
                                        ui.label(&d.location);
                                        ui.label(RichText::new(&d.uuid).weak());
                                        ui.end_row();
                                    }
                                });
                        });
                }
                ui.separator();
                egui::CollapsingHeader::new("Raw Command Output (JSON)")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.vm_output)
                                    .desired_rows(8)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
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
                if !self.last_action_summary.is_empty() {
                    ui.label(RichText::new(&self.last_action_summary).strong());
                }
                egui::CollapsingHeader::new("Raw Command Output (JSON)")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.vm_output)
                                    .desired_rows(8)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
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
                if !self.last_action_summary.is_empty() {
                    ui.label(RichText::new(&self.last_action_summary).strong());
                }
                egui::CollapsingHeader::new("Raw Telemetry Output (JSON)")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.vm_output)
                                    .desired_rows(6)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
                    });
            }
            Page::Sandbox => {
                ui.heading("Ephemeral Sandbox Execution");
                ui.label("Executes Python or PowerShell in an isolated 30s timeout sandbox.");
                ui.horizontal(|ui| {
                    ui.label("Language:");
                    if ui
                        .selectable_label(self.sandbox_lang == "python", "Python")
                        .clicked()
                    {
                        self.sandbox_lang = "python".to_string();
                        if self.sandbox_code.trim() == "Write-Output 'hi-vm'" {
                            self.sandbox_code = "print('hi-vm')".to_string();
                        }
                    }
                    if ui
                        .selectable_label(self.sandbox_lang == "powershell", "PowerShell")
                        .clicked()
                    {
                        self.sandbox_lang = "powershell".to_string();
                        if self.sandbox_code.trim() == "print('hi-vm')" {
                            self.sandbox_code = "Write-Output 'hi-vm'".to_string();
                        }
                    }
                });
                ui.add(
                    egui::TextEdit::multiline(&mut self.sandbox_code)
                        .desired_rows(6)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
                if ui.button("▶ Run Sandbox").clicked() {
                    let code = self.sandbox_code.clone();
                    let lang = self.sandbox_lang.clone();
                    self.spawn_command("Sandbox Run", Command::SandboxRun { lang, code });
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

                                        ui.label(RichText::new(id).monospace().color(pal.accent));
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
            vm_info_fields: Vec::new(),
            networks: Vec::new(),
            disks: Vec::new(),
            sandbox_lang: "python".to_string(),
            last_action_summary: String::new(),
            vm_filter: String::new(),
            vm_filter_mode: VmFilterMode::All,
            activity_log: Vec::new(),
            guest_user: "user".to_string(),
            guest_pass: "user".to_string(),
            guest_cmd: "uname -a; lsb_release -ds 2>/dev/null || true".to_string(),
            guest_output: String::new(),
            naked_repo: "sandraschi/vmcraft".to_string(),
            naked_snapshot: "clean-desktop-2026-11-22".to_string(),
            naked_linux: true,
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
            vm_info_fields: Vec::new(),
            networks: Vec::new(),
            disks: Vec::new(),
            sandbox_lang: "python".to_string(),
            last_action_summary: String::new(),
            vm_filter: String::new(),
            vm_filter_mode: VmFilterMode::All,
            activity_log: Vec::new(),
            guest_user: "user".to_string(),
            guest_pass: "user".to_string(),
            guest_cmd: "uname -a; lsb_release -ds 2>/dev/null || true".to_string(),
            guest_output: String::new(),
            naked_repo: "sandraschi/vmcraft".to_string(),
            naked_snapshot: "clean-desktop-2026-11-22".to_string(),
            naked_linux: true,
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
            vm_info_fields: Vec::new(),
            networks: Vec::new(),
            disks: Vec::new(),
            sandbox_lang: "python".to_string(),
            last_action_summary: String::new(),
            vm_filter: String::new(),
            vm_filter_mode: VmFilterMode::All,
            activity_log: Vec::new(),
            guest_user: "user".to_string(),
            guest_pass: "user".to_string(),
            guest_cmd: "uname -a; lsb_release -ds 2>/dev/null || true".to_string(),
            guest_output: String::new(),
            naked_repo: "sandraschi/vmcraft".to_string(),
            naked_snapshot: "clean-desktop-2026-11-22".to_string(),
            naked_linux: true,
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

    #[test]
    fn parse_vm_info_fields_extracts_human_readable_summary() {
        let val = serde_json::json!({
            "status": "ok",
            "info": "name=\"ubu-2404-lts\"\r\nostype=\"Ubuntu (64-bit)\"\r\nVMState=\"poweroff\"\r\ncpus=4\r\nmemory=8192\r\nCurrentSnapshotName=\"clean-desktop-2026-11-22\"\r\n"
        });
        let fields = parse_vm_info_fields(&val);
        assert!(fields
            .iter()
            .any(|(k, v)| k == "Name" && v == "ubu-2404-lts"));
        assert!(fields
            .iter()
            .any(|(k, v)| k == "OS Type" && v == "Ubuntu (64-bit)"));
        assert!(fields.iter().any(|(k, v)| k == "Memory" && v == "8192 MB"));
        assert!(fields.iter().any(|(k, v)| k == "CPUs" && v == "4"));
        assert!(fields.iter().any(|(k, v)| k == "State" && v == "poweroff"));
    }

    #[test]
    fn parse_network_and_disk_blocks_work() {
        let nets = parse_network_blocks(
            "Name: VirtualBox Host-Only Ethernet Adapter\nIPAddress: 192.168.56.1\nNetworkMask: 255.255.255.0\nStatus: Up\n",
        );
        assert_eq!(nets.len(), 1);
        assert_eq!(nets[0].ip, "192.168.56.1");

        let disks = parse_disk_blocks(
            "UUID: abc-123\nState: created\nStorage format: VDI\nCapacity: 40960 MBytes\nLocation: C:\\vms\\u.vdi\n",
        );
        assert_eq!(disks.len(), 1);
        assert_eq!(disks[0].format, "VDI");
        assert_eq!(disks[0].capacity, "40960 MBytes");
    }

    #[test]
    fn format_sandbox_output_is_human_readable() {
        let ok = serde_json::json!({"status": "ok", "exit": 0, "stdout": "hi-vm\n", "stderr": ""});
        let s = format_sandbox_output(&ok);
        assert!(s.contains("Status: ok (exit 0)"));
        assert!(s.contains("hi-vm"));
    }
}
