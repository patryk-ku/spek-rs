#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use std::env;
use std::process::{Command, Stdio};

mod app;
mod settings;
mod utils;
use app::MyApp;

fn main() -> eframe::Result {
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // .with_inner_size([800.0 + 282.0, 500.0 + 128.0 + 39.0])
            .with_min_inner_size([800.0 + 282.0, 500.0 + 128.0 + 39.0]) // spectogram + legend, spectogram + legend + menu bar
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Spek-rs",
        options,
        Box::new(move |_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            _cc.egui_ctx.set_theme(egui::Theme::Dark);
            // Ok(Box::new(MyApp::new(spectrogram_image, input_path)))
            Ok(Box::new(MyApp::new(None, input_path)))
        }),
    )
}
