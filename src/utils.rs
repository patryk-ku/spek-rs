use crate::settings::AppSettings;
use eframe::egui::ColorImage;
use image::{GenericImageView, RgbaImage};
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::time::Instant;

/// Converts an `image::RgbaImage` to an `eframe::egui::ColorImage`.
pub fn rgba_image_to_color_image(rgba_image: &RgbaImage) -> ColorImage {
    let size = [rgba_image.width() as usize, rgba_image.height() as usize];
    let pixels = rgba_image.as_raw();
    ColorImage::from_rgba_unmultiplied(size, pixels)
}

fn get_audio_duration(input_path: &str) -> Option<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            input_path,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        eprintln!("ffprobe error: {}", String::from_utf8_lossy(&output.stderr));
        return None;
    }

    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    duration_str.parse::<f64>().ok()
}

/// Generates a spectrogram by calling ffmpeg and captures the output image from stdout.
pub fn generate_spectrogram_in_memory(
    input_path: &str,
    settings: &AppSettings,
    width: u32,
    height: u32,
) -> Option<ColorImage> {
    let start = Instant::now();
    println!("Generating spectrogram for: {}", input_path,);
    println!("{:#?}", settings);

    let mode = if settings.split_channels {
        "separate"
    } else {
        "combined"
    };

    let orientation = if settings.horizontal {
        "horizontal"
    } else {
        "vertical"
    };

    let lavfi_filter = format!(
        "showspectrumpic=s={}x{}:legend={}:color={}:win_func={}:scale={}:gain={}:saturation={}:mode={}:orientation={}",
        width,
        height,
        settings.legend,
        settings.color_scheme.as_str(),
        settings.win_func.as_str(),
        settings.scale.as_str(),
        settings.gain,
        settings.saturation,
        mode,
        orientation
    );

    let mut cmd = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            input_path,
            "-lavfi",
            &lavfi_filter,
            "-f",
            "image2pipe",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    let mut stdout = cmd.stdout.take().unwrap();
    let mut buffer = Vec::new();
    if stdout.read_to_end(&mut buffer).is_err() {
        eprintln!("Failed to read ffmpeg stdout");
        return None;
    }

    let status = cmd.wait().ok()?;

    if !status.success() {
        let mut stderr_output = String::new();
        if let Some(mut stderr) = cmd.stderr.take() {
            if stderr.read_to_string(&mut stderr_output).is_ok() {
                eprintln!("ffmpeg error:\n{}", stderr_output);
            }
        }
        eprintln!("ffmpeg process exited with non-zero status");
        return None;
    }

    let image = match image::load_from_memory(&buffer) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to decode image: {}", e);
            return None;
        }
    };

    let (width, height) = image.dimensions();
    let rgba_image = image.to_rgba8();

    let color_image =
        ColorImage::from_rgba_unmultiplied([width as usize, height as usize], rgba_image.as_raw());

    println!("Spectrogram generated in {:?}.", start.elapsed());
    Some(color_image)
}

pub fn stream_spectrogram_frames(
    sender: Sender<Option<ColorImage>>,
    input_path: &str,
    settings: &AppSettings,
    width: u32,
    height: u32,
) {
    let start = Instant::now();
    println!("Generating spectrogram for: {}", input_path,);
    println!("{:#?}", settings);

    let duration = match get_audio_duration(input_path) {
        Some(d) if d > 0.0 => d,
        _ => {
            eprintln!("Failed to get valid audio duration.");
            return;
        }
    };

    let fps = width as f64 / duration;
    let mode = if settings.split_channels {
        "separate"
    } else {
        "combined"
    };
    let orientation = if settings.horizontal {
        "horizontal"
    } else {
        "vertical"
    };
    let temp_width = 10;

    let lavfi_filter = format!(
        "showspectrum=s={}x{}:legend=0:color={}:win_func={}:scale={}:gain={}:saturation={}:mode={}:orientation={}:slide=scroll",
        temp_width,
        height,
        settings.color_scheme.as_str(),
        settings.win_func.as_str(),
        settings.scale.as_str(),
        settings.gain,
        settings.saturation,
        mode,
        orientation
    );

    let mut cmd = match Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            input_path,
            "-lavfi",
            &lavfi_filter,
            "-r",
            &fps.to_string(),
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Failed to spawn ffmpeg: {}", e);
            return;
        }
    };

    let mut stdout = cmd.stdout.take().unwrap();
    let frame_size = (temp_width * height * 4) as usize;
    let mut frame_buffer = vec![0; frame_size];

    // Discard the first few frames which are often empty
    let frames_to_discard = 1;
    for _ in 0..frames_to_discard {
        if stdout.read_exact(&mut frame_buffer).is_err() {
            // Not enough frames to discard, probably a short file
            break;
        }
    }

    loop {
        match stdout.read_exact(&mut frame_buffer) {
            Ok(_) => {
                let mut slice_pixels = Vec::with_capacity((height * 4) as usize);
                for y in 0..height {
                    let start_index = (y * temp_width * 4 + (temp_width - 1) * 4) as usize;
                    slice_pixels.extend_from_slice(&frame_buffer[start_index..start_index + 4]);
                }

                let slice_image =
                    ColorImage::from_rgba_unmultiplied([1, height as usize], &slice_pixels);
                if sender.send(Some(slice_image)).is_err() {
                    if let Err(e) = cmd.kill() {
                        eprintln!("Failed to kill ffmpeg: {}", e);
                    }
                    break;
                }
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                eprintln!("Failed to read from ffmpeg stdout: {}", e);
                break;
            }
        }
    }

    if let Ok(status) = cmd.wait() {
        if !status.success() {
            let mut stderr_output = String::new();
            if let Some(mut stderr) = cmd.stderr.take() {
                if stderr.read_to_string(&mut stderr_output).is_ok() {
                    eprintln!("ffmpeg error:\n{}", stderr_output);
                }
            }
            eprintln!("ffmpeg process exited with non-zero status");
        }
    }

    println!("Spectrogram generated in {:?}.", start.elapsed());
}

pub fn save_image(image: &Option<ColorImage>, input_path: &String) {
    if let Some(image) = image {
        if let Some(pictures_dir) = dirs::picture_dir() {
            let input_filename = Path::new(input_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("spectrogram");

            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&format!("{}.png", input_filename))
                .set_directory(&pictures_dir)
                .save_file()
            {
                let pixels: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                if let Some(rgba_image) =
                    image::RgbaImage::from_raw(image.width() as u32, image.height() as u32, pixels)
                {
                    if let Err(e) = rgba_image.save(&path) {
                        eprintln!("Failed to save image: {}", e);
                    } else {
                        println!("Image saved to {:?}", path);
                    }
                }
            }
        }
    }
}
