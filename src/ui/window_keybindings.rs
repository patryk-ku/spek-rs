use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn show(ctx: &egui::Context, is_open: &mut bool) {
    egui::Window::new("Keybindings")
        .open(is_open)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .resizable(false)
        .collapsible(false)
        .min_width(270.0)
        .max_width(270.0)
        .show(ctx, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .column(Column::auto().at_least(90.0))
                .column(Column::remainder())
                .body(|body| {
                    let rows = [
                        ("Ctrl + O", "Open File"),
                        ("Ctrl + S", "Save As"),
                        ("P,   Shift + P", "Cycle Color Palette"),
                        ("F,   Shift + F", "Cycle Window Function"),
                        ("A,   Shift + A", "Cycle Scale"),
                        ("G,   Shift + G", "Adjust Gain"),
                        ("T,   Shift + T", "Adjust Saturation"),
                        ("C", "Toggle Split Channels"),
                        ("ESC", "Close Application"),
                        ("F1", "Open Help"),
                        ("F2", "Open Keybindings"),
                        ("F3", "Open About"),
                    ];
                    body.rows(18.0, rows.len(), |mut row| {
                        let index = row.index();
                        let (shortcut, desc) = rows[index];
                        row.col(|ui| {
                            ui.label(shortcut);
                        });
                        row.col(|ui| {
                            ui.label(desc);
                        });
                    });
                });
        });
}
