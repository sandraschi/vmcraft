//! Tier-1 L3 shell: VM list + detail + host + sandbox. Full gallery views land in Tier-2.

use eframe::egui;
use vmcraft_core::{dispatch, Command};

pub struct VmApp {
    rt: Option<tokio::runtime::Runtime>,
    rt_error: Option<String>,
    status: String,
    vm_output: String,
    vm_name: String,
    sandbox_code: String,
    sandbox_output: String,
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
            status: "starting…".to_string(),
            vm_output: String::new(),
            vm_name: String::new(),
            sandbox_code: "print('hi-vm')".to_string(),
            sandbox_output: String::new(),
        }
    }

    fn run<F>(&self, fut: F) -> Option<serde_json::Value>
    where
        F: std::future::Future<Output = serde_json::Value>,
    {
        self.rt.as_ref().map(|r| r.block_on(fut))
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
            ui.label(self.status.chars().take(160).collect::<String>());
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("VM:");
            ui.text_edit_singleline(&mut self.vm_name);
            if ui.button("vm list").clicked() {
                if let Some(v) = self.run(dispatch(Command::VmList { limit: Some(50) })) {
                    self.vm_output = v.to_string();
                }
            }
            if ui.button("vm info").clicked() {
                let name = self.vm_name.clone();
                if let Some(v) = self.run(dispatch(Command::VmInfo { vm: name })) {
                    self.vm_output = v.to_string();
                }
            }
            if ui.button("host info").clicked() {
                if let Some(v) = self.run(dispatch(Command::HostInfo)) {
                    self.vm_output = v.to_string();
                }
            }
        });
        ui.label("output (read-only Tier-1)");
        ui.text_edit_multiline(&mut self.vm_output);
        ui.separator();
        ui.label("sandbox run (python, 30s)");
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
}
