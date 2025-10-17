use crate::utils::AudioInfo;
use ab_glyph::{FontVec, PxScale};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_line_segment_mut, draw_text_mut};
use imageproc::rect::Rect;

pub const TOP_MARGIN: u32 = 64;
pub const BOTTOM_MARGIN: u32 = 64;
pub const LEFT_MARGIN: u32 = 141;
pub const RIGHT_MARGIN: u32 = 141;

// Helper function for linear interpolation of color components
fn lerp(start: u8, end: u8, t: f32) -> u8 {
    (start as f32 * (1.0 - t) + end as f32 * t) as u8
}

pub fn draw_gradient_line_mut(
    image: &mut RgbaImage,
    start: (f32, f32),
    end: (f32, f32),
    colors: &[Rgba<u8>; 5],
    thickness: u32,
) {
    let (x0, y0) = start;
    let (x1, y1) = end;

    let dx = x1 - x0;
    let dy = y1 - y0;

    let steps = dx.abs().max(dy.abs());

    if steps < 1.0 {
        if (x0 as u32) < image.width() && (y0 as u32) < image.height() {
            let color = colors[0];
            for i in 0..thickness {
                let x = (x0 as u32) + i;
                if x < image.width() {
                    image.put_pixel(x, y0 as u32, color);
                }
            }
        }
        return;
    }

    for i in 0..=steps as i32 {
        let t = i as f32 / steps;
        let x = (x0 + t * dx).round() as u32;
        let y = (y0 + t * dy).round() as u32;

        // Determine which two colors to interpolate between
        let num_segments = (colors.len() - 1) as f32;
        let segment_float = t * num_segments;
        let color_index = (segment_float.floor() as usize).min(colors.len() - 2);

        let segment_t = segment_float - color_index as f32;

        let start_color = colors[color_index];
        let end_color = colors[color_index + 1];

        // Interpolate each color channel
        let r = lerp(start_color[0], end_color[0], segment_t);
        let g = lerp(start_color[1], end_color[1], segment_t);
        let b = lerp(start_color[2], end_color[2], segment_t);
        let a = lerp(start_color[3], end_color[3], segment_t);
        let color = Rgba([r, g, b, a]);

        // Draw a horizontal line for thickness
        for i in 0..thickness {
            let current_x = x + i;
            if current_x < image.width() && y < image.height() {
                image.put_pixel(current_x, y, color);
            }
        }
    }
}

fn draw_time_scale(
    image: &mut RgbaImage,
    spec_width: u32,
    spec_height: u32,
    duration: f64,
    font: &FontVec,
    scale: PxScale,
    color: Rgba<u8>,
    is_top: bool,
    draw_labels: bool,
) {
    let num_ticks = 10;
    for i in 0..=num_ticks {
        let fraction = i as f32 / num_ticks as f32;
        let x = LEFT_MARGIN as f32 + fraction * spec_width as f32;

        let (y_start, y_end, label_y) = if is_top {
            let y_start = TOP_MARGIN as f32 - 6.0;
            let y_end = TOP_MARGIN as f32 - 1.0;
            let (_, text_height) = imageproc::drawing::text_size(scale, font, "0");
            (y_start, y_end, y_start - text_height as f32 - 4.0)
        } else {
            let y_start = TOP_MARGIN as f32 + spec_height as f32;
            let y_end = y_start + 5.0;
            (y_start, y_end, y_end + 8.0)
        };

        draw_line_segment_mut(image, (x, y_start), (x, y_end), color);

        if draw_labels {
            let time_sec = duration * fraction as f64;
            let minutes = (time_sec / 60.0).floor() as u32;
            let seconds = (time_sec % 60.0).floor() as u32;
            let label = format!("{}:{:02}", minutes, seconds);
            let (text_width, _) = imageproc::drawing::text_size(scale, font, &label);
            draw_text_mut(
                image,
                color,
                (x - text_width as f32 / 2.0) as i32,
                label_y as i32,
                scale,
                font,
                &label,
            );
        }
    }
}

fn draw_freq_scale(
    image: &mut RgbaImage,
    spec_width: u32,
    spec_height: u32,
    sample_rate: u32,
    font: &FontVec,
    scale: PxScale,
    color: Rgba<u8>,
) {
    let max_freq_khz = (sample_rate / 2) as f32 / 1000.0;
    let num_ticks = 10;

    for i in 0..=num_ticks {
        let fraction = i as f32 / num_ticks as f32;
        let y = (TOP_MARGIN - 1) as f32 + (1.0 - fraction) * (spec_height + 1) as f32;

        // Left ticks
        let x_start_left = LEFT_MARGIN as f32 - 6.0;
        let x_end_left = LEFT_MARGIN as f32 - 1.0;
        draw_line_segment_mut(image, (x_start_left, y), (x_end_left, y), color);

        // Right ticks
        let x_start_right = LEFT_MARGIN as f32 + spec_width as f32 + 1.0;
        let x_end_right = x_start_right + 5.0;
        draw_line_segment_mut(image, (x_start_right, y), (x_end_right, y), color);

        // Freq labels
        let freq_khz = fraction * max_freq_khz;
        let label = format!("{:.0} kHz", freq_khz);
        let (text_width, text_height) = imageproc::drawing::text_size(scale, font, &label);
        draw_text_mut(
            image,
            color,
            (x_start_left - text_width as f32 - 8.0) as i32,
            (y - text_height as f32 / 2.0) as i32,
            scale,
            font,
            &label,
        );
    }
}

