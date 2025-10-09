use eframe::egui::{self, ColorImage};
use std::sync::mpsc;
use std::thread;

use crate::utils;

pub struct MyApp {
    texture: Option<egui::TextureHandle>,
    image: Option<ColorImage>,
    input_path: String,
    legend: bool,
    color_scheme: utils::SpectrogramColorScheme,
    win_func: utils::SpectogramWinFunc,
    scale: utils::SpectrogramScale,
    gain: f32,
    saturation: f32,
    split_channels: bool,
    is_generating: bool,
    image_receiver: Option<mpsc::Receiver<Option<ColorImage>>>,
    custom_resolution: bool,
    resolution: [u32; 2],
    horizontal: bool,
}

impl MyApp {
    pub fn new(image: Option<ColorImage>, input_path: String) -> Self {
        Self {
            texture: None,
            image,
            input_path,
            legend: true,
            color_scheme: utils::SpectrogramColorScheme::Intensity,
            win_func: utils::SpectogramWinFunc::Hann,
            scale: utils::SpectrogramScale::Log,
            gain: 1.0,
            saturation: 1.0,
            split_channels: false,
            is_generating: false,
            image_receiver: None,
            custom_resolution: false,
            resolution: [800, 500],
            horizontal: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for a newly generated image from the background thread
        if let Some(receiver) = &self.image_receiver {
            if let Ok(maybe_image) = receiver.try_recv() {
                self.is_generating = false;
                self.image_receiver = None; // We're done with this receiver
                if let Some(new_image) = maybe_image {
                    self.texture = Some(ctx.load_texture(
                        "spectrogram",
                        new_image.clone(),
                        Default::default(),
                    ));
                    self.image = Some(new_image);
                }
            }
        }

        if self.texture.is_none() && self.image.is_some() {
            if let Some(image) = self.image.as_ref() {
                self.texture =
                    Some(ctx.load_texture("spectrogram", image.clone(), Default::default()));
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(egui::Margin::same(8))
                    .stroke(egui::Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    ))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let inner_gap = 4.0;

                            ui.strong("Spek-rs");
                            ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                            ui.add_space(inner_gap);

                            ui.add_enabled_ui(!self.is_generating && self.image.is_some(), |ui| {
                                if ui.button("Save As...").clicked() {
                                    utils::save_image(&self.image, &self.input_path);
                                }
                            });

                            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                                ui.horizontal(|ui| {
                                    let mut trigger_regeneration = false;

                                    // Start generating on first launch
                                    if self.image.is_none() && !self.is_generating {
                                        trigger_regeneration = true;
                                    }

                                    ui.add_enabled_ui(!self.is_generating, |ui| {
                                        // More options menu
                                        let response = ui.button("More...");
                                        egui::Popup::menu(&response)
                                            .gap(6.0)
                                            .align(egui::RectAlign::BOTTOM_END)
                                            .close_behavior(
                                                egui::PopupCloseBehavior::CloseOnClickOutside,
                                            )
                                            .show(|ui| {
                                                ui.add_enabled_ui(!self.is_generating, |ui| {
                                                    // Legend checkbox
                                                    let old_legend = self.legend;
                                                    ui.checkbox(&mut self.legend, "Legend");
                                                    if self.legend != old_legend {
                                                        trigger_regeneration = true;
                                                    }

                                                    // Split channels checkbox
                                                    let old_split_channels = self.split_channels;
                                                    ui.checkbox(
                                                        &mut self.split_channels,
                                                        "Split Channels",
                                                    );
                                                    if self.split_channels != old_split_channels {
                                                        trigger_regeneration = true;
                                                    }

                                                    // Horizontal spectogram
                                                    let old_horizontal = self.horizontal;
                                                    ui.checkbox(&mut self.horizontal, "Horizontal");
                                                    if self.horizontal != old_horizontal {
                                                        trigger_regeneration = true;
                                                    }

                                                    // Reset settings button
                                                    if ui.button("Reset").clicked() {
                                                        ui.close();
                                                        self.legend = true;
                                                        self.color_scheme =
                                                        utils::SpectrogramColorScheme::Intensity;
                                                        self.win_func =
                                                            utils::SpectogramWinFunc::Hann;
                                                        self.scale = utils::SpectrogramScale::Log;
                                                        self.gain = 1.0;
                                                        self.saturation = 1.0;
                                                        self.split_channels = false;
                                                        self.custom_resolution = false;
                                                        self.resolution = [800, 500];
                                                        self.horizontal = false;
                                                        trigger_regeneration = true;
                                                    }
                                                });
                                            });

                                        ui.add_space(inner_gap);

                                        // Scale combobox
                                        let old_scale = self.scale;
                                        egui::ComboBox::from_label("Scale:")
                                            .selected_text(self.scale.to_string())
                                            .width(55.0)
                                            .show_ui(ui, |ui| {
                                                for scale in utils::SpectrogramScale::VALUES {
                                                    ui.selectable_value(
                                                        &mut self.scale,
                                                        scale,
                                                        scale.to_string(),
                                                    );
                                                }
                                            });
                                        if self.scale != old_scale {
                                            trigger_regeneration = true;
                                        }

                                        ui.add_space(inner_gap);

                                        // Window function combobox
                                        let old_win_func = self.win_func;
                                        egui::ComboBox::from_label("F:")
                                            .selected_text(self.win_func.to_string())
                                            .width(80.0)
                                            .height(600.0)
                                            .show_ui(ui, |ui| {
                                                for win_function in utils::SpectogramWinFunc::VALUES
                                                {
                                                    ui.selectable_value(
                                                        &mut self.win_func,
                                                        win_function,
                                                        win_function.to_string(),
                                                    );
                                                }
                                            });
                                        if self.win_func != old_win_func {
                                            trigger_regeneration = true;
                                        }

                                        ui.add_space(inner_gap);

                                        // Color scheme combobox
                                        let old_color_scheme = self.color_scheme;
                                        egui::ComboBox::from_label("Colors:")
                                            .selected_text(self.color_scheme.to_string())
                                            .width(80.0)
                                            .height(600.0)
                                            .show_ui(ui, |ui| {
                                                for color in utils::SpectrogramColorScheme::VALUES {
                                                    ui.selectable_value(
                                                        &mut self.color_scheme,
                                                        color,
                                                        color.to_string(),
                                                    );
                                                }
                                            });
                                        if self.color_scheme != old_color_scheme {
                                            trigger_regeneration = true;
                                        }

                                        ui.add_space(inner_gap);

                                        // Saturation drag input
                                        let saturation_drag_value =
                                            egui::DragValue::new(&mut self.saturation)
                                                .speed(0.1)
                                                .range(-10.0..=10.0);
                                        let saturation_response =
                                            ui.add(saturation_drag_value.prefix("Saturation: "));
                                        if saturation_response.drag_stopped()
                                            || saturation_response.lost_focus()
                                        {
                                            trigger_regeneration = true;
                                        }

                                        ui.add_space(inner_gap);

                                        // Gain drag input
                                        let gain_drag_value = egui::DragValue::new(&mut self.gain)
                                            .speed(1.0)
                                            .range(1.0..=100.0);
                                        let gain_response =
                                            ui.add(gain_drag_value.prefix("Gain: "));
                                        if gain_response.drag_stopped()
                                            || gain_response.lost_focus()
                                        {
                                            trigger_regeneration = true;
                                        }

                                        ui.add_space(inner_gap);

                                        // Custom resolution checkbox and drag inputs
                                        let old_custom_resolution = self.custom_resolution;
                                        ui.checkbox(&mut self.custom_resolution, "Custom Res");
                                        if self.custom_resolution != old_custom_resolution {
                                            trigger_regeneration = true;
                                        }

                                        if self.custom_resolution {
                                            ui.horizontal(|ui| {
                                                let width_response = ui.add(
                                                    egui::DragValue::new(&mut self.resolution[0])
                                                        // .prefix("Width: ")
                                                        .speed(10.0)
                                                        .range(100.0..=7892.0),
                                                );
                                                ui.label("x");
                                                let height_response = ui.add(
                                                    egui::DragValue::new(&mut self.resolution[1])
                                                        // .prefix("Height: ")
                                                        .speed(10.0)
                                                        .range(100.0..=7992.0),
                                                );
                                                if width_response.drag_stopped()
                                                    || width_response.lost_focus()
                                                    || height_response.drag_stopped()
                                                    || height_response.lost_focus()
                                                {
                                                    trigger_regeneration = true;
                                                }
                                            });
                                        }
                                    });

                                    if trigger_regeneration && !self.is_generating {
                                        self.is_generating = true;
                                        let (sender, receiver) = mpsc::channel();
                                        self.image_receiver = Some(receiver);

                                        let input_path = self.input_path.clone();
                                        let legend = self.legend;
                                        let color_scheme = self.color_scheme;
                                        let win_func = self.win_func;
                                        let scale = self.scale;
                                        let gain = self.gain;
                                        let saturation = self.saturation;
                                        let split_channels = self.split_channels;
                                        let (width, height) = if self.custom_resolution {
                                            (self.resolution[0], self.resolution[1])
                                        } else {
                                            (800, 500)
                                        };
                                        let horizontal = self.horizontal;
                                        let ctx_clone = ctx.clone();

                                        thread::spawn(move || {
                                            let image = utils::generate_spectrogram_in_memory(
                                                &input_path,
                                                legend,
                                                color_scheme,
                                                win_func,
                                                scale,
                                                gain,
                                                saturation,
                                                split_channels,
                                                width,
                                                height,
                                                horizontal,
                                            );
                                            sender.send(image).ok();
                                            ctx_clone.request_repaint(); // Wake up UI thread
                                        });
                                    }
                                });
                            });
                        });
                    });

                if self.is_generating {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        ui.label("Generating...");
                    });
                } else if let Some(texture) = &self.texture {
                    let available_size = ui.available_size();
                    let image_size = texture.size_vec2();

                    let image_aspect = image_size.x / image_size.y;
                    let available_aspect = available_size.x / available_size.y;

                    let new_size = if image_aspect > available_aspect {
                        // Fit to width
                        egui::vec2(available_size.x, available_size.x / image_aspect)
                    } else {
                        // Fit to height
                        egui::vec2(available_size.y * image_aspect, available_size.y)
                    };

                    ui.centered_and_justified(|ui| {
                        if self.custom_resolution {
                            ui.image((texture.id(), new_size));
                        } else {
                            ui.add(egui::Image::from_texture(texture));
                        }
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Failed to generate or load spectrogram.");
                    });
                }
            });
    }
}
