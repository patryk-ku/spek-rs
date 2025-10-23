use eframe::egui;

pub fn show(ctx: &egui::Context, is_open: &mut bool) {
    egui::Window::new("Keybindings")
        .open(is_open)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .resizable(false)
        .collapsible(false)
        .min_width(250.0)
        .max_width(250.0)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                egui::Grid::new("keybinding_grid")
                    .num_columns(1)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .min_col_width(80.0)
                    .max_col_width(150.0)
                    .show(ui, |ui| {
                        ui.label("Ctrl + O");
                        ui.label("Open File");
                        ui.end_row();

                        ui.label("Ctrl + S");
                        ui.label("Save As");
                        ui.end_row();

                        ui.label("P,   Shift + P");
                        ui.label("Cycle Color Palette");
                        ui.end_row();

                        ui.label("F,   Shift + F");
                        ui.label("Cycle Window Function");
                        ui.end_row();

                        ui.label("A,   Shift + A");
                        ui.label("Cycle Scale");
                        ui.end_row();

                        ui.label("G,   Shift + G");
                        ui.label("Adjust Gain");
                        ui.end_row();

                        ui.label("T,   Shift + T");
                        ui.label("Adjust Saturation");
                        ui.end_row();

                        ui.label("C");
                        ui.label("Toggle Split Channels");
                        ui.end_row();

                        ui.label("ESC");
                        ui.label("Close Application");
                        ui.end_row();

                        ui.label("F1");
                        ui.label("Open Help");
                        ui.end_row();

                        ui.label("F2");
                        ui.label("Open Keybindings");
                        ui.end_row();

                        ui.label("F3");
                        ui.label("Open About");
                        ui.end_row();
                    });
            });
        });
}
