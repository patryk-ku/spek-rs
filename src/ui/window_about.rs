use eframe::egui;

pub fn show(ctx: &egui::Context, is_open: &mut bool) {
    egui::Window::new("About Spek-rs")
        .open(is_open)
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
