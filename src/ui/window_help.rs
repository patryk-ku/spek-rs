use eframe::egui;

pub fn show(ctx: &egui::Context, is_open: &mut bool) {
    egui::Window::new("Help")
        .open(is_open)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .resizable(false)
        .collapsible(false)
        .min_width(400.0)
        .max_width(400.0)
        .show(ctx, |ui| {
            ui.label("This application is a GUI for ffmpeg's showspectrumpic function, allowing you to generate spectrograms from audio files.");

            ui.add_space(5.0);

            ui.label("For a detailed explanation of the available options and their functionalities, please refer to the official ffmpeg documentation:");
            ui.hyperlink("https://ffmpeg.org/ffmpeg-filters.html#showspectrumpic");

            ui.add_space(5.0);

            ui.label("For more information, to report bugs, or to share ideas, please visit the program's repository:");
            ui.hyperlink("https://github.com/patryk-ku/spek-rs");

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            ui.label("About Live Mode:");
            ui.label("Live mode generates the spectrogram in real-time, but the visual quality is lower than a normal generation. This is because it uses a different, less precise method to generate the image on the fly. This option is useful when you want to quickly preview the beginning of a file, for example, to check if a FLAC file is a genuine lossless file or an upscaled lossy file.");
            ui.add_space(2.0);
        });
}