fn draw_dbfs_scale(
    image: &mut RgbaImage,
    spec_width: u32,
    spec_height: u32,
    font: &FontVec,
    scale: PxScale,
    color: Rgba<u8>,
) {
    let db_range: f32 = -120.0;
    let num_ticks = 10;
    let gradient_x = LEFT_MARGIN as f32 + spec_width as f32 + 34.0;
    let gradient_width = 10.0;
    let label_x = gradient_x + gradient_width + 5.0;

    for i in 0..=num_ticks {
        let fraction = i as f32 / num_ticks as f32;
        let y = (TOP_MARGIN - 1) as f32 + (1.0 - fraction) * (spec_height + 1) as f32;

        let db_level = (fraction - 1.0) * db_range.abs();
        let label = format!("{:.0}", db_level);

        let (_, text_height) = imageproc::drawing::text_size(scale, font, &label);
        draw_text_mut(
            image,
            color,
            label_x as i32,
            (y - text_height as f32 / 2.0) as i32,
            scale,
            font,
            &label,
        );
    }
}

/// Creates an image with a legend template.
/// The spectrogram itself will be drawn on top of this template later.
pub fn draw_legend(
    spec_width: u32,
    spec_height: u32,
    filename: &str,
    ffmpeg_settings: &str,
    audio_info: Option<AudioInfo>,
) -> RgbaImage {
    let final_width = spec_width + LEFT_MARGIN + RIGHT_MARGIN;
    let final_height = spec_height + TOP_MARGIN + BOTTOM_MARGIN;

    // Create a new image with a black background
    let mut image = RgbaImage::new(final_width, final_height);
    draw_filled_rect_mut(
        &mut image,
        Rect::at(0, 0).of_size(final_width, final_height),
        Rgba([0u8, 0u8, 0u8, 255u8]),
    );

    // Draw spec borders
    let white = Rgba([255u8, 255u8, 255u8, 255u8]);
    let top_left = (LEFT_MARGIN as f32 - 1.0, TOP_MARGIN as f32 - 1.0);
    let top_right = ((LEFT_MARGIN + spec_width) as f32, TOP_MARGIN as f32 - 1.0);
    let bottom_left = (LEFT_MARGIN as f32 - 1.0, (TOP_MARGIN + spec_height) as f32);
    let bottom_right = (
        (LEFT_MARGIN + spec_width) as f32,
        (TOP_MARGIN + spec_height) as f32,
    );
    draw_line_segment_mut(&mut image, top_left, top_right, white);
    draw_line_segment_mut(&mut image, top_right, bottom_right, white);
    draw_line_segment_mut(&mut image, bottom_right, bottom_left, white);
    draw_line_segment_mut(&mut image, bottom_left, top_left, white);

    // Load font
    let font_data = include_bytes!("../assets/DejaVuSans.ttf");
    let font =
        FontVec::try_from_vec(font_data.to_vec()).expect("Error constructing Font from bytes");

    let font_normal = PxScale::from(16.0);
    let font_small = PxScale::from(13.0);
    let font_scales = PxScale::from(14.0);
    let text_color = Rgba([255u8, 255u8, 255u8, 255u8]);

    // Draw filename
    draw_text_mut(
        &mut image,
        text_color,
        LEFT_MARGIN as i32,
        5,
        font_normal,
        &font,
        filename,
    );

    // Draw ffmpeg settings
    draw_text_mut(
        &mut image,
        text_color,
        LEFT_MARGIN as i32,
        23,
        font_normal,
        &font,
        ffmpeg_settings,
    );

    // Draw app name and version in top-right corner
    let app_info = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let (text_width, _) = imageproc::drawing::text_size(font_normal, &font, &app_info);
    draw_text_mut(
        &mut image,
        text_color,
        (final_width - text_width - 10) as i32,
        5,
        font_normal,
        &font,
        &app_info,
    );

    // dBFS gradient (right)
    let dbfs_label = "dBFS";
    let (text_width, _) = imageproc::drawing::text_size(font_small, &font, dbfs_label);
    let gradient_center_x = (LEFT_MARGIN + spec_width + 34 + 5) as i32;
    draw_text_mut(
        &mut image,
        text_color,
        gradient_center_x - (text_width / 2) as i32,
        (TOP_MARGIN + spec_height + 25) as i32,
        font_small,
        &font,
        dbfs_label,
    );

    // Time scale (bottom)
    draw_text_mut(
        &mut image,
        text_color,
        (LEFT_MARGIN + spec_width / 2) as i32,
        (TOP_MARGIN + spec_height + 35) as i32,
        font_normal,
        &font,
        "Time",
    );

    // dBFS vertical gradient line on the right
    // TMP: only "Intensity" palette
    let gradient_colors = [
        Rgba([253, 254, 249, 255]),
        Rgba([252, 175, 0, 255]),
        Rgba([190, 2, 39, 255]),
        Rgba([69, 0, 111, 255]),
        Rgba([0, 0, 0, 255]),
    ];

    let line_x = (LEFT_MARGIN + spec_width + 34) as f32;
    let start_point = (line_x, TOP_MARGIN as f32);
    let end_point = (line_x, (TOP_MARGIN + spec_height) as f32);
    draw_gradient_line_mut(&mut image, start_point, end_point, &gradient_colors, 10);

    if let Some(info) = audio_info {
        draw_time_scale(
            &mut image,
            spec_width,
            spec_height,
            info.duration,
            &font,
            font_scales,
            text_color,
            false, // bottom
            true,  // draw_labels
        );
        draw_time_scale(
            &mut image,
            spec_width,
            spec_height,
            info.duration,
            &font,
            font_scales,
            text_color,
            true,  // top
            false, // draw_labels
        );
        draw_freq_scale(
            &mut image,
            spec_width,
            spec_height,
            info.sample_rate,
            &font,
            font_scales,
            text_color,
        );
    }

    draw_dbfs_scale(
        &mut image,
        spec_width,
        spec_height,
        &font,
        font_scales,
        text_color,
    );
    image
}
