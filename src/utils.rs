use crate::settings::AppSettings;
use eframe::egui::ColorImage;
use image::{GenericImageView, RgbaImage};
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct AudioInfo {
    pub duration: f64,
    pub sample_rate: u32,
    pub format: String,
    pub bits_per_sample: u32,
    pub channels: u32,
}

/// Converts an `image::RgbaImage` to an `eframe::egui::ColorImage`.
pub fn rgba_image_to_color_image(rgba_image: &RgbaImage) -> ColorImage {
    let size = [rgba_image.width() as usize, rgba_image.height() as usize];
    let pixels = rgba_image.as_raw();
    ColorImage::from_rgba_unmultiplied(size, pixels)
}

/// Retrieves audio information (duration, sample rate, format, and bit depth) using ffprobe.
pub fn get_audio_info(input_path: &str) -> Option<AudioInfo> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=duration,sample_rate,bits_per_sample,bits_per_raw_sample,codec_name,channels:format=format_name",
            "-of",
            "default=noprint_wrappers=1",
            input_path,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        eprintln!("ffprobe error: {}", String::from_utf8_lossy(&output.stderr));
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut duration = None;
    let mut sample_rate = None;
    let mut format_name = None;
    let mut codec_name = None;
    let mut bits_per_sample = None;
    let mut bits_per_raw_sample = None;
    let mut channels = None;

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            match parts[0] {
                "duration" => duration = parts[1].parse::<f64>().ok(),
                "sample_rate" => sample_rate = parts[1].parse::<u32>().ok(),
                "format_name" => format_name = Some(parts[1].to_string()),
                "codec_name" => codec_name = Some(parts[1].to_string()),
                "bits_per_sample" => bits_per_sample = parts[1].parse::<u32>().ok(),
                "bits_per_raw_sample" => bits_per_raw_sample = parts[1].parse::<u32>().ok(),
                "channels" => channels = parts[1].parse::<u32>().ok(),
                _ => {}
            }
        }
    }

    let mut final_bits = bits_per_sample.unwrap_or(0);
    if let Some(raw_bits) = bits_per_raw_sample {
        if raw_bits > 0 {
            final_bits = raw_bits;
        }
    }

    let format = if let Some(f) = format_name {
        if f.contains(',') { codec_name } else { Some(f) }
    } else {
        codec_name
    };

    match (duration, sample_rate, format, channels) {
        (Some(d), Some(s), Some(f), Some(c)) => Some(AudioInfo {
            duration: d,
            sample_rate: s,
            format: f,
            bits_per_sample: final_bits,
            channels: c,
        }),
        _ => None,
    }
}

/// Generates a spectrogram by calling ffmpeg and captures the output image from stdout.
pub fn generate_spectrogram_in_memory(
    input_path: &str,
    settings: &AppSettings,
    width: u32,
    height: u32,
    cancel_token: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Option<ColorImage> {
    let start = Instant::now();
    println!("Generating spectrogram for: {}", input_path,);
    println!("{:#?}", settings);

    let mode = if settings.split_channels {
        "separate"
    } else {
        "combined"
    };

    let orientation = if settings.horizontal && !settings.custom_legend {
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

    let mut cmd = match Command::new("ffmpeg")
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
    {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Failed to spawn ffmpeg: {}", e);
            return None;
        }
    };

    let mut stdout = cmd.stdout.take().unwrap();
    let mut buffer = Vec::new();
    let mut read_buf = [0; 4096]; // 4KB buffer

    loop {
        if cancel_token.load(std::sync::atomic::Ordering::Relaxed) {
            if let Err(e) = cmd.kill() {
                eprintln!("Failed to kill ffmpeg process: {}", e);
            }
            return None;
        }

        match stdout.read(&mut read_buf) {
            Ok(0) => break, // EOF
            Ok(n) => buffer.extend_from_slice(&read_buf[..n]),
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("Failed to read ffmpeg stdout: {}", e);
                if let Err(e) = cmd.kill() {
                    eprintln!("Failed to kill ffmpeg process: {}", e);
                }
                return None;
            }
        }
    }

    let status = match cmd.wait() {
        Ok(status) => status,
        Err(e) => {
            eprintln!("Failed to wait for ffmpeg process: {}", e);
            return None;
        }
    };

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
    cancel_token: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    let start = Instant::now();
    println!("Generating spectrogram for: {}", input_path,);
    println!("{:#?}", settings);

    let duration = match get_audio_info(input_path) {
        Some(info) if info.duration > 0.0 => info.duration,
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
    // let orientation = if settings.horizontal && !settings.custom_legend {
    //     "horizontal"
    // } else {
    //     "vertical"
    // };
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
        "vertical", // orientation
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
        if cancel_token.load(std::sync::atomic::Ordering::Relaxed) {
            if let Err(e) = cmd.kill() {
                eprintln!("Failed to kill ffmpeg process: {}", e);
            }
            break;
        }
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

pub fn cycle_option<T: PartialEq + Clone>(current: T, values: &[T], up: bool) -> T {
    let current_index = values.iter().position(|c| c == &current).unwrap_or(0);
    let new_index = if up {
        if current_index == 0 {
            values.len() - 1
        } else {
            current_index - 1
        }
    } else {
        (current_index + 1) % values.len()
    };
    values[new_index].clone()
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
