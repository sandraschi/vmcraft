//! Tier-2 L3: full workstation pages (VMs, snapshots, network, disks, shot, sandbox).

use eframe::egui;
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

pub struct VmApp {
    rt: Option<tokio::runtime::Runtime>,
    rt_error: Option<String>,
    page: Page,
    status: String,
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
        let built = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .or_else(|_| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
            });
        let (rt, rt_error) = match built {
            Ok(r) => (Some(r), None),
            Err(e) => (None, Some(format!("runtime init failed: {e}"))),
        };
        Self {
            rt,
            rt_error,
            page: Page::Dashboard,
            status: "starting…".to_string(),
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
        }
    }

    fn run<F>(&self, fut: F) -> Option<serde_json::Value>
    where
        F: std::future::Future<Output = serde_json::Value>,
    {
        self.rt.as_ref().map(|r| r.block_on(fut))
    }

    fn confirm(&self) -> bool {
        self.mutate_confirm
    }
}

impl eframe::App for VmApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(err) = self.rt_error.clone() {
            ui.colored_label(egui::Color32::RED, err);
            return;
        }
        ui.horizontal(|ui| {
            ui.heading("vmcraft");
            if ui.button("status").clicked() {
                if let Some(v) = self.run(dispatch(Command::Status)) {
                    self.status = v.to_string();
                }
            }
            ui.label(self.status.chars().take(200).collect::<String>());
        });
        ui.separator();
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
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.mutate_confirm, "allow mutate (== --yes)");
            ui.label("mutating verbs need this or VMCRAFT_ALLOW_MUTATE=1");
        });
        ui.separator();
        match self.page {
            Page::Dashboard => {
                ui.label("output");
                ui.text_edit_multiline(&mut self.vm_output);
                if ui.button("refresh: vm list + host info").clicked() {
                    let mut parts = Vec::new();
                    if let Some(v) = self.run(dispatch(Command::VmList { limit: Some(50) })) {
                        parts.push(v.to_string());
                    }
                    if let Some(v) = self.run(dispatch(Command::HostInfo)) {
                        parts.push(v.to_string());
                    }
                    self.vm_output = parts.join("\n");
                }
            }
            Page::Vms => {
                ui.horizontal(|ui| {
                    ui.label("VM:");
                    ui.text_edit_singleline(&mut self.vm_name);
                    ui.label("ostype:");
                    ui.text_edit_singleline(&mut self.ostype);
                    ui.label("mem:");
                    ui.text_edit_singleline(&mut self.memory_mb);
                });
                ui.horizontal(|ui| {
                    if ui.button("list").clicked() {
                        if let Some(v) = self.run(dispatch(Command::VmList { limit: Some(50) })) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("info").clicked() {
                        let name = self.vm_name.clone();
                        if let Some(v) = self.run(dispatch(Command::VmInfo { vm: name })) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("create").clicked() {
                        let mem = self.memory_mb.trim().parse::<u32>().unwrap_or(0);
                        if let Some(v) = self.run(dispatch(Command::VmCreate {
                            vm: self.vm_name.clone(),
                            ostype: self.ostype.clone(),
                            memory_mb: mem,
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("start").clicked() {
                        if let Some(v) = self.run(dispatch(Command::VmStart {
                            vm: self.vm_name.clone(),
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("stop").clicked() {
                        if let Some(v) = self.run(dispatch(Command::VmStop {
                            vm: self.vm_name.clone(),
                            force: false,
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("delete").clicked() {
                        if let Some(v) = self.run(dispatch(Command::VmDelete {
                            vm: self.vm_name.clone(),
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                });
                ui.text_edit_multiline(&mut self.vm_output);
            }
            Page::Snapshots => {
                ui.horizontal(|ui| {
                    ui.label("VM:");
                    ui.text_edit_singleline(&mut self.vm_name);
                    ui.label("snap:");
                    ui.text_edit_singleline(&mut self.snap_name);
                });
                ui.horizontal(|ui| {
                    if ui.button("list").clicked() {
                        if let Some(v) = self.run(dispatch(Command::SnapshotList {
                            vm: self.vm_name.clone(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("take").clicked() {
                        if let Some(v) = self.run(dispatch(Command::SnapshotCreate {
                            vm: self.vm_name.clone(),
                            snap: self.snap_name.clone(),
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("restore").clicked() {
                        if let Some(v) = self.run(dispatch(Command::SnapshotRestore {
                            vm: self.vm_name.clone(),
                            snap: self.snap_name.clone(),
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("delete").clicked() {
                        if let Some(v) = self.run(dispatch(Command::SnapshotDelete {
                            vm: self.vm_name.clone(),
                            snap: self.snap_name.clone(),
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                });
                ui.text_edit_multiline(&mut self.vm_output);
            }
            Page::Network => {
                ui.horizontal(|ui| {
                    ui.label("ip:");
                    ui.text_edit_singleline(&mut self.net_ip);
                    ui.label("mask:");
                    ui.text_edit_singleline(&mut self.netmask);
                    ui.label("iface:");
                    ui.text_edit_singleline(&mut self.net_name);
                });
                ui.horizontal(|ui| {
                    if ui.button("list").clicked() {
                        if let Some(v) = self.run(dispatch(Command::NetworkList)) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("create hostonlyif").clicked() {
                        if let Some(v) = self.run(dispatch(Command::NetworkCreate {
                            ip: self.net_ip.clone(),
                            netmask: self.netmask.clone(),
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("remove").clicked() {
                        if let Some(v) = self.run(dispatch(Command::NetworkRemove {
                            net: self.net_name.clone(),
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                });
                ui.text_edit_multiline(&mut self.vm_output);
            }
            Page::Disks => {
                ui.horizontal(|ui| {
                    ui.label("path (.vdi):");
                    ui.text_edit_singleline(&mut self.disk_path);
                    ui.label("GB:");
                    ui.text_edit_singleline(&mut self.disk_gb);
                });
                ui.horizontal(|ui| {
                    if ui.button("list hdds").clicked() {
                        if let Some(v) = self.run(dispatch(Command::DiskList)) {
                            self.vm_output = v.to_string();
                        }
                    }
                    if ui.button("create vdi").clicked() {
                        let gb = self.disk_gb.trim().parse::<u32>().unwrap_or(0);
                        if let Some(v) = self.run(dispatch(Command::DiskCreate {
                            path: self.disk_path.clone(),
                            size_gb: gb,
                            confirm: self.confirm(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                });
                ui.text_edit_multiline(&mut self.vm_output);
            }
            Page::Shot => {
                ui.horizontal(|ui| {
                    ui.label("VM:");
                    ui.text_edit_singleline(&mut self.vm_name);
                    ui.label("out .png:");
                    ui.text_edit_singleline(&mut self.shot_out);
                    if ui.button("screenshotpng").clicked() {
                        if let Some(v) = self.run(dispatch(Command::VmShot {
                            vm: self.vm_name.clone(),
                            out: self.shot_out.clone(),
                        })) {
                            self.vm_output = v.to_string();
                        }
                    }
                });
                ui.text_edit_multiline(&mut self.vm_output);
            }
            Page::Sandbox => {
                ui.label("sandbox run (python/pwsh, 30s, read-only safe)");
                ui.text_edit_multiline(&mut self.sandbox_code);
                if ui.button("run sandbox").clicked() {
                    let code = self.sandbox_code.clone();
                    if let Some(v) = self.run(dispatch(Command::SandboxRun {
                        lang: "python".to_string(),
                        code,
                    })) {
                        self.sandbox_output = v.to_string();
                    }
                }
                ui.text_edit_multiline(&mut self.sandbox_output);
            }
            Page::Help => {
                ui.label("vmcraft Tier-2: read-only always; mutating verbs need the checkbox (= --yes) or VMCRAFT_ALLOW_MUTATE=1. VM must run for Shot. VBoxManage via PATH or VBOXMANAGE_PATH.");
            }
        }
    }
}
