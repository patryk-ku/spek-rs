use crate::utils::get_ffmpeg_paths;
use eframe::egui;
use ffmpeg_sidecar::{
    self,
    download::{download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg},
};
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub struct FfmpegSetup {
    dialog_description: String,
    is_installing: bool,
    status_rx: Option<Receiver<String>>,
    status_message: String,
}

impl FfmpegSetup {
    pub fn new(dialog_description: String) -> Self {
        Self {
            dialog_description,
            is_installing: false,
            status_rx: None,
            status_message: String::new(),
        }
    }

    fn start_install_ffmpeg(&mut self, ctx: &egui::Context) {
        self.is_installing = true;
        let (tx, rx) = mpsc::channel();
        self.status_rx = Some(rx);

        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            let ff_paths = get_ffmpeg_paths();
            println!("Downloading FFmpeg to {}", ff_paths.directory.display());

            // Create dir
            let _ = tx.send(format!("Creating directory..."));
            ctx_clone.request_repaint();
            if let Err(e) = std::fs::create_dir_all(&ff_paths.directory) {
                let _ = tx.send(format!("Error: {}", e));
                ctx_clone.request_repaint();
                return;
            }

            // Download archive
            let _ = tx.send("Downloading...".to_string());
            ctx_clone.request_repaint();
            let archive_path = match ffmpeg_download_url() {
                Ok(url) => match download_ffmpeg_package(url, &ff_paths.directory) {
                    Ok(path) => Some(path),
                    Err(e) => {
                        let _ = tx.send(format!("Error, download failed: {}", e));
                        None
                    }
                },
                Err(e) => {
                    let _ = tx.send(format!("Error, failed to get URL: {}", e));
                    None
                }
            };

            // Unpack archive
            if let Some(path) = archive_path {
                let _ = tx.send("Unpacking...".to_string());
                ctx_clone.request_repaint();
                if let Err(e) = unpack_ffmpeg(&path, &ff_paths.directory) {
                    let _ = tx.send(format!("Error, failed to unpack archive: {}", e));
                } else {
                    let _ = tx.send("Done!".to_string());
                }
            }

            ctx_clone.request_repaint();
        });
    }
}

impl eframe::App for FfmpegSetup {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.status_rx {
            for status in rx.try_iter() {
                self.status_message = status;
            }
        }

        let frame = egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(20));
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("FFmpeg Installation");
                ui.add_space(15.0);

                if !self.status_message.starts_with("Error") {
                    ui.label(self.dialog_description.to_owned());
                    ui.add_space(20.0);
                }

                if self.is_installing {
                    if self.status_message.starts_with("Error") {
                        ui.add_space(40.0);
                        ui.heading("Instalation failed");
                        ui.add_space(15.0);
                        ui.strong(self.status_message.to_owned());
                        ui.add_space(20.0);
                        let exit_button =
                            egui::Button::new("  Exit  ").min_size(egui::vec2(120.0, 30.0));
                        if ui.add(exit_button).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    } else {
                        ui.columns(3, |columns| {
                            columns[1].vertical_centered(|ui| {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(self.status_message.to_owned());
                                });
                            });
                        });
                    }

                    if self.status_message == "Done!" {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                } else {
                    ui.columns(2, |columns| {
                        columns[0].vertical_centered(|ui| {
                            ui.scope(|ui| {
                                let accent_color = ui.visuals().selection.bg_fill;
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill = accent_color;
                                ui.style_mut().visuals.widgets.hovered.weak_bg_fill = accent_color;
                                ui.style_mut().visuals.widgets.active.weak_bg_fill = accent_color;

                                let install_button = egui::Button::new("  Install  ")
                                    .min_size(egui::vec2(120.0, 30.0));

                                if ui.add(install_button).clicked() {
                                    self.start_install_ffmpeg(ctx);
                                }
                            });
                        });
                        columns[1].vertical_centered(|ui| {
                            let cancel_button =
                                egui::Button::new("  Cancel  ").min_size(egui::vec2(120.0, 30.0));

                            if ui.add(cancel_button).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });
                }
            });
        });
    }
}
