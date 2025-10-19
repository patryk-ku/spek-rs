use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
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
    pub fn as_str(&self) -> &'static str {
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

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
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
    pub fn as_str(&self) -> &'static str {
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

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SpectrogramScale {
    Lin,
    Sqrt,
    Cbrt,
    Log,
    FourthRt,
    FifthRt,
}

impl SpectrogramScale {
    pub fn as_str(&self) -> &'static str {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct AppSettings {
    pub color_scheme: SpectrogramColorScheme,
    pub win_func: SpectogramWinFunc,
    pub scale: SpectrogramScale,
    pub gain: f32,
    pub saturation: f32,
    pub split_channels: bool,
    pub custom_resolution: bool,
    pub resolution: [u32; 2],
    pub horizontal: bool,
    pub legend: bool,
    pub live_mode: bool,
    pub remember_settings: bool,
    pub custom_legend: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            color_scheme: SpectrogramColorScheme::Intensity,
            win_func: SpectogramWinFunc::Hann,
            scale: SpectrogramScale::Log,
            gain: 1.0,
            saturation: 1.0,
            split_channels: false,
            custom_resolution: false,
            resolution: [500, 320],
            horizontal: false,
            legend: true,
            live_mode: false,
            remember_settings: false,
            custom_legend: true,
        }
    }
}

impl AppSettings {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("spek-rs");
            fs::create_dir_all(&path).ok();
            path.push("config.toml");
            path
        })
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(settings) = toml::from_str::<AppSettings>(&content) {
                    if settings.remember_settings {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Ok(content) = toml::to_string_pretty(self) {
                if fs::write(path, content).is_err() {
                    eprintln!("Failed to write config file.");
                }
            }
        }
    }
}
