use ffmpeg_sidecar::{self, command::ffmpeg_is_installed};

use crate::ui::FfmpegSetup;
use crate::utils::get_ffmpeg_paths;
use eframe::egui;
use image;

pub fn setup_ffmpeg() -> eframe::Result<()> {
    if ffmpeg_is_installed() {
        println!("FFmpeg is installed.");
        return Ok(());
    }

    let ff_paths = get_ffmpeg_paths();

    if ff_paths.ffmpeg.exists() && ff_paths.ffprobe.exists() {
        println!("FFmpeg found in: {}", ff_paths.directory.display());
        return Ok(());
    }

    println!("FFmpeg is not installed.");

    let dialog_description = format!(
        "FFmpeg is not found. It is required for this application to function. Do you want to download it automatically?\n\nIt will be installed in: {}\n\nDownload may take a few minutes, and the application might appear unresponsive during this time.",
        ff_paths.directory.to_string_lossy()
    );

    let options = {
        let mut viewport = egui::ViewportBuilder::default();
        let icon = {
            let image = image::load_from_memory(include_bytes!("../assets/icon.ico"))
                .expect("Failed to load icon");
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            }
        };
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
        viewport = viewport
            .with_min_inner_size([380.0, 260.0])
            .with_inner_size([380.0, 260.0])
            .with_resizable(false);

        eframe::NativeOptions {
            viewport,
            ..Default::default()
        }
    };

    eframe::run_native(
        "Spek-rs",
        options,
        Box::new(move |_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            // _cc.egui_ctx.set_theme(egui::Theme::Light);
            _cc.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::new(FfmpegSetup::new(dialog_description)))
        }),
    )
    .ok();

    if !ff_paths.ffmpeg.exists() || !ff_paths.ffprobe.exists() {
        panic!(
            "Installation failed. FFmpeg executable not found at expected path: {}",
            ff_paths.directory.display()
        )
    }

    Ok(())
}
