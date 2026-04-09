use ratatui::style::Color;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColorTheme {
 #[default]
 Default,
 Nord,
 Solarized,
 Dracula,
}

impl FromStr for ColorTheme {
 type Err = String;

 fn from_str(s: &str) -> Result<Self, Self::Err> {
  ColorTheme::from_str_impl(s).ok_or_else(|| format!("Unknown color theme: {}", s))
 }
}

#[derive(Clone, Debug)]
pub struct ThemeColors {
 pub window_bg: Option<Color>,
 pub window_fg: Option<Color>,
 pub cursor: Option<Color>,
 pub line_number: Option<Color>,
 pub text: Option<Color>,
 pub border: Option<Color>,
 pub border_inactive: Option<Color>,
 pub menu: Option<Color>,
}

impl Default for ThemeColors {
 fn default() -> Self {
  Self {
   window_bg: None,
   window_fg: None,
   cursor: None,
   line_number: None,
   text: None,
   border: None,
   border_inactive: None,
   menu: None,
  }
 }
}

impl ColorTheme {
 pub fn get_colors(&self) -> ThemeColors {
  match self {
   ColorTheme::Default => ThemeColors::default(),
   ColorTheme::Nord => ThemeColors {
    window_bg: None,
    window_fg: None,
    cursor: Some(Color::Rgb(0xBF, 0x61, 0x6A)),
    line_number: Some(Color::Rgb(0x4C, 0x56, 0x6A)),
    text: Some(Color::Rgb(0xD8, 0xDE, 0xE9)),
    border: Some(Color::Rgb(0x81, 0xA1, 0xC1)),
    border_inactive: Some(Color::Rgb(0x4C, 0x56, 0x6A)),
    menu: Some(Color::Rgb(0x3B, 0x42, 0x52)),
   },
   ColorTheme::Solarized => ThemeColors {
    window_bg: None,
    window_fg: None,
    cursor: Some(Color::Rgb(0xB5, 0x89, 0x00)),
    line_number: Some(Color::Rgb(0x58, 0x6E, 0x75)),
    text: Some(Color::Rgb(0x83, 0x94, 0x96)),
    border: Some(Color::Rgb(0x26, 0x8B, 0xD2)),
    border_inactive: Some(Color::Rgb(0x58, 0x6E, 0x75)),
    menu: Some(Color::Rgb(0x07, 0x36, 0x42)),
   },
   ColorTheme::Dracula => ThemeColors {
    window_bg: None,
    window_fg: None,
    cursor: Some(Color::Rgb(0xFF, 0x79, 0xC6)),
    line_number: Some(Color::Rgb(0x62, 0x72, 0xA4)),
    text: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
    border: Some(Color::Rgb(0xBD, 0x93, 0xF9)),
    border_inactive: Some(Color::Rgb(0x62, 0x72, 0xA4)),
    menu: Some(Color::Rgb(0x44, 0x47, 0x5A)),
   },
  }
 }

 pub fn all_themes() -> &'static [(&'static str, ColorTheme)] {
  &[
   ("default", ColorTheme::Default),
   ("nord", ColorTheme::Nord),
   ("solarized", ColorTheme::Solarized),
   ("dracula", ColorTheme::Dracula),
  ]
 }

 fn from_str_impl(s: &str) -> Option<Self> {
  match s.to_lowercase().as_str() {
   "default" => Some(ColorTheme::Default),
   "nord" => Some(ColorTheme::Nord),
   "solarized" => Some(ColorTheme::Solarized),
   "dracula" => Some(ColorTheme::Dracula),
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
  }
 }
}
