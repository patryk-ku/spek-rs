#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use std::env;

mod app;
mod utils;
use app::MyApp;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let args: Vec<String> = env::args().collect();
    let input_path = match args.get(1) {
        Some(path) => path.clone(),
        None => {
            eprintln!("Usage: spek-rs <path_to_media_file>");
            std::process::exit(1);
        }
    };

    let spectrogram_image = utils::generate_spectrogram_in_memory(
        &input_path,
        true,
        utils::SpectrogramColorScheme::Intensity,
        utils::SpectogramWinFunc::Hann,
        utils::SpectrogramScale::Log,
        1.0,
        1.0,
        false,
        800,
        500,
        false,
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0 + 282.0, 500.0 + 128.0 + 39.0]) // spectogram + legend, spectogram + legend + menu bar
            .with_min_inner_size([800.0 + 282.0, 500.0 + 128.0 + 39.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Spek-rs",
        options,
        Box::new(move |_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            _cc.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::new(MyApp::new(spectrogram_image, input_path)))
        }),
    )
}
