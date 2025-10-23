use eframe::egui;

pub fn show(ctx: &egui::Context, is_open: &mut bool) {
    egui::Window::new("Legend Settings")
        .open(is_open)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .resizable(false)
        .collapsible(false)
        .min_width(280.0)
        .max_width(280.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(
                    "This functionality is a work in progress and will be implemented in the future.",
                );
                ui.add_space(10.0);
            });
        });
}
