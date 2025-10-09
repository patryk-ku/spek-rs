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
        1.0,
        false,
        700,
        500,
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([982.0, 628.0 + 29.0])
            .with_min_inner_size([982.0, 628.0 + 29.0])
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
