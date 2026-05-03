use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorTheme {
 #[default]
 Default,
 Nord,
 Solarized,
 Dracula,
 Gruvbox,
 Monokai,
 OneDark,
 Catppuccin,
 TokyoNight,
 Ayu,
 GitHubDark,
 MaterialDark,
 OceanicNext,
 Palenight,
 DeepOcean,
 VSCodeDark,
 Horizon,
 NightOwl,
 Synthwave,
 Cyberpunk,
 Argonaut,
 Bordeaux,
}

impl FromStr for ColorTheme {
 type Err = String;

 fn from_str(s: &str) -> Result<Self, Self::Err> {
  ColorTheme::from_str_impl(s).ok_or_else(|| format!("Unknown color theme: {}", s))
 }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeColorsJson {
 pub window_bg: Option<String>,
 pub window_fg: Option<String>,
 pub cursor: Option<String>,
 pub cursor_inactive: Option<String>,
 pub line_number: Option<String>,
 pub text: Option<String>,
 pub border: Option<String>,
 pub border_inactive: Option<String>,
 pub menu: Option<String>,
}

impl ThemeColorsJson {
 fn to_theme_colors(&self) -> ThemeColors {
  ThemeColors {
   window_bg: self.window_bg.as_ref().and_then(|s| parse_hex_color(s)),
   window_fg: self.window_fg.as_ref().and_then(|s| parse_hex_color(s)),
   cursor: self.cursor.as_ref().and_then(|s| parse_hex_color(s)),
   cursor_inactive: self
    .cursor_inactive
    .as_ref()
    .and_then(|s| parse_hex_color(s)),
   line_number: self.line_number.as_ref().and_then(|s| parse_hex_color(s)),
   text: self.text.as_ref().and_then(|s| parse_hex_color(s)),
   border: self.border.as_ref().and_then(|s| parse_hex_color(s)),
   border_inactive: self
    .border_inactive
    .as_ref()
    .and_then(|s| parse_hex_color(s)),
   menu: self.menu.as_ref().and_then(|s| parse_hex_color(s)),
  }
 }
}

fn parse_hex_color(s: &str) -> Option<Color> {
 let s = s.trim_start_matches('#');
 if s.len() == 6 {
  let r = u8::from_str_radix(&s[0..2], 16).ok()?;
  let g = u8::from_str_radix(&s[2..4], 16).ok()?;
  let b = u8::from_str_radix(&s[4..6], 16).ok()?;
  Some(Color::Rgb(r, g, b))
 } else {
  None
 }
}

fn color_to_hex(c: &Option<Color>) -> Option<String> {
 match c {
  Some(Color::Rgb(r, g, b)) => Some(format!("#{:02X}{:02X}{:02X}", r, g, b)),
  _ => None,
 }
}

fn dim_color(color: Option<Color>) -> Option<Color> {
 color.map(|c| match c {
  Color::Rgb(r, g, b) => {
   let dim_factor = 0.6;
   Color::Rgb(
    (r as f32 * dim_factor) as u8,
    (g as f32 * dim_factor) as u8,
    (b as f32 * dim_factor) as u8,
   )
  }
  _ => c,
 })
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ThemeColors {
 pub window_bg: Option<Color>,
 pub window_fg: Option<Color>,
 pub cursor: Option<Color>,
 pub cursor_inactive: Option<Color>,
 pub line_number: Option<Color>,
 pub text: Option<Color>,
 pub border: Option<Color>,
 pub border_inactive: Option<Color>,
 pub menu: Option<Color>,
}

impl ColorTheme {
 pub fn get_colors(&self) -> ThemeColors {
  self.get_colors_with_override(None)
 }

 pub fn get_colors_with_override(&self, custom: Option<&ThemeColors>) -> ThemeColors {
  if let Some(colors) = custom {
   let mut colors = colors.clone();
   if colors.cursor_inactive.is_none() {
    colors.cursor_inactive = dim_color(colors.cursor);
   }
   return colors;
  }
  let mut colors = match self {
   ColorTheme::Default => ThemeColors::default(),
   ColorTheme::Nord => ThemeColors {
    window_bg: Some(Color::Rgb(0x29, 0x2E, 0x3A)),
    window_fg: Some(Color::Rgb(0xD8, 0xDE, 0xE9)),
    cursor: Some(Color::Rgb(0xBF, 0x61, 0x6A)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x4C, 0x56, 0x6A)),
    text: Some(Color::Rgb(0xD8, 0xDE, 0xE9)),
    border: Some(Color::Rgb(0x81, 0xA1, 0xC1)),
    border_inactive: Some(Color::Rgb(0x4C, 0x56, 0x6A)),
    menu: Some(Color::Rgb(0x43, 0x4C, 0x5E)),
   },
   ColorTheme::Solarized => ThemeColors {
    window_bg: Some(Color::Rgb(0x00, 0x24, 0x2F)),
    window_fg: Some(Color::Rgb(0x83, 0x94, 0x96)),
    cursor: Some(Color::Rgb(0xB5, 0x89, 0x00)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x58, 0x6E, 0x75)),
    text: Some(Color::Rgb(0x83, 0x94, 0x96)),
    border: Some(Color::Rgb(0x26, 0x8B, 0xD2)),
    border_inactive: Some(Color::Rgb(0x58, 0x6E, 0x75)),
    menu: Some(Color::Rgb(0x0F, 0x42, 0x51)),
   },
   ColorTheme::Dracula => ThemeColors {
    window_bg: Some(Color::Rgb(0x22, 0x24, 0x30)),
    window_fg: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
    cursor: Some(Color::Rgb(0xFF, 0x79, 0xC6)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x62, 0x72, 0xA4)),
    text: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
    border: Some(Color::Rgb(0xBD, 0x93, 0xF9)),
    border_inactive: Some(Color::Rgb(0x62, 0x72, 0xA4)),
    menu: Some(Color::Rgb(0x4D, 0x51, 0x66)),
   },
   ColorTheme::Gruvbox => ThemeColors {
    window_bg: Some(Color::Rgb(0x1D, 0x20, 0x21)),
    window_fg: Some(Color::Rgb(0xEB, 0xDB, 0xB2)),
    cursor: Some(Color::Rgb(0xFB, 0x49, 0x34)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x66, 0x55, 0x4B)),
    text: Some(Color::Rgb(0xEB, 0xDB, 0xB2)),
    border: Some(Color::Rgb(0xFE, 0x86, 0x29)),
    border_inactive: Some(Color::Rgb(0x66, 0x55, 0x4B)),
    menu: Some(Color::Rgb(0x3C, 0x38, 0x36)),
   },
   ColorTheme::Monokai => ThemeColors {
    window_bg: Some(Color::Rgb(0x27, 0x28, 0x22)),
    window_fg: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
    cursor: Some(Color::Rgb(0xF9, 0x26, 0x72)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x58, 0x5E, 0x5E)),
    text: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
    border: Some(Color::Rgb(0xA6, 0xE2, 0x2E)),
    border_inactive: Some(Color::Rgb(0x58, 0x5E, 0x5E)),
    menu: Some(Color::Rgb(0x3B, 0x3C, 0x34)),
   },
   ColorTheme::OneDark => ThemeColors {
    window_bg: Some(Color::Rgb(0x28, 0x2C, 0x34)),
    window_fg: Some(Color::Rgb(0xAB, 0xB2, 0xBF)),
    cursor: Some(Color::Rgb(0xE5, 0xC0, 0x7B)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x4B, 0x52, 0x63)),
    text: Some(Color::Rgb(0xAB, 0xB2, 0xBF)),
    border: Some(Color::Rgb(0x61, 0xAF, 0xEF)),
    border_inactive: Some(Color::Rgb(0x4B, 0x52, 0x63)),
    menu: Some(Color::Rgb(0x3E, 0x44, 0x52)),
   },
   ColorTheme::Catppuccin => ThemeColors {
    window_bg: Some(Color::Rgb(0x1E, 0x1E, 0x2E)),
    window_fg: Some(Color::Rgb(0xCD, 0xD6, 0xF4)),
    cursor: Some(Color::Rgb(0xF5, 0xC2, 0xE7)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x45, 0x45, 0x5A)),
    text: Some(Color::Rgb(0xCD, 0xD6, 0xF4)),
    border: Some(Color::Rgb(0x89, 0xB4, 0xFA)),
    border_inactive: Some(Color::Rgb(0x45, 0x45, 0x5A)),
    menu: Some(Color::Rgb(0x31, 0x31, 0x3F)),
   },
   ColorTheme::TokyoNight => ThemeColors {
    window_bg: Some(Color::Rgb(0x1A, 0x1B, 0x26)),
    window_fg: Some(Color::Rgb(0xA9, 0xB1, 0xD6)),
    cursor: Some(Color::Rgb(0xBB, 0x9A, 0xF7)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x36, 0x43, 0x56)),
    text: Some(Color::Rgb(0xA9, 0xB1, 0xD6)),
    border: Some(Color::Rgb(0x7A, 0xA2, 0xE3)),
    border_inactive: Some(Color::Rgb(0x36, 0x43, 0x56)),
    menu: Some(Color::Rgb(0x44, 0x45, 0x53)),
   },
   ColorTheme::Ayu => ThemeColors {
    window_bg: Some(Color::Rgb(0x0A, 0x0E, 0x14)),
    window_fg: Some(Color::Rgb(0xB3, 0xB1, 0xAD)),
    cursor: Some(Color::Rgb(0xFF, 0x99, 0x00)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x4A, 0x4D, 0x52)),
    text: Some(Color::Rgb(0xB3, 0xB1, 0xAD)),
    border: Some(Color::Rgb(0x39, 0xBA, 0xE6)),
    border_inactive: Some(Color::Rgb(0x4A, 0x4D, 0x52)),
    menu: Some(Color::Rgb(0x34, 0x38, 0x3F)),
   },
   ColorTheme::GitHubDark => ThemeColors {
    window_bg: Some(Color::Rgb(0x0D, 0x11, 0x17)),
    window_fg: Some(Color::Rgb(0xC9, 0xD1, 0xD9)),
    cursor: Some(Color::Rgb(0x58, 0xA6, 0xFF)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x48, 0x4F, 0x5A)),
    text: Some(Color::Rgb(0xC9, 0xD1, 0xD9)),
    border: Some(Color::Rgb(0x1F, 0x6F, 0xEB)),
    border_inactive: Some(Color::Rgb(0x48, 0x4F, 0x5A)),
    menu: Some(Color::Rgb(0x36, 0x3C, 0x44)),
   },
   ColorTheme::MaterialDark => ThemeColors {
    window_bg: Some(Color::Rgb(0x26, 0x32, 0x38)),
    window_fg: Some(Color::Rgb(0xEE, 0xFF, 0xFF)),
    cursor: Some(Color::Rgb(0x80, 0xCB, 0xFC)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x54, 0x67, 0x75)),
    text: Some(Color::Rgb(0xEE, 0xFF, 0xFF)),
    border: Some(Color::Rgb(0x82, 0xAA, 0xFF)),
    border_inactive: Some(Color::Rgb(0x54, 0x67, 0x75)),
    menu: Some(Color::Rgb(0x37, 0x44, 0x50)),
   },
   ColorTheme::OceanicNext => ThemeColors {
    window_bg: Some(Color::Rgb(0x1B, 0x2B, 0x34)),
    window_fg: Some(Color::Rgb(0xC0, 0xC5, 0xCE)),
    cursor: Some(Color::Rgb(0xEC, 0x5F, 0x67)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x3B, 0x50, 0x62)),
    text: Some(Color::Rgb(0xC0, 0xC5, 0xCE)),
    border: Some(Color::Rgb(0x66, 0x99, 0xBB)),
    border_inactive: Some(Color::Rgb(0x3B, 0x50, 0x62)),
    menu: Some(Color::Rgb(0x41, 0x51, 0x5E)),
   },
   ColorTheme::Palenight => ThemeColors {
    window_bg: Some(Color::Rgb(0x29, 0x27, 0x3D)),
    window_fg: Some(Color::Rgb(0xA7, 0xA6, 0xB2)),
    cursor: Some(Color::Rgb(0xC3, 0x3D, 0xEE)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x62, 0x5D, 0x7A)),
    text: Some(Color::Rgb(0xA7, 0xA6, 0xB2)),
    border: Some(Color::Rgb(0x89, 0xDD, 0xFF)),
    border_inactive: Some(Color::Rgb(0x62, 0x5D, 0x7A)),
    menu: Some(Color::Rgb(0x5B, 0x58, 0x6E)),
   },
   ColorTheme::DeepOcean => ThemeColors {
    window_bg: Some(Color::Rgb(0x0A, 0x19, 0x2B)),
    window_fg: Some(Color::Rgb(0xBB, 0xD0, 0xE0)),
    cursor: Some(Color::Rgb(0x4F, 0xB6, 0xDB)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x1C, 0x2D, 0x42)),
    text: Some(Color::Rgb(0xBB, 0xD0, 0xE0)),
    border: Some(Color::Rgb(0x36, 0x89, 0xB0)),
    border_inactive: Some(Color::Rgb(0x1C, 0x2D, 0x42)),
    menu: Some(Color::Rgb(0x30, 0x44, 0x58)),
   },
   ColorTheme::VSCodeDark => ThemeColors {
    window_bg: Some(Color::Rgb(0x1E, 0x1E, 0x1E)),
    window_fg: Some(Color::Rgb(0xD4, 0xD4, 0xD4)),
    cursor: Some(Color::Rgb(0x00, 0x9A, 0xCE)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x85, 0x85, 0x85)),
    text: Some(Color::Rgb(0xD4, 0xD4, 0xD4)),
    border: Some(Color::Rgb(0x00, 0x7F, 0xC8)),
    border_inactive: Some(Color::Rgb(0x85, 0x85, 0x85)),
    menu: Some(Color::Rgb(0x45, 0x45, 0x45)),
   },
   ColorTheme::Horizon => ThemeColors {
    window_bg: Some(Color::Rgb(0x1C, 0x1E, 0x28)),
    window_fg: Some(Color::Rgb(0xCB, 0xD0, 0xDC)),
    cursor: Some(Color::Rgb(0xEF, 0xB4, 0x93)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x4B, 0x4E, 0x5F)),
    text: Some(Color::Rgb(0xCB, 0xD0, 0xDC)),
    border: Some(Color::Rgb(0x2A, 0xB3, 0xBD)),
    border_inactive: Some(Color::Rgb(0x4B, 0x4E, 0x5F)),
    menu: Some(Color::Rgb(0x4B, 0x4D, 0x5A)),
   },
   ColorTheme::NightOwl => ThemeColors {
    window_bg: Some(Color::Rgb(0x0B, 0x14, 0x1F)),
    window_fg: Some(Color::Rgb(0xD2, 0x70, 0xA1)),
    cursor: Some(Color::Rgb(0xFC, 0xA4, 0x7C)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x2B, 0x38, 0x4C)),
    text: Some(Color::Rgb(0xD2, 0x70, 0xA1)),
    border: Some(Color::Rgb(0x82, 0xAA, 0xFF)),
    border_inactive: Some(Color::Rgb(0x2B, 0x38, 0x4C)),
    menu: Some(Color::Rgb(0x28, 0x35, 0x47)),
   },
   ColorTheme::Synthwave => ThemeColors {
    window_bg: Some(Color::Rgb(0x26, 0x17, 0x3A)),
    window_fg: Some(Color::Rgb(0xF7, 0xEE, 0xE8)),
    cursor: Some(Color::Rgb(0xFF, 0x79, 0xC6)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x52, 0x33, 0x66)),
    text: Some(Color::Rgb(0xF7, 0xEE, 0xE8)),
    border: Some(Color::Rgb(0x00, 0xD8, 0xFF)),
    border_inactive: Some(Color::Rgb(0x52, 0x33, 0x66)),
    menu: Some(Color::Rgb(0x4A, 0x38, 0x5C)),
   },
   ColorTheme::Cyberpunk => ThemeColors {
    window_bg: Some(Color::Rgb(0x00, 0x05, 0x12)),
    window_fg: Some(Color::Rgb(0xFF, 0xB8, 0x67)),
    cursor: Some(Color::Rgb(0xFF, 0x00, 0xFF)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x2E, 0x24, 0x3A)),
    text: Some(Color::Rgb(0xFF, 0xB8, 0x67)),
    border: Some(Color::Rgb(0x00, 0xF0, 0xFF)),
    border_inactive: Some(Color::Rgb(0x2E, 0x24, 0x3A)),
    menu: Some(Color::Rgb(0x38, 0x32, 0x48)),
   },
   ColorTheme::Argonaut => ThemeColors {
    window_bg: Some(Color::Rgb(0x26, 0x26, 0x2D)),
    window_fg: Some(Color::Rgb(0xEA, 0xE6, 0xDE)),
    cursor: Some(Color::Rgb(0xFF, 0x00, 0x41)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x46, 0x44, 0x50)),
    text: Some(Color::Rgb(0xEA, 0xE6, 0xDE)),
    border: Some(Color::Rgb(0x2B, 0xB5, 0xC6)),
    border_inactive: Some(Color::Rgb(0x46, 0x44, 0x50)),
    menu: Some(Color::Rgb(0x53, 0x51, 0x5A)),
   },
   ColorTheme::Bordeaux => ThemeColors {
    window_bg: Some(Color::Rgb(0x26, 0x1C, 0x1C)),
    window_fg: Some(Color::Rgb(0xF5, 0x9C, 0xA4)),
    cursor: Some(Color::Rgb(0xFF, 0x72, 0x8E)),
    cursor_inactive: None,
    line_number: Some(Color::Rgb(0x52, 0x34, 0x36)),
    text: Some(Color::Rgb(0xF5, 0x9C, 0xA4)),
    border: Some(Color::Rgb(0xDA, 0x6F, 0x7C)),
    border_inactive: Some(Color::Rgb(0x52, 0x34, 0x36)),
    menu: Some(Color::Rgb(0x56, 0x46, 0x46)),
   },
  };
  if colors.cursor_inactive.is_none() {
   colors.cursor_inactive = dim_color(colors.cursor);
  }
  colors
 }

 pub fn all_themes() -> &'static [(&'static str, ColorTheme)] {
  &[
   ("default", ColorTheme::Default),
   ("nord", ColorTheme::Nord),
   ("solarized", ColorTheme::Solarized),
   ("dracula", ColorTheme::Dracula),
   ("gruvbox", ColorTheme::Gruvbox),
   ("monokai", ColorTheme::Monokai),
   ("onedark", ColorTheme::OneDark),
   ("catppuccin", ColorTheme::Catppuccin),
   ("tokyonight", ColorTheme::TokyoNight),
   ("ayu", ColorTheme::Ayu),
   ("githubdark", ColorTheme::GitHubDark),
   ("materialdark", ColorTheme::MaterialDark),
   ("oceanicnext", ColorTheme::OceanicNext),
   ("palenight", ColorTheme::Palenight),
   ("deepocean", ColorTheme::DeepOcean),
   ("vscode", ColorTheme::VSCodeDark),
   ("horizon", ColorTheme::Horizon),
   ("nightowl", ColorTheme::NightOwl),
   ("synthwave", ColorTheme::Synthwave),
   ("cyberpunk", ColorTheme::Cyberpunk),
   ("argonaut", ColorTheme::Argonaut),
   ("bordeaux", ColorTheme::Bordeaux),
  ]
 }

 pub fn to_json(&self) -> String {
  let colors = self.get_colors();
  let json_colors = ThemeColorsJson {
   window_bg: color_to_hex(&colors.window_bg),
   window_fg: color_to_hex(&colors.window_fg),
   cursor: color_to_hex(&colors.cursor),
   cursor_inactive: color_to_hex(&colors.cursor_inactive),
   line_number: color_to_hex(&colors.line_number),
   text: color_to_hex(&colors.text),
   border: color_to_hex(&colors.border),
   border_inactive: color_to_hex(&colors.border_inactive),
   menu: color_to_hex(&colors.menu),
  };
  serde_json::to_string_pretty(&json_colors).unwrap_or_default()
 }

 pub fn from_json(json_str: &str) -> Result<ThemeColors, String> {
  let json_colors: ThemeColorsJson =
   serde_json::from_str(json_str).map_err(|e| format!("Failed to parse theme JSON: {}", e))?;
  Ok(json_colors.to_theme_colors())
 }

 fn from_str_impl(s: &str) -> Option<Self> {
  match s.to_lowercase().as_str() {
   "default" => Some(ColorTheme::Default),
   "nord" => Some(ColorTheme::Nord),
   "solarized" => Some(ColorTheme::Solarized),
   "dracula" => Some(ColorTheme::Dracula),
   "gruvbox" => Some(ColorTheme::Gruvbox),
   "monokai" => Some(ColorTheme::Monokai),
   "onedark" => Some(ColorTheme::OneDark),
   "catppuccin" => Some(ColorTheme::Catppuccin),
   "tokyonight" => Some(ColorTheme::TokyoNight),
   "ayu" => Some(ColorTheme::Ayu),
   "githubdark" => Some(ColorTheme::GitHubDark),
   "materialdark" => Some(ColorTheme::MaterialDark),
   "oceanicnext" => Some(ColorTheme::OceanicNext),
   "palenight" => Some(ColorTheme::Palenight),
   "deepocean" => Some(ColorTheme::DeepOcean),
   "vscode" => Some(ColorTheme::VSCodeDark),
   "horizon" => Some(ColorTheme::Horizon),
   "nightowl" => Some(ColorTheme::NightOwl),
   "synthwave" => Some(ColorTheme::Synthwave),
   "cyberpunk" => Some(ColorTheme::Cyberpunk),
   "argonaut" => Some(ColorTheme::Argonaut),
   "bordeaux" => Some(ColorTheme::Bordeaux),
   _ => None,
  }
 }
}

