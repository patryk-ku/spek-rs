use eframe::egui::ColorImage;
use image::GenericImageView;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum SpectrogramColorScheme {
    Intensity,
    Channel,
    Rainbow,
    Moreland,
    Nebulae,
    Fire,
    Fiery,
    Fruit,
    Cool,
    Magma,
    Green,
    Viridis,
    Plasma,
    Cividis,
    Terrain,
}

impl SpectrogramColorScheme {
    fn as_str(&self) -> &'static str {
        match self {
            SpectrogramColorScheme::Intensity => "intensity",
            SpectrogramColorScheme::Channel => "channel",
            SpectrogramColorScheme::Rainbow => "rainbow",
            SpectrogramColorScheme::Moreland => "moreland",
            SpectrogramColorScheme::Nebulae => "nebulae",
            SpectrogramColorScheme::Fire => "fire",
            SpectrogramColorScheme::Fiery => "fiery",
            SpectrogramColorScheme::Fruit => "fruit",
            SpectrogramColorScheme::Cool => "cool",
            SpectrogramColorScheme::Magma => "magma",
            SpectrogramColorScheme::Green => "green",
            SpectrogramColorScheme::Viridis => "viridis",
            SpectrogramColorScheme::Plasma => "plasma",
            SpectrogramColorScheme::Cividis => "cividis",
            SpectrogramColorScheme::Terrain => "terrain",
        }
    }

    pub const VALUES: [Self; 15] = [
        Self::Intensity,
        Self::Channel,
        Self::Rainbow,
        Self::Moreland,
        Self::Nebulae,
        Self::Fire,
        Self::Fiery,
        Self::Fruit,
        Self::Cool,
        Self::Magma,
        Self::Green,
        Self::Viridis,
        Self::Plasma,
        Self::Cividis,
        Self::Terrain,
    ];
}

impl std::fmt::Display for SpectrogramColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum SpectogramWinFunc {
    Rect,
    Bartlett,
    Hann,
    Hanning,
    Hamming,
    Blackman,
    Welch,
    Flattop,
    Bharris,
    Bnuttall,
    Bhann,
    Sine,
    Nuttall,
    Lanczos,
    Gauss,
    Tukey,
    Dolph,
    Cauchy,
    Parzen,
    Poisson,
    Bohman,
    Kaiser,
}

impl SpectogramWinFunc {
    fn as_str(&self) -> &'static str {
        match self {
            SpectogramWinFunc::Rect => "rect",
            SpectogramWinFunc::Bartlett => "bartlett",
            SpectogramWinFunc::Hann => "hann",
            SpectogramWinFunc::Hanning => "hanning",
            SpectogramWinFunc::Hamming => "hamming",
            SpectogramWinFunc::Blackman => "blackman",
            SpectogramWinFunc::Welch => "welch",
            SpectogramWinFunc::Flattop => "flattop",
            SpectogramWinFunc::Bharris => "bharris",
            SpectogramWinFunc::Bnuttall => "bnuttall",
            SpectogramWinFunc::Bhann => "bhann",
            SpectogramWinFunc::Sine => "sine",
            SpectogramWinFunc::Nuttall => "nuttall",
            SpectogramWinFunc::Lanczos => "lanczos",
            SpectogramWinFunc::Gauss => "gauss",
            SpectogramWinFunc::Tukey => "tukey",
            SpectogramWinFunc::Dolph => "dolph",
            SpectogramWinFunc::Cauchy => "cauchy",
            SpectogramWinFunc::Parzen => "parzen",
            SpectogramWinFunc::Poisson => "poisson",
            SpectogramWinFunc::Bohman => "bohman",
            SpectogramWinFunc::Kaiser => "kaiser",
        }
    }

    pub const VALUES: [Self; 22] = [
        Self::Rect,
        Self::Bartlett,
        Self::Hann,
        Self::Hanning,
        Self::Hamming,
        Self::Blackman,
        Self::Welch,
        Self::Flattop,
        Self::Bharris,
        Self::Bnuttall,
        Self::Bhann,
        Self::Sine,
        Self::Nuttall,
        Self::Lanczos,
        Self::Gauss,
        Self::Tukey,
        Self::Dolph,
        Self::Cauchy,
        Self::Parzen,
        Self::Poisson,
        Self::Bohman,
        Self::Kaiser,
    ];
}

impl std::fmt::Display for SpectogramWinFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum SpectrogramScale {
    Lin,
    Sqrt,
    Cbrt,
    Log,
    FourthRt,
    FifthRt,
}

impl SpectrogramScale {
    fn as_str(&self) -> &'static str {
        match self {
            SpectrogramScale::Lin => "lin",
            SpectrogramScale::Sqrt => "sqrt",
            SpectrogramScale::Cbrt => "cbrt",
            SpectrogramScale::Log => "log",
            SpectrogramScale::FourthRt => "4thrt",
            SpectrogramScale::FifthRt => "5thrt",
        }
    }

    pub const VALUES: [Self; 6] = [
        Self::Lin,
        Self::Sqrt,
        Self::Cbrt,
        Self::Log,
        Self::FourthRt,
        Self::FifthRt,
    ];
}

impl std::fmt::Display for SpectrogramScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Generates a spectrogram by calling ffmpeg and captures the output image from stdout.
pub fn generate_spectrogram_in_memory(
    input_path: &str,
    legend: bool,
    color_scheme: SpectrogramColorScheme,
    win_func: SpectogramWinFunc,
    scale: SpectrogramScale,
    gain: f32,
    saturation: f32,
    split_channels: bool,
    width: u32,
    height: u32,
    horizontal: bool,
) -> Option<ColorImage> {
    println!(
        "Generating spectrogram for: \"{}\" settings = legend: {}, color: {}, win_func: {}, scale: {}, gain: {}, saturation: {}, split: {}, size: {}x{}, horizontal: {}",
        input_path,
        legend,
        color_scheme.as_str(),
        win_func,
        scale,
        gain,
        saturation,
        split_channels,
        width,
        height,
        horizontal
    );

    let mode = if split_channels {
        "separate"
    } else {
        "combined"
    };

    // let safe_path_string = shell_single_quote(input_path);
    // let text = format!(
    //     "drawtext=text='{}':x=10:y=10:fontsize=12:fontcolor=white",
    //     safe_path_string
    // );

    let orientation = if horizontal { "horizontal" } else { "vertical" };

    let lavfi_filter = format!(
        "showspectrumpic=s={}x{}:legend={}:color={}:win_func={}:scale={}:gain={}:saturation={}:mode={}:orientation={}",
        width,
        height,
        legend,
        color_scheme.as_str(),
        win_func.as_str(),
        scale.as_str(),
        gain,
        saturation,
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
            // "-vcodec",
            // "png",
            // "pipe:1",
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

    println!("Spectrogram generated successfully.");
    Some(color_image)
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

// fn shell_single_quote(s: &str) -> String {
//     format!("'{}'", s.replace('\'', "'\\''"))
// }
