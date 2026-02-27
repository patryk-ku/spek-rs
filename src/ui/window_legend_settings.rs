use eframe::egui;

use crate::settings::AppSettings;

pub fn show(ctx: &egui::Context, is_open: &mut bool, settings: &mut AppSettings) {
    let mut custom_legend_bg_color = settings.custom_legend_bg_color;
    let mut custom_legend_text_color = settings.custom_legend_text_color;
    let mut custom_legend_line_color = settings.custom_legend_line_color;

    let mut changed = false;

    egui::Window::new("Legend Settings")
        .open(is_open)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .resizable(false)
        .collapsible(false)
        .min_width(200.0)
        .max_width(200.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                
                egui::Grid::new("legend_color_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Background Color:");
                        if egui::color_picker::color_edit_button_srgb(ui, &mut custom_legend_bg_color).changed() {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label("Text Color:");
                        if egui::color_picker::color_edit_button_srgb(ui, &mut custom_legend_text_color).changed() {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label("Line Color:");
                        if egui::color_picker::color_edit_button_srgb(ui, &mut custom_legend_line_color).changed() {
                            changed = true;
                        }
                        ui.end_row();
                    });

                ui.add_space(10.0);

                if ui.button("Reset to default").clicked() {
                    custom_legend_bg_color = [0, 0, 0];
                    custom_legend_text_color = [255, 255, 255];
                    custom_legend_line_color = [255, 255, 255];
                    changed = true;
                }

                ui.add_space(10.0);
            });
        });

    if changed {
        settings.custom_legend_bg_color = custom_legend_bg_color;
        settings.custom_legend_text_color = custom_legend_text_color;
        settings.custom_legend_line_color = custom_legend_line_color;
        settings.save();
    }
}
