use crate::palettes;
use crate::utils::AudioInfo;
use ab_glyph::{FontVec, PxScale};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_line_segment_mut, draw_text_mut};
use imageproc::rect::Rect;

pub const TOP_MARGIN: u32 = 64;
pub const BOTTOM_MARGIN: u32 = 64;
pub const LEFT_MARGIN: u32 = 141;
pub const RIGHT_MARGIN: u32 = 141;

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
        let x = (LEFT_MARGIN - 1) as f32 + fraction * (spec_width as f32 + 1.0); // "- 1" so it starts with border

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
    audio_info: AudioInfo,
    font: &FontVec,
    scale: PxScale,
    color: Rgba<u8>,
    split_channels: bool,
) {
    let max_freq_khz = (audio_info.sample_rate / 2) as f32 / 1000.0;
    let draw_multi_channel = audio_info.channels > 1 && split_channels;

    let channel_count = if draw_multi_channel { 2 } else { 1 };
    let height_per_channel = if draw_multi_channel {
        spec_height / 2
    } else {
        spec_height
    };

    for channel in 0..channel_count {
        let y_offset = TOP_MARGIN + (channel * height_per_channel);
        let num_ticks = if draw_multi_channel { 5 } else { 10 };
        for i in 0..=num_ticks {
            let fraction = i as f32 / num_ticks as f32;
            let y = (y_offset - 1) as f32 + (1.0 - fraction) * (height_per_channel + 1) as f32;

            let x_start_left = LEFT_MARGIN as f32 - 6.0;

            if !(draw_multi_channel && channel == 1 && i == num_ticks) {
                // Left ticks
                let x_end_left = LEFT_MARGIN as f32 - 1.0;
                draw_line_segment_mut(image, (x_start_left, y), (x_end_left, y), color);

                // Right ticks
                let x_start_right = LEFT_MARGIN as f32 + spec_width as f32 + 1.0;
                let x_end_right = x_start_right + 5.0;
                draw_line_segment_mut(image, (x_start_right, y), (x_end_right, y), color);
            }

            // Freq labels
            if draw_multi_channel && channel == 1 && i == num_ticks {
                // Skip max freq label for bottom channel to avoid overlap
            } else {
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

fn truncate_text(font: &FontVec, scale: PxScale, text: &str, max_width: u32) -> String {
    let (text_width, _) = imageproc::drawing::text_size(scale, font, text);
    if text_width <= max_width {
        return text.to_string();
    }

    let ellipsis = "...";
    let (ellipsis_width, _) = imageproc::drawing::text_size(scale, font, ellipsis);

    let mut truncated = text.to_string();
    if max_width > ellipsis_width {
        // Try to fit with ellipsis
        let target_width = max_width - ellipsis_width;
        while !truncated.is_empty() {
            let (w, _) = imageproc::drawing::text_size(scale, font, &truncated);
            if w <= target_width {
                truncated.push_str(ellipsis);
                return truncated;
            }
            truncated.pop();
        }
    }
    String::new()
}

fn yuv8bit_to_rgb(y: f32, u: f32, v: f32) -> Rgba<u8> {
    // Formula for full-range YUV [0,255] to RGB [0,255]
    let u = u - 128.0;
    let v = v - 128.0;

    let r = y + 1.402 * v;
    let g = y - 0.344136 * u - 0.714136 * v;
    let b = y + 1.772 * u;

    Rgba([
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
        255,
    ])
}

pub fn draw_gradient_line_mut(
    image: &mut RgbaImage,
    start: (f32, f32),
    end: (f32, f32),
    palette: &[(f32, f32, f32, f32)], // (stop, y, u, v)
    saturation: f32,
    thickness: u32,
) {
    let (x0, y0) = start;
    let (x1, y1) = end;

    let dx = x1 - x0;
    let dy = y1 - y0;

    let steps = dx.abs().max(dy.abs());

    if steps < 1.0 {
        if (x0 as u32) < image.width() && (y0 as u32) < image.height() {
            let (_, y_interp, u_interp, v_interp) = palette[0];
            let y_8bit = y_interp * 255.0;
            let u_8bit = 128.0 + u_interp * 255.0 * saturation;
            let v_8bit = 128.0 + v_interp * 255.0 * saturation;
            let color = yuv8bit_to_rgb(
                y_8bit.clamp(0.0, 255.0),
                u_8bit.clamp(0.0, 255.0),
                v_8bit.clamp(0.0, 255.0),
            );
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
        let a = 1.0 - t; // Reverse the gradient direction (0.0 at bottom, 1.0 at top)
        let x_pos = (x0 + t * dx).round() as u32;
        let y_pos = (y0 + t * dy).round() as u32;

        // Find the segment in the palette that `a` falls into
        let mut end_idx = 1;
        while end_idx < palette.len() - 1 && palette[end_idx].0 < a {
            end_idx += 1;
        }
        let start_idx = end_idx - 1;

        let start_stop = palette[start_idx];
        let end_stop = palette[end_idx];

        let (start_a, start_y, start_u, start_v) = start_stop;
        let (end_a, end_y, end_u, end_v) = end_stop;

        // Calculate interpolation factor within the segment
        let lerp_frac = if (end_a - start_a).abs() < f32::EPSILON {
            0.0
        } else {
            (a - start_a) / (end_a - start_a)
        };

        // Interpolate Y, U, V
        let y_interp = start_y * (1.0 - lerp_frac) + end_y * lerp_frac;
        let u_interp = start_u * (1.0 - lerp_frac) + end_u * lerp_frac;
        let v_interp = start_v * (1.0 - lerp_frac) + end_v * lerp_frac;

        // Construct 8-bit YUV pixel, applying saturation, to match ffmpeg's internal pipeline
        let y_8bit = y_interp * 255.0;
        let u_8bit = 128.0 + u_interp * 255.0 * saturation;
        let v_8bit = 128.0 + v_interp * 255.0 * saturation;

        // Clip YUV components before conversion, which is crucial for high saturation
        let color = yuv8bit_to_rgb(
            y_8bit.clamp(0.0, 255.0),
            u_8bit.clamp(0.0, 255.0),
            v_8bit.clamp(0.0, 255.0),
        );

        // Draw a horizontal line for thickness
        for k in 0..thickness {
            let current_x = x_pos + k;
            if current_x < image.width() && y_pos < image.height() {
                image.put_pixel(current_x, y_pos, color);
            }
        }
    }
}

use crate::settings::SpectrogramColorScheme;

/// Creates an image with a legend template.
/// The spectrogram itself will be drawn on top of this template later.
pub fn draw_legend(
    spec_width: u32,
    spec_height: u32,
    filename: &str,
    ffmpeg_settings: &str,
    audio_info: Option<AudioInfo>,
    saturation: f32,
    color_scheme: SpectrogramColorScheme,
    split_channels: bool,
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
    let truncated_filename = truncate_text(&font, font_normal, filename, spec_width);
    draw_text_mut(
        &mut image,
        text_color,
        LEFT_MARGIN as i32,
        10,
        font_normal,
        &font,
        &truncated_filename,
    );

    // Draw ffmpeg settings
    let mut display_string = String::from(ffmpeg_settings);
    if let Some(info) = &audio_info {
        let mut details = Vec::new();
        details.push(info.format.to_uppercase());
        details.push(format!("{} Hz", info.sample_rate));
        if info.bits_per_sample > 0 {
            details.push(format!("{} bit", info.bits_per_sample));
        }
        let audio_details = details.join(", ");
        if !ffmpeg_settings.is_empty() {
            display_string = format!("{}, {}", audio_details, ffmpeg_settings);
        } else {
            display_string = audio_details;
        }
    }
    let truncated_display_string = truncate_text(&font, font_normal, &display_string, spec_width);
    draw_text_mut(
        &mut image,
        text_color,
        LEFT_MARGIN as i32,
        28,
        font_normal,
        &font,
        &truncated_display_string,
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
    let palette = palettes::get_palette(color_scheme);
    let line_x = (LEFT_MARGIN + spec_width + 34) as f32;
    let start_point = (line_x, TOP_MARGIN as f32);
    let end_point = (line_x, (TOP_MARGIN + spec_height) as f32);
    draw_gradient_line_mut(&mut image, start_point, end_point, palette, saturation, 10);

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
            info,
            &font,
            font_scales,
            text_color,
            split_channels,
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