impl fmt::Display for ColorTheme {
 fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  match self {
   ColorTheme::Default => write!(f, "default"),
   ColorTheme::Nord => write!(f, "nord"),
   ColorTheme::Solarized => write!(f, "solarized"),
   ColorTheme::Dracula => write!(f, "dracula"),
   ColorTheme::Gruvbox => write!(f, "gruvbox"),
   ColorTheme::Monokai => write!(f, "monokai"),
   ColorTheme::OneDark => write!(f, "onedark"),
   ColorTheme::Catppuccin => write!(f, "catppuccin"),
   ColorTheme::TokyoNight => write!(f, "tokyonight"),
   ColorTheme::Ayu => write!(f, "ayu"),
   ColorTheme::GitHubDark => write!(f, "githubdark"),
   ColorTheme::MaterialDark => write!(f, "materialdark"),
   ColorTheme::OceanicNext => write!(f, "oceanicnext"),
   ColorTheme::Palenight => write!(f, "palenight"),
   ColorTheme::DeepOcean => write!(f, "deepocean"),
   ColorTheme::VSCodeDark => write!(f, "vscode"),
   ColorTheme::Horizon => write!(f, "horizon"),
   ColorTheme::NightOwl => write!(f, "nightowl"),
   ColorTheme::Synthwave => write!(f, "synthwave"),
   ColorTheme::Cyberpunk => write!(f, "cyberpunk"),
   ColorTheme::Argonaut => write!(f, "argonaut"),
   ColorTheme::Bordeaux => write!(f, "bordeaux"),
  }
 }
}
