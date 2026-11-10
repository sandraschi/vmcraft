//! Tier-2 L3: full workstation pages (VMs, snapshots, network, disks, shot, sandbox).

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
    Sandbox,
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
            Page::Sandbox => "Sandbox",
            Page::Help => "Help",
        }
    }
}

#[derive(Clone, Debug)]
pub struct VmItem {
    pub name: String,
    pub uuid: String,
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
            status: "Connecting to hypervisor…".to_string(),
            host_info: None,
            vms: Vec::new(),
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
                    let st = val
                        .get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("unknown");
                    if vbox {
                        self.status = format!("VirtualBox: active ({st})");
                    } else {
                        self.status = format!("VirtualBox: not detected ({st})");
                    }
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
                            if !name.is_empty() {
                                self.vms.push(VmItem { name, uuid });
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
                Page::Sandbox,
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
                            ui.label("Loading VMs from hypervisor…");
                        });
                    } else {
                        ui.label("No VirtualBox VMs registered. Click '⟳ Refresh' or create a VM in the 'VMs' tab.");
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
                                    ui.label(RichText::new("UUID").strong());
                                    ui.label(RichText::new("Actions").strong());
                                    ui.end_row();

                                    for vm in &self.vms {
                                        ui.label(
                                            RichText::new(&vm.name)
                                                .color(egui::Color32::from_rgb(220, 230, 245)),
                                        );
                                        ui.label(RichText::new(&vm.uuid).weak());
                                        ui.horizontal(|ui| {
                                            if ui.button("Select").clicked() {
                                                select_vm = Some(vm.name.clone());
                                            }
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
                        self.spawn_command(
                            "VM Screenshot",
                            Command::VmShot {
                                vm: self.vm_name.clone(),
                                out: self.shot_out.clone(),
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
            Page::Help => {
                ui.heading("vmcraft — Pure-Rust ArtCraft VM Workstation");
                ui.label("• Status: Queries VirtualBox installation and hypervisor health.");
                ui.label("• VMs: List registered virtual machines, inspect hardware info, start/stop/create/delete.");
                ui.label("• Snapshots: Point-in-time state capture and instant restore.");
                ui.label("• Network: Manage VirtualBox host-only networking interfaces.");
                ui.label("• Disks: Inspect and provision virtual disk images (.vdi).");
                ui.label(
                    "• Shot: Capture real-time framebuffer PNG screenshots of active running VMs.",
                );
                ui.label("• Sandbox: Safe 30s isolated script execution.");
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
            Page::Sandbox,
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
        };

        let sample_json = serde_json::json!({
            "status": "ok",
            "vms": [
                { "name": "ubu-2404-lts", "uuid": "{ef4f7178-1087-42a5-9539-e6d37a0c5616}" },
                { "name": "batocera-clean", "uuid": "{8a9b25ff-bba3-43b3-865e-2b22bcdd05fb}" }
            ]
        });

        let _ = tx.send(AppMessage::VmList(sample_json));
        app.drain();

        assert_eq!(app.vms.len(), 2);
        assert_eq!(app.vms[0].name, "ubu-2404-lts");
        assert_eq!(app.vm_name, "ubu-2404-lts");
        assert_eq!(app.pending_tasks, 0);
    }
}
