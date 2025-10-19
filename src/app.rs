use eframe::egui::{self, Color32, ColorImage};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::legend;
use crate::settings::{AppSettings, SpectogramWinFunc, SpectrogramColorScheme, SpectrogramScale};
use crate::utils;
use crate::utils::rgba_image_to_color_image;

pub struct MyApp {
    texture: Option<egui::TextureHandle>,
    final_image: Option<eframe::egui::ColorImage>,
    input_path: Option<String>,
    settings: AppSettings,
    is_generating: bool,
    image_receiver: Option<Receiver<Option<ColorImage>>>,
    spectrogram_slice_position: usize,
    about_window_open: bool,
    audio_info: Option<utils::AudioInfo>,
}

impl MyApp {
    pub fn new(image: Option<ColorImage>, input_path: Option<String>) -> Self {
        let audio_info = if let Some(path) = &input_path {
            utils::get_audio_info(path)
        } else {
            None
        };
        Self {
            texture: None,
            final_image: image,
            input_path,
            settings: AppSettings::load(),
            is_generating: false,
            image_receiver: None,
            spectrogram_slice_position: 0,
            about_window_open: false,
            audio_info,
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

        let (sender, receiver) = mpsc::channel();
        self.image_receiver = Some(receiver);

        let (width, height) = if self.settings.custom_resolution {
            (self.settings.resolution[0], self.settings.resolution[1])
        } else {
            (800, 500)
        };

        let use_custom_legend =
            self.settings.legend && (self.settings.custom_legend || self.settings.live_mode);
        let mut thread_settings = self.settings.clone();

        if use_custom_legend {
            self.spectrogram_slice_position = 0;
            let filename = std::path::Path::new(&input_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown File");
            let ffmpeg_settings = format!(
                "{}, {}, {}",
                self.settings.win_func.to_string(),
                self.settings.scale.to_string(),
                self.settings.color_scheme.to_string()
            );
            let audio_info = self.audio_info.clone();

            let legend_rgba = legend::draw_legend(
                width,
                height,
                filename,
                &ffmpeg_settings,
                audio_info,
                self.settings.saturation,
                self.settings.color_scheme,
                self.settings.split_channels,
            );
            let legend_color_image = rgba_image_to_color_image(&legend_rgba);

            self.final_image = Some(legend_color_image.clone());
            self.texture =
                Some(ctx.load_texture("spectrogram", legend_color_image, Default::default()));

            // Force ffmpeg legend off when using custom one
            thread_settings.legend = false;
        } else if self.settings.live_mode {
            // In live mode, even without a legend, we need a canvas to draw on.
            self.spectrogram_slice_position = 0;
            let empty_canvas = ColorImage::new(
                [width as usize, height as usize],
                vec![Color32::BLACK; (width * height) as usize],
            );
            self.final_image = Some(empty_canvas.clone());
            self.texture = Some(ctx.load_texture("spectrogram", empty_canvas, Default::default()));
            thread_settings.legend = false;
        } else {
            self.final_image = None;
            self.texture = None;
        }

        let ctx_clone = ctx.clone();
        thread::spawn(move || {
            if thread_settings.live_mode {
                utils::stream_spectrogram_frames(
                    sender,
                    &input_path,
                    &thread_settings,
                    width,
                    height,
                );
            } else {
                let image = utils::generate_spectrogram_in_memory(
                    &input_path,
                    &thread_settings,
                    width,
                    height,
                );
                if let Some(img) = image {
                    sender.send(Some(img)).ok();
                }
            }
            ctx_clone.request_repaint();
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let use_custom_legend =
            self.settings.legend && (self.settings.custom_legend || self.settings.live_mode);

        if self.is_generating {
            if let Some(receiver) = &self.image_receiver {
                if self.settings.live_mode {
                    // Live mode (always custom legend): receive slices and draw them
                    for slice in receiver.try_iter() {
                        if let Some(slice) = slice {
                            if let Some(image) = self.final_image.as_mut() {
                                let slice_width = slice.width();

                                let (spec_width, x_offset, y_offset) = if use_custom_legend {
                                    (
                                        image.width()
                                            - (legend::LEFT_MARGIN as usize
                                                + legend::RIGHT_MARGIN as usize),
                                        legend::LEFT_MARGIN as usize,
                                        legend::TOP_MARGIN as usize,
                                    )
                                } else {
                                    (image.width(), 0, 0)
                                };

                                if self.spectrogram_slice_position + slice_width <= spec_width {
                                    for y in 0..slice.height() {
                                        for x in 0..slice_width {
                                            let dest_x =
                                                self.spectrogram_slice_position + x + x_offset;
                                            let dest_y = y + y_offset;
                                            if dest_x < image.width() && dest_y < image.height() {
                                                image[(dest_x, dest_y)] = slice[(x, y)];
                                            }
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
                } else {
                    // Normal mode: receive the full spectrogram
                    if let Ok(maybe_image) = receiver.try_recv() {
                        self.is_generating = false;
                        self.image_receiver = None;
                        if let Some(new_spectrogram) = maybe_image {
                            if use_custom_legend {
                                // Composite onto custom legend
                                if let Some(final_image) = self.final_image.as_mut() {
                                    for y in 0..new_spectrogram.height() {
                                        for x in 0..new_spectrogram.width() {
                                            let dest_x = x + legend::LEFT_MARGIN as usize;
                                            let dest_y = y + legend::TOP_MARGIN as usize;
                                            if dest_x < final_image.width()
                                                && dest_y < final_image.height()
                                            {
                                                final_image[(dest_x, dest_y)] =
                                                    new_spectrogram[(x, y)];
                                            }
                                        }
                                    }
                                    self.texture = Some(ctx.load_texture(
                                        "spectrogram",
                                        final_image.clone(),
                                        Default::default(),
                                    ));
                                }
                            } else {
                                // Display ffmpeg-generated image directly
                                self.texture = Some(ctx.load_texture(
                                    "spectrogram",
                                    new_spectrogram.clone(),
                                    Default::default(),
                                ));
                                self.final_image = Some(new_spectrogram);
                            }
                        }
                    }
                }
                ctx.request_repaint();
            }
        }

        if self.texture.is_none() && self.final_image.is_some() {
            if let Some(image) = self.final_image.as_ref() {
                self.texture =
                    Some(ctx.load_texture("spectrogram", image.clone(), Default::default()));
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
                                        self.audio_info = utils::get_audio_info(
                                            self.input_path.as_ref().unwrap(),
                                        );
                                        trigger_regeneration = true;
                                    }
                                }

                                if self.final_image.is_some() {
                                    if ui.button("Save As...").clicked() {
                                        if let Some(input_path) = &self.input_path {
                                            utils::save_image(&self.final_image, input_path);
                                        }
                                    }
                                }
                            });

                            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                                ui.horizontal(|ui| {
                                    // Start generating on first launch
                                    if self.final_image.is_none() && !self.is_generating {
                                        if self.input_path.is_some() {
                                            trigger_regeneration = true;
                                        }
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
                                                    let mut dummy_true = true;
                                                    let mut dummy_false = false;

                                                    // Legend section
                                                    if ui
                                                        .checkbox(
                                                            &mut self.settings.legend,
                                                            "Draw legend",
                                                        )
                                                        .changed()
                                                    {
                                                        trigger_regeneration = true;
                                                    }

                                                    if self.settings.legend {
                                                        if self.settings.live_mode {
                                                            ui.add_enabled(
                                                                false,
                                                                egui::Checkbox::new(
                                                                    &mut dummy_true,
                                                                    "Custom Legend",
                                                                ),
                                                            );
                                                        } else {
                                                            if ui
                                                                .checkbox(
                                                                    &mut self
                                                                        .settings
                                                                        .custom_legend,
                                                                    "Custom Legend",
                                                                )
                                                                .changed()
                                                            {
                                                                trigger_regeneration = true;
                                                            }
                                                        }

                                                        if self.settings.custom_legend
                                                            || (self.settings.legend
                                                                && self.settings.live_mode)
                                                        {
                                                            if ui
                                                                .button("Legend settings")
                                                                .clicked()
                                                            {
                                                                // self.legend_window_open = true;
                                                                ui.close();
                                                            }
                                                        }

                                                        ui.separator();
                                                    }

                                                    // Split channels checkbox
                                                    let has_multiple_channels =
                                                        match &self.audio_info {
                                                            Some(info) => info.channels > 1,
                                                            None => false,
                                                        };

                                                    if has_multiple_channels {
                                                        if ui
                                                            .checkbox(
                                                                &mut self.settings.split_channels,
                                                                "Split channels",
                                                            )
                                                            .changed()
                                                        {
                                                            trigger_regeneration = true;
                                                        }
                                                    } else {
                                                        ui.add_enabled(
                                                            false,
                                                            egui::Checkbox::new(
                                                                &mut dummy_false,
                                                                "Split channels",
                                                            ),
                                                        );
                                                    }

                                                    // Horizontal spectogram
                                                    if self.settings.live_mode
                                                        || self.settings.custom_legend
                                                    {
                                                        ui.add_enabled(
                                                            false,
                                                            egui::Checkbox::new(
                                                                &mut dummy_false,
                                                                "Horizontal",
                                                            ),
                                                        );
                                                    } else {
                                                        if ui
                                                            .checkbox(
                                                                &mut self.settings.horizontal,
                                                                "Horizontal",
                                                            )
                                                            .changed()
                                                        {
                                                            trigger_regeneration = true;
                                                        }
                                                    }

                                                    ui.add_enabled(
                                                        false,
                                                        egui::Checkbox::new(
                                                            &mut dummy_false,
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

                                                    ui.separator();

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

                                                    if ui.button("Keybindings").clicked() {
                                                        // self.keybindings_window_open = true;
                                                        ui.close();
                                                    }

                                                    if ui.button("Help").clicked() {
                                                        // self.help_window_open = true;
                                                        ui.close();
                                                    }

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
                                        egui::ComboBox::from_label("Color:")
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
