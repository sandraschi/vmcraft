//! Tier-3 L3: workstation pages (VMs, snapshots, network, disks, shot, sandbox, commands bus).

use eframe::egui::{self, RichText};
use std::sync::{mpsc, Arc};
use vmcraft_core::{dispatch, Command};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Dashboard,
    Vms,
    Snapshots,
    Network,
    Disks,
    Shot,
    Detonate,
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
    detonate_keygen: String,
    detonate_target: String,
    detonate_disasm_exe: String,
    detonate_user: String,
    detonate_pass: String,
    detonate_guest_out: String,
    detonate_host_out: String,
    detonate_airgap: bool,
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
            detonate_keygen: String::new(),
            detonate_target: String::new(),
            detonate_disasm_exe: "ida64.exe".to_string(),
            detonate_user: String::new(),
            detonate_pass: String::new(),
            detonate_guest_out: "C:\\detonate\\out.asm".to_string(),
            detonate_host_out: "out.asm".to_string(),
            detonate_airgap: true,
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
}

impl eframe::App for VmApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.drain();

        if self.pending_tasks > 0 {
            ui.ctx().request_repaint();
        }

        if let Some(err) = self.rt_error.clone() {
            ui.colored_label(egui::Color32::RED, err);
            return;
        }

        // Top bar
        ui.horizontal(|ui| {
            ui.heading(RichText::new("vmcraft").strong());
            if ui.button("⟳ Refresh").clicked() {
                self.refresh_all();
            }
            if self.pending_tasks > 0 {
                ui.spinner();
                ui.label(RichText::new(format!("Loading ({}…)", self.pending_tasks)).weak());
            } else {
                ui.label(RichText::new(&self.status).color(egui::Color32::from_rgb(100, 200, 100)));
            }
            if let Some(ref host) = self.host_info {
                ui.separator();
                ui.label(RichText::new(host).weak());
            }
        });
        ui.separator();

        // Page navigation bar
        ui.horizontal(|ui| {
            for p in [
                Page::Dashboard,
                Page::Vms,
                Page::Snapshots,
                Page::Network,
                Page::Disks,
                Page::Shot,
                Page::Detonate,
                Page::Sandbox,
                Page::Commands,
                Page::Help,
            ] {
                if ui.selectable_label(self.page == p, p.label()).clicked() {
                    self.page = p;
                }
            }
        });
        ui.separator();

        // Safety mutation toggle
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.mutate_confirm, "allow mutate (== --yes)");
            ui.label(
                RichText::new("mutating verbs need this checkbox or VMCRAFT_ALLOW_MUTATE=1").weak(),
            );
        });
        ui.separator();

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
                                                .color(egui::Color32::from_rgb(220, 230, 245)),
                                        );
                                        if vm.provider == "hyperv" {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(100, 180, 255),
                                                "Hyper-V",
                                            );
                                        } else {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(255, 170, 80),
                                                "VirtualBox",
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
                ui.label(RichText::new("Detonation Progress & Telemetry:").strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.vm_output)
                            .desired_rows(12)
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
                ui.heading("vmcraft — Pure-Rust ArtCraft VM Workstation");
                ui.label(
                    "• Status: Queries VirtualBox and Hyper-V installation and hypervisor health.",
                );
                ui.label("• VMs: Multi-hypervisor VM manager (VirtualBox + Hyper-V), inspect hardware info, start/stop/create/delete.");
                ui.label("• Snapshots: Structured tree listing, point-in-time state capture, restore, and delete.");
                ui.label("• Network: Manage VirtualBox host-only networking interfaces.");
                ui.label("• Disks: Inspect and provision virtual disk images (.vdi).");
                ui.label(
                    "• Shot: Capture real-time framebuffer PNG screenshots and render them directly in-app.",
                );
                ui.label("• Sandbox: Safe 30s isolated script execution.");
                ui.label("• Commands: ArtCraft Command Bus interactive dispatcher and full registry catalog.");
                ui.separator();
                ui.label(RichText::new("Mutating Operations Safety Gate:").strong());
                ui.label("Destructive/mutating actions (create, delete, start, stop) require checking 'allow mutate' or setting VMCRAFT_ALLOW_MUTATE=1 in the environment.");
            }
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
            detonate_keygen: String::new(),
            detonate_target: String::new(),
            detonate_disasm_exe: "ida64.exe".to_string(),
            detonate_user: String::new(),
            detonate_pass: String::new(),
            detonate_guest_out: "C:\\detonate\\out.asm".to_string(),
            detonate_host_out: "out.asm".to_string(),
            detonate_airgap: true,
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
}
