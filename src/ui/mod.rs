use eframe::egui::{self, Color32, ColorImage};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::legend;
use crate::settings::AppSettings;
use crate::utils;

mod settings_panel;
mod window_about;

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
    generation_cancel_token: Option<Arc<AtomicBool>>,
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
            generation_cancel_token: None,
        }
    }

    fn regenerate_spectrogram(&mut self, ctx: &egui::Context) {
        if self.input_path.is_none() {
            return;
        }

        if let Some(token) = &self.generation_cancel_token {
            token.store(true, Ordering::Relaxed);
        }

        if self.settings.remember_settings {
            self.settings.save();
        }

        self.is_generating = true;
        let input_path = self.input_path.clone().unwrap();

        let (sender, receiver) = mpsc::channel();
        self.image_receiver = Some(receiver);

        let (width, height) = if self.settings.custom_resolution || self.settings.resize_with_window
        {
            (self.settings.resolution[0], self.settings.resolution[1])
        } else {
            (500, 320)
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
            let legend_color_image = utils::rgba_image_to_color_image(&legend_rgba);

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
        let cancel_token = Arc::new(AtomicBool::new(false));
        self.generation_cancel_token = Some(cancel_token.clone());

        thread::spawn(move || {
            if thread_settings.live_mode {
                utils::stream_spectrogram_frames(
                    sender,
                    &input_path,
                    &thread_settings,
                    width,
                    height,
                    cancel_token,
                );
            } else {
                let image = utils::generate_spectrogram_in_memory(
                    &input_path,
                    &thread_settings,
                    width,
                    height,
                    cancel_token,
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
        let mut trigger_regeneration_due_to_resize = false;
        if self.settings.resize_with_window {
            let inner_size = ctx
                .input(|i| i.viewport().inner_rect)
                .unwrap_or(egui::Rect::ZERO)
                .size();

            let new_width = (inner_size.x - 180.0).max(100.0) as u32;
            let new_height = (inner_size.y - 128.0 - 39.0).max(100.0) as u32;

            let new_res = [new_width, new_height];
            if self.settings.resolution != new_res {
                self.settings.resolution = new_res;
                trigger_regeneration_due_to_resize = true;
            }
        }

        if trigger_regeneration_due_to_resize {
            self.regenerate_spectrogram(ctx);
        }

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
                            ui.strong("Spek-rs");
                            ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                            ui.add_space(4.0);
                            self.show_settings_panel(ctx, ui);
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
            window_about::show(ctx, &mut self.about_window_open);
        }
    }
}
