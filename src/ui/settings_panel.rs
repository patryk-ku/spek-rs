use eframe::egui;

use super::MyApp;
use crate::settings::{AppSettings, SpectogramWinFunc, SpectrogramColorScheme, SpectrogramScale};

impl MyApp {
    pub(super) fn show_settings_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let mut trigger_regeneration = false;

        self.show_file_buttons(ui, &mut trigger_regeneration);
        self.show_settings_controls(ctx, ui, &mut trigger_regeneration);

        if trigger_regeneration && !self.is_generating {
            self.regenerate_spectrogram(ctx);
        }
    }

    fn show_file_buttons(&mut self, ui: &mut egui::Ui, trigger_regeneration: &mut bool) {
        ui.add_enabled_ui(!self.is_generating, |ui| {
            if ui.button("Open File...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.input_path = Some(path.display().to_string());
                    self.audio_info =
                        crate::utils::get_audio_info(self.input_path.as_ref().unwrap());
                    *trigger_regeneration = true;
                }
            }

            if self.final_image.is_some() {
                if ui.button("Save As...").clicked() {
                    if let Some(input_path) = &self.input_path {
                        crate::utils::save_image(&self.final_image, input_path);
                    }
                }
            }
        });
    }

    fn show_settings_controls(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        trigger_regeneration: &mut bool,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
            ui.horizontal(|ui| {
                if self.final_image.is_none() && !self.is_generating && self.input_path.is_some() {
                    *trigger_regeneration = true;
                }

                ui.add_enabled_ui(!self.is_generating, |ui| {
                    self.show_more_options_menu(ui, trigger_regeneration);

                    ui.add_space(4.0);

                    self.show_scale_combo(ui, trigger_regeneration);

                    ui.add_space(4.0);

                    self.show_win_func_combo(ui, trigger_regeneration);

                    ui.add_space(4.0);

                    self.show_color_scheme_combo(ui, trigger_regeneration);

                    ui.add_space(4.0);

                    self.show_saturation_drag(ui, trigger_regeneration);

                    ui.add_space(4.0);

                    self.show_gain_drag(ui, trigger_regeneration);

                    ui.add_space(4.0);

                    self.show_custom_res_controls(ui, trigger_regeneration);
                });
            });
        });
    }

    fn show_more_options_menu(&mut self, ui: &mut egui::Ui, trigger_regeneration: &mut bool) {
        let more_button = ui.button("More...");
        egui::Popup::menu(&more_button)
            .gap(6.0)
            .align(egui::RectAlign::BOTTOM_END)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.add_enabled_ui(!self.is_generating, |ui| {
                    let mut dummy_true = true;
                    let mut dummy_false = false;

                    if ui
                        .checkbox(&mut self.settings.legend, "Draw legend")
                        .changed()
                    {
                        *trigger_regeneration = true;
                    }

                    if self.settings.legend {
                        if self.settings.live_mode {
                            ui.add_enabled(
                                false,
                                egui::Checkbox::new(&mut dummy_true, "Custom Legend"),
                            );
                        } else if ui
                            .checkbox(&mut self.settings.custom_legend, "Custom Legend")
                            .changed()
                        {
                            *trigger_regeneration = true;
                        }

                        if self.settings.custom_legend
                            || (self.settings.legend && self.settings.live_mode)
                        {
                            if ui.button("Legend settings").clicked() {
                                ui.close();
                            }
                        }
                        ui.separator();
                    }

                    let has_multiple_channels = self
                        .audio_info
                        .as_ref()
                        .map_or(false, |info| info.channels > 1);

                    if has_multiple_channels {
                        if ui
                            .checkbox(&mut self.settings.split_channels, "Split channels")
                            .changed()
                        {
                            *trigger_regeneration = true;
                        }
                    } else {
                        ui.add_enabled(
                            false,
                            egui::Checkbox::new(&mut dummy_false, "Split channels"),
                        );
                    }

                    if self.settings.live_mode || self.settings.custom_legend {
                        ui.add_enabled(false, egui::Checkbox::new(&mut dummy_false, "Horizontal"));
                    } else if ui
                        .checkbox(&mut self.settings.horizontal, "Horizontal")
                        .changed()
                    {
                        *trigger_regeneration = true;
                    }

                    ui.add_enabled(
                        false,
                        egui::Checkbox::new(&mut dummy_false, "Resize with window"),
                    );

                    if ui
                        .checkbox(&mut self.settings.live_mode, "Live mode (WIP)")
                        .changed()
                    {
                        *trigger_regeneration = true;
                    }

                    ui.separator();

                    if ui
                        .checkbox(&mut self.settings.remember_settings, "Remember settings")
                        .changed()
                    {
                        self.settings.save();
                    }

                    if ui.button("Reset settings").clicked() {
                        ui.close();
                        self.settings = AppSettings::default();
                        *trigger_regeneration = true;
                    }

                    ui.separator();

                    if ui.button("Keybindings").clicked() {
                        ui.close();
                    }
                    if ui.button("Help").clicked() {
                        ui.close();
                    }
                    if ui.button("About").clicked() {
                        self.about_window_open = true;
                        ui.close();
                    }
                });
            });
    }

    fn show_scale_combo(&mut self, ui: &mut egui::Ui, trigger_regeneration: &mut bool) {
        let old_scale = self.settings.scale;
        egui::ComboBox::from_label("Scale:")
            .selected_text(self.settings.scale.to_string())
            .width(55.0)
            .show_ui(ui, |ui| {
                for scale in SpectrogramScale::VALUES {
                    ui.selectable_value(&mut self.settings.scale, scale, scale.to_string());
                }
            });
        if self.settings.scale != old_scale {
            *trigger_regeneration = true;
        }
    }

    fn show_win_func_combo(&mut self, ui: &mut egui::Ui, trigger_regeneration: &mut bool) {
        let old_win_func = self.settings.win_func;
        egui::ComboBox::from_label("F:")
            .selected_text(self.settings.win_func.to_string())
            .width(80.0)
            .height(600.0)
            .show_ui(ui, |ui| {
                for win_function in SpectogramWinFunc::VALUES {
                    ui.selectable_value(
                        &mut self.settings.win_func,
                        win_function,
                        win_function.to_string(),
                    );
                }
            });
        if self.settings.win_func != old_win_func {
            *trigger_regeneration = true;
        }
    }

    fn show_color_scheme_combo(&mut self, ui: &mut egui::Ui, trigger_regeneration: &mut bool) {
        let old_color_scheme = self.settings.color_scheme;
        egui::ComboBox::from_label("Color:")
            .selected_text(self.settings.color_scheme.to_string())
            .width(80.0)
            .height(600.0)
            .show_ui(ui, |ui| {
                for color in SpectrogramColorScheme::VALUES {
                    ui.selectable_value(&mut self.settings.color_scheme, color, color.to_string());
                }
            });
        if self.settings.color_scheme != old_color_scheme {
            *trigger_regeneration = true;
        }
    }

    fn show_saturation_drag(&mut self, ui: &mut egui::Ui, trigger_regeneration: &mut bool) {
        let saturation_drag_value = egui::DragValue::new(&mut self.settings.saturation)
            .speed(0.1)
            .range(-10.0..=10.0);
        let saturation_response = ui.add(saturation_drag_value.prefix("Saturation: "));
        if saturation_response.drag_stopped() || saturation_response.lost_focus() {
            *trigger_regeneration = true;
        }
    }

    fn show_gain_drag(&mut self, ui: &mut egui::Ui, trigger_regeneration: &mut bool) {
        let gain_drag_value = egui::DragValue::new(&mut self.settings.gain)
            .speed(1.0)
            .range(1.0..=100.0);
        let gain_response = ui.add(gain_drag_value.prefix("Gain: "));
        if gain_response.drag_stopped() || gain_response.lost_focus() {
            *trigger_regeneration = true;
        }
    }

    fn show_custom_res_controls(&mut self, ui: &mut egui::Ui, trigger_regeneration: &mut bool) {
        if ui
            .checkbox(&mut self.settings.custom_resolution, "Custom Res")
            .changed()
        {
            *trigger_regeneration = true;
        }

        if self.settings.custom_resolution {
            ui.horizontal(|ui| {
                let width_response = ui.add(
                    egui::DragValue::new(&mut self.settings.resolution[0])
                        .prefix("w: ")
                        .suffix("px")
                        .speed(10.0)
                        .range(100.0..=7892.0),
                );
                ui.label("x");
                let height_response = ui.add(
                    egui::DragValue::new(&mut self.settings.resolution[1])
                        .prefix("h: ")
                        .suffix("px")
                        .speed(10.0)
                        .range(100.0..=7992.0),
                );
                if width_response.drag_stopped()
                    || width_response.lost_focus()
                    || height_response.drag_stopped()
                    || height_response.lost_focus()
                {
                    *trigger_regeneration = true;
                }
            });
        }
    }
}
