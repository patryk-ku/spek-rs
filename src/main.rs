#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use image;
use std::env;
use std::process::{Command, Stdio};

mod ui;
use ui::MyApp;
mod ffmpeg_setup;
mod legend;
mod palettes;
mod settings;
mod utils;

fn main() -> eframe::Result {
    ffmpeg_setup::setup_ffmpeg()?;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    println!("spek-rs v{}", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = env::args().collect();
    let input_path = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    // Handle multiple files
    if args.len() > 2 {
        let exe_path = env::current_exe().expect("Failed to get current executable path");
        for path in args.iter().skip(2) {
            if let Err(e) = Command::new(&exe_path)
                .arg(path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                eprintln!("Failed to spawn process for {}: {}", path, e);
            }
        }
    }

    let app_settings = settings::AppSettings::load();

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

        if app_settings.save_window_size {
            viewport = viewport.with_inner_size(app_settings.window_size);
        } else {
            // spectogram + legend, spectogram + legend + menu bar
            viewport = viewport.with_inner_size([500.0 + 180.0, 320.0 + 128.0 + 39.0]);
        }
        viewport = viewport
            .with_min_inner_size([500.0 + 180.0, 320.0 + 128.0 + 39.0])
            .with_resizable(true);

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
            Ok(Box::new(MyApp::new(None, input_path, app_settings)))
        }),
    )
}
