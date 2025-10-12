use eframe::egui::{self, Color32, ColorImage};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::settings::{AppSettings, SpectogramWinFunc, SpectrogramColorScheme, SpectrogramScale};
use crate::utils;

pub struct MyApp {
    texture: Option<egui::TextureHandle>,
    image: Option<eframe::egui::ColorImage>,
    input_path: Option<String>,
    settings: AppSettings,
    is_generating: bool,
    image_receiver: Option<Receiver<Option<ColorImage>>>,
    spectrogram_slice_position: usize,
    about_window_open: bool,
}

impl MyApp {
    pub fn new(image: Option<ColorImage>, input_path: Option<String>) -> Self {
        Self {
            texture: None,
            image,
            input_path,
            settings: AppSettings::load(),
            is_generating: false,
            image_receiver: None,
            spectrogram_slice_position: 0,
            about_window_open: false,
        }
    }

    fn regenerate_spectrogram(&mut self, ctx: &egui::Context) {
        if self.input_path.is_none() {
            return;
        }

        if self.settings.remember_settings {
            self.settings.save();
        }

        self.is_generating = true;
        let input_path = self.input_path.clone().unwrap();
        let settings = self.settings.clone();

        let (sender, receiver) = mpsc::channel();
        self.image_receiver = Some(receiver);

        let (width, height) = if self.settings.custom_resolution {
            (self.settings.resolution[0], self.settings.resolution[1])
        } else {
            (800, 500)
        };

        if self.settings.live_mode {
            self.spectrogram_slice_position = 0;
            let new_image = ColorImage::new(
                [width as usize, height as usize],
                vec![Color32::BLACK; (width * height) as usize],
            );
            self.texture =
                Some(ctx.load_texture("spectrogram", new_image.clone(), Default::default()));
            self.image = Some(new_image);

            thread::spawn(move || {
                utils::stream_spectrogram_frames(sender, &input_path, &settings, width, height);
            });
        } else {
            let ctx_clone = ctx.clone();
            thread::spawn(move || {
                let image =
                    utils::generate_spectrogram_in_memory(&input_path, &settings, width, height);
                sender.send(image).ok();
                ctx_clone.request_repaint(); // Wake up UI thread
            });
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.settings.live_mode {
            if self.is_generating {
                if let Some(receiver) = &self.image_receiver {
                    for slice in receiver.try_iter() {
                        if let Some(slice) = slice {
                            if let Some(image) = self.image.as_mut() {
                                let slice_width = slice.width();
                                let image_width = image.width();
                                // println!(
                                //     "spectrogram_slice_position: {}",
                                //     self.spectrogram_slice_position
                                // );
                                if self.spectrogram_slice_position + slice_width <= image_width {
                                    for y in 0..image.height() {
                                        for x in 0..slice_width {
                                            image[(self.spectrogram_slice_position + x, y)] =
                                                slice[(x, y)];
                                        }
                                    }
                                    if let Some(texture) = self.texture.as_mut() {
                                        texture.set(image.clone(), Default::default());
                                    }
                                    self.spectrogram_slice_position += slice_width;
                                }
                            }
                        }
                    }

                    // A bit of a hack to check if the channel is disconnected
                    if let Err(mpsc::TryRecvError::Disconnected) = receiver.try_recv() {
                        self.is_generating = false;
                        self.image_receiver = None;
                    }
                }
                ctx.request_repaint();
            }
        } else {
            // Check for a newly generated image from the background thread
            if let Some(receiver) = &self.image_receiver {
                if let Ok(maybe_image) = receiver.try_recv() {
                    self.is_generating = false;
                    self.image_receiver = None; // done with this receiver
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
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::BLACK))
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
                            let mut trigger_regeneration = false;

                            ui.strong("Spek-rs");
                            ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                            ui.add_space(inner_gap);

                            ui.add_enabled_ui(!self.is_generating, |ui| {
                                if ui.button("Open File...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                                        self.input_path = Some(path.display().to_string());
                                        trigger_regeneration = true;
                                    }
                                }

                                if self.image.is_some() {
                                    if ui.button("Save As...").clicked() {
                                        if let Some(input_path) = &self.input_path {
                                            utils::save_image(&self.image, input_path);
                                        }
                                    }
                                }
                            });

                            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                                ui.horizontal(|ui| {
                                    // Start generating on first launch
                                    if self.image.is_none() && !self.is_generating {
                                        trigger_regeneration = true;
                                    }

                                    ui.add_enabled_ui(!self.is_generating, |ui| {
                                        // More options menu
                                        let more_button = ui.button("More...");
                                        egui::Popup::menu(&more_button)
                                            .gap(6.0)
                                            .align(egui::RectAlign::BOTTOM_END)
                                            .close_behavior(
                                                egui::PopupCloseBehavior::CloseOnClickOutside,
                                            )
                                            .show(|ui| {
                                                ui.add_enabled_ui(!self.is_generating, |ui| {
                                                    let mut useless_bool = false;

                                                    // Legend checkbox
                                                    if !self.settings.live_mode {
                                                        if ui
                                                            .checkbox(
                                                                &mut self.settings.legend,
                                                                "Draw legend",
                                                            )
                                                            .changed()
                                                        {
                                                            trigger_regeneration = true;
                                                        }
                                                    } else {
                                                        ui.add_enabled(
                                                            false,
                                                            egui::Checkbox::new(
                                                                &mut useless_bool,
                                                                "Draw legend",
                                                            ),
                                                        );
                                                    }

                                                    // Split channels checkbox
                                                    if ui
                                                        .checkbox(
                                                            &mut self.settings.split_channels,
                                                            "Split channels",
                                                        )
                                                        .changed()
                                                    {
                                                        trigger_regeneration = true;
                                                    }

                                                    // Horizontal spectogram
                                                    if !self.settings.live_mode {
                                                        if ui
                                                            .checkbox(
                                                                &mut self.settings.horizontal,
                                                                "Horizontal",
                                                            )
                                                            .changed()
                                                        {
                                                            trigger_regeneration = true;
                                                        }
                                                    } else {
                                                        ui.add_enabled(
                                                            false,
                                                            egui::Checkbox::new(
                                                                &mut useless_bool,
                                                                "Horizontal",
                                                            ),
                                                        );
                                                    }

                                                    ui.add_enabled(
                                                        false,
                                                        egui::Checkbox::new(
                                                            &mut useless_bool,
                                                            "Resize with window",
                                                        ),
                                                    );

                                                    // Live mode
                                                    if ui
                                                        .checkbox(
                                                            &mut self.settings.live_mode,
                                                            "Live mode (WIP)",
                                                        )
                                                        .changed()
                                                    {
                                                        trigger_regeneration = true;
                                                    }

                                                    if ui
                                                        .checkbox(
                                                            &mut self.settings.remember_settings,
                                                            "Remember settings",
                                                        )
                                                        .changed()
                                                    {
                                                        self.settings.save();
                                                    }

                                                    // Reset settings button
                                                    if ui.button("Reset settings").clicked() {
                                                        ui.close();
                                                        self.settings = AppSettings::default();
                                                        trigger_regeneration = true;
                                                    }

                                                    ui.separator();

                                                    if ui.button("About").clicked() {
                                                        self.about_window_open = true;
                                                        ui.close();
                                                    }
                                                });
                                            });

                                        ui.add_space(inner_gap);

                                        // Scale combobox
                                        let old_scale = self.settings.scale;
                                        egui::ComboBox::from_label("Scale:")
                                            .selected_text(self.settings.scale.to_string())
                                            .width(55.0)
                                            .show_ui(ui, |ui| {
                                                for scale in SpectrogramScale::VALUES {
                                                    ui.selectable_value(
                                                        &mut self.settings.scale,
                                                        scale,
                                                        scale.to_string(),
                                                    );
                                                }
                                            });
                                        if self.settings.scale != old_scale {
                                            trigger_regeneration = true;
                                        }

                                        ui.add_space(inner_gap);

                                        // Window function combobox
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
                                            trigger_regeneration = true;
                                        }

                                        ui.add_space(inner_gap);

                                        // Color scheme combobox
                                        let old_color_scheme = self.settings.color_scheme;
                                        egui::ComboBox::from_label("Colors:")
                                            .selected_text(self.settings.color_scheme.to_string())
                                            .width(80.0)
                                            .height(600.0)
                                            .show_ui(ui, |ui| {
                                                for color in SpectrogramColorScheme::VALUES {
                                                    ui.selectable_value(
                                                        &mut self.settings.color_scheme,
                                                        color,
                                                        color.to_string(),
                                                    );
                                                }
                                            });
                                        if self.settings.color_scheme != old_color_scheme {
                                            trigger_regeneration = true;
                                        }

                                        ui.add_space(inner_gap);

                                        // Saturation drag input
                                        let saturation_drag_value =
                                            egui::DragValue::new(&mut self.settings.saturation)
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
                                        let gain_drag_value =
                                            egui::DragValue::new(&mut self.settings.gain)
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
                                        if ui
                                            .checkbox(
                                                &mut self.settings.custom_resolution,
                                                "Custom Res",
                                            )
                                            .changed()
                                        {
                                            trigger_regeneration = true;
                                        }

                                        if self.settings.custom_resolution {
                                            ui.horizontal(|ui| {
                                                let width_response = ui.add(
                                                    egui::DragValue::new(
                                                        &mut self.settings.resolution[0],
                                                    )
                                                    .prefix("w: ")
                                                    .suffix("px")
                                                    .speed(10.0)
                                                    .range(100.0..=7892.0),
                                                );
                                                ui.label("x");
                                                let height_response = ui.add(
                                                    egui::DragValue::new(
                                                        &mut self.settings.resolution[1],
                                                    )
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
                                                    trigger_regeneration = true;
                                                }
                                            });
                                        }
                                    });

                                    if trigger_regeneration && !self.is_generating {
                                        self.regenerate_spectrogram(&ctx);
                                    }
                                });
                            });
                        });
                    });

                if self.is_generating && !self.settings.live_mode {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        // ui.label("Generating...");
                    });
                }

                if let Some(texture) = &self.texture {
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
                        if self.settings.custom_resolution {
                            ui.image((texture.id(), new_size));
                        } else {
                            ui.add(egui::Image::from_texture(texture));
                        }
                    });
                } else if !self.is_generating {
                    ui.centered_and_justified(|ui| {
                        if self.input_path.is_some() {
                            ui.label("Failed to generate or load spectrogram.");
                        } else {
                            ui.label("Open a file to begin.");
                        }
                    });
                }
            });

        if self.about_window_open {
            egui::Window::new("About Spek-rs")
                .open(&mut self.about_window_open)
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ctx.content_rect().center())
                .resizable(false)
                .collapsible(false)
                .min_width(280.0)
                .max_width(280.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(format!("Spek-rs v{}", env!("CARGO_PKG_VERSION")));
                        ui.add_space(10.0);
                        ui.label("Copyright © 2025 Patryk Kurdziel");
                        ui.label("Released under the MIT License.");
                        ui.add_space(10.0);
                        ui.hyperlink("https://github.com/patryk-ku/spek-rs");
                    });
                });
        }
    }
}
