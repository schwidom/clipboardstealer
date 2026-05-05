use crossbeam_skiplist::SkipMap;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

struct CustomCounter(AtomicUsize);

impl CustomCounter {
 fn new() -> Self {
  Self(AtomicUsize::new(0))
 }
}

use lazy_static::lazy_static;

lazy_static! {
 static ref CUSTOM_COUNTER_OBJECT: CustomCounter = CustomCounter::new();
}

fn custom_counter() -> String {
 let mut value = CUSTOM_COUNTER_OBJECT.0.load(Relaxed);
 let ret = format!("custom_{}", value);
 value += 1;
 CUSTOM_COUNTER_OBJECT.0.store(value, Relaxed);
 ret
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ThemeColorsJson {
 #[serde(default = "custom_counter")]
 pub(crate) name: String,
 pub(crate) window_bg: Option<String>,
 pub(crate) window_fg: Option<String>,
 pub(crate) cursor: Option<String>,
 pub(crate) cursor_inactive: Option<String>,
 pub(crate) line_number: Option<String>,
 pub(crate) text: Option<String>,
 pub(crate) border: Option<String>,
 pub(crate) border_inactive: Option<String>,
 pub(crate) menu: Option<String>,
}

impl ThemeColorsJson {
 fn to_theme_colors(&self) -> ThemeColors {
  ThemeColors {
   name: self.name.clone(),
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ThemeColors {
 pub(crate) name: String,
 pub(crate) window_bg: Option<Color>,
 pub(crate) window_fg: Option<Color>,
 pub(crate) cursor: Option<Color>,
 pub(crate) cursor_inactive: Option<Color>,
 pub(crate) line_number: Option<Color>,
 pub(crate) text: Option<Color>,
 pub(crate) border: Option<Color>,
 pub(crate) border_inactive: Option<Color>,
 pub(crate) menu: Option<Color>,
}

impl Default for ThemeColors {
 fn default() -> Self {
  Self {
   name: "default".into(),
   window_bg: Default::default(),
   window_fg: Default::default(),
   cursor: Default::default(),
   cursor_inactive: Default::default(),
   line_number: Default::default(),
   text: Default::default(),
   border: Default::default(),
   border_inactive: Default::default(),
   menu: Default::default(),
  }
 }
}

impl ThemeColors {
 pub(crate) fn to_json(&self) -> String {
  let colors = self;
  let json_colors = ThemeColorsJson {
   name: self.name.clone(),
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

 // pub(crate) fn from_json(json_str: &str) -> Result<ThemeColors, String> {
 //  ColorTheme::from_json(json_str)
 // }

 pub(crate) fn from_json(json_str: &str) -> Result<ThemeColors, String> {
  let json_colors: ThemeColorsJson =
   serde_json::from_str(json_str).map_err(|e| format!("Failed to parse theme JSON: {}", e))?;
  Ok(json_colors.to_theme_colors())
 }
}

fn create_theme_colors() -> Vec<ThemeColors> {
 vec![
  ThemeColors::default(),
  ThemeColors {
   name: "nord".into(),
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
  ThemeColors {
   name: "solarized".into(),
   window_bg: Some(Color::Rgb(0x00, 0x24, 0x2F)),
   window_fg: Some(Color::Rgb(0x83, 0x94, 0x96)),
   cursor: Some(Color::Rgb(0xB5, 0x89, 0x00)),
   cursor_inactive: None,
   line_number: Some(Color::Rgb(0x58, 0x6E, 0x75)),
   text: Some(Color::Rgb(0x83, 0x94, 0x96)),
   border: Some(Color::Rgb(0x26, 0x8B, 0xD2)),
   border_inactive: Some(Color::Rgb(0x58, 0x6E, 0x75)),
   // menu: Some(Color::Rgb(0x0F, 0x42, 0x51)),
   menu: Some(Color::Rgb(0x12, 0x4f, 0x61)),
  },
  ThemeColors {
   name: "dracula".into(),
   window_bg: Some(Color::Rgb(0x22, 0x24, 0x30)),
   window_fg: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
   cursor: Some(Color::Rgb(0xFF, 0x79, 0xC6)),
   cursor_inactive: None,
   line_number: Some(Color::Rgb(0x62, 0x72, 0xA4)),
   text: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
   border: Some(Color::Rgb(0xBD, 0x93, 0xF9)),
   border_inactive: Some(Color::Rgb(0x62, 0x72, 0xA4)),
   // menu: Some(Color::Rgb(0x4D, 0x51, 0x66)),
   menu: Some(Color::Rgb(0x5c, 0x61, 0x7a)),
  },
  ThemeColors {
   name: "gruvbox".into(),
   window_bg: Some(Color::Rgb(0x1D, 0x20, 0x21)),
   window_fg: Some(Color::Rgb(0xEB, 0xDB, 0xB2)),
   cursor: Some(Color::Rgb(0xFB, 0x49, 0x34)),
   cursor_inactive: None,
   line_number: Some(Color::Rgb(0x66, 0x55, 0x4B)),
   text: Some(Color::Rgb(0xEB, 0xDB, 0xB2)),
   border: Some(Color::Rgb(0xFE, 0x86, 0x29)),
   border_inactive: Some(Color::Rgb(0x66, 0x55, 0x4B)),
   // menu: Some(Color::Rgb(0x3C, 0x38, 0x36)),
   // menu: Some(Color::Rgb(0x54, 0x4e, 0x4b)),
   menu: Some(Color::Rgb(0x5a, 0x54, 0x51)),
  },
  ThemeColors {
   name: "monokai".into(),
   window_bg: Some(Color::Rgb(0x27, 0x28, 0x22)),
   window_fg: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
   cursor: Some(Color::Rgb(0xF9, 0x26, 0x72)),
   cursor_inactive: None,
   line_number: Some(Color::Rgb(0x58, 0x5E, 0x5E)),
   text: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
   border: Some(Color::Rgb(0xA6, 0xE2, 0x2E)),
   border_inactive: Some(Color::Rgb(0x58, 0x5E, 0x5E)),
   // menu: Some(Color::Rgb(0x3B, 0x3C, 0x34)),
   menu: Some(Color::Rgb(0x58, 0x5a, 0x4e)),
  },
  ThemeColors {
   name: "onedark".into(),
   window_bg: Some(Color::Rgb(0x28, 0x2C, 0x34)),
   window_fg: Some(Color::Rgb(0xAB, 0xB2, 0xBF)),
   cursor: Some(Color::Rgb(0xE5, 0xC0, 0x7B)),
   cursor_inactive: None,
   line_number: Some(Color::Rgb(0x4B, 0x52, 0x63)),
   text: Some(Color::Rgb(0xAB, 0xB2, 0xBF)),
   border: Some(Color::Rgb(0x61, 0xAF, 0xEF)),
   border_inactive: Some(Color::Rgb(0x4B, 0x52, 0x63)),
   // menu: Some(Color::Rgb(0x3E, 0x44, 0x52)),
   menu: Some(Color::Rgb(0x5d, 0x66, 0x7b)),
  },
  ThemeColors {
   name: "catppuccin".into(),
   window_bg: Some(Color::Rgb(0x1E, 0x1E, 0x2E)),
   window_fg: Some(Color::Rgb(0xCD, 0xD6, 0xF4)),
   cursor: Some(Color::Rgb(0xF5, 0xC2, 0xE7)),
   cursor_inactive: None,
   line_number: Some(Color::Rgb(0x45, 0x45, 0x5A)),
   text: Some(Color::Rgb(0xCD, 0xD6, 0xF4)),
   border: Some(Color::Rgb(0x89, 0xB4, 0xFA)),
   border_inactive: Some(Color::Rgb(0x45, 0x45, 0x5A)),
   // menu: Some(Color::Rgb(0x31, 0x31, 0x3F)),
   menu: Some(Color::Rgb(0x49, 0x49, 0x5e)),
  },
  ThemeColors {
   name: "tokyonight".into(),
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
  ThemeColors {
   name: "ayu".into(),
   window_bg: Some(Color::Rgb(0x0A, 0x0E, 0x14)),
   window_fg: Some(Color::Rgb(0xB3, 0xB1, 0xAD)),
   cursor: Some(Color::Rgb(0xFF, 0x99, 0x00)),
   cursor_inactive: None,
   line_number: Some(Color::Rgb(0x4A, 0x4D, 0x52)),
   text: Some(Color::Rgb(0xB3, 0xB1, 0xAD)),
   border: Some(Color::Rgb(0x39, 0xBA, 0xE6)),
   border_inactive: Some(Color::Rgb(0x4A, 0x4D, 0x52)),
   // menu: Some(Color::Rgb(0x34, 0x38, 0x3F)),
   menu: Some(Color::Rgb(0x4e, 0x54, 0x5e)),
  },
  ThemeColors {
   name: "githubdark".into(),
   window_bg: Some(Color::Rgb(0x0D, 0x11, 0x17)),
   window_fg: Some(Color::Rgb(0xC9, 0xD1, 0xD9)),
   cursor: Some(Color::Rgb(0x58, 0xA6, 0xFF)),
   cursor_inactive: None,
   line_number: Some(Color::Rgb(0x48, 0x4F, 0x5A)),
   text: Some(Color::Rgb(0xC9, 0xD1, 0xD9)),
   border: Some(Color::Rgb(0x1F, 0x6F, 0xEB)),
   border_inactive: Some(Color::Rgb(0x48, 0x4F, 0x5A)),
   // menu: Some(Color::Rgb(0x36, 0x3C, 0x44)),
   menu: Some(Color::Rgb(0x51, 0x5a, 0x66)),
  },
  ThemeColors {
   name: "materialdark".into(),
   window_bg: Some(Color::Rgb(0x26, 0x32, 0x38)),
   window_fg: Some(Color::Rgb(0xEE, 0xFF, 0xFF)),
   cursor: Some(Color::Rgb(0x80, 0xCB, 0xFC)),
   cursor_inactive: None,
   line_number: Some(Color::Rgb(0x54, 0x67, 0x75)),
   text: Some(Color::Rgb(0xEE, 0xFF, 0xFF)),
   border: Some(Color::Rgb(0x82, 0xAA, 0xFF)),
   border_inactive: Some(Color::Rgb(0x54, 0x67, 0x75)),
   // menu: Some(Color::Rgb(0x37, 0x44, 0x50)),
   menu: Some(Color::Rgb(0x52, 0x66, 0x78)),
  },
  ThemeColors {
   name: "oceanicnext".into(),
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
  ThemeColors {
   name: "palenight".into(),
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
  ThemeColors {
   name: "deepocean".into(),
   window_bg: Some(Color::Rgb(0x0A, 0x19, 0x2B)),
   window_fg: Some(Color::Rgb(0xBB, 0xD0, 0xE0)),
   cursor: Some(Color::Rgb(0x4F, 0xB6, 0xDB)),
   cursor_inactive: None,
   // line_number: Some(Color::Rgb(0x1C, 0x2D, 0x42)),
   line_number: Some(Color::Rgb(0x2a, 0x43, 0x63)),
   text: Some(Color::Rgb(0xBB, 0xD0, 0xE0)),
   border: Some(Color::Rgb(0x36, 0x89, 0xB0)),
   border_inactive: Some(Color::Rgb(0x1C, 0x2D, 0x42)),
   // menu: Some(Color::Rgb(0x30, 0x44, 0x58)),
   menu: Some(Color::Rgb(0x48, 0x66, 0x84)),
  },
  ThemeColors {
   name: "vscodium".into(),
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
  ThemeColors {
   name: "horizon".into(),
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
  ThemeColors {
   name: "nightowl".into(),
   window_bg: Some(Color::Rgb(0x0B, 0x14, 0x1F)),
   window_fg: Some(Color::Rgb(0xD2, 0x70, 0xA1)),
   cursor: Some(Color::Rgb(0xFC, 0xA4, 0x7C)),
   cursor_inactive: None,
   // line_number: Some(Color::Rgb(0x2B, 0x38, 0x4C)),
   line_number: Some(Color::Rgb(0x40, 0x54, 0x72)),
   text: Some(Color::Rgb(0xD2, 0x70, 0xA1)),
   border: Some(Color::Rgb(0x82, 0xAA, 0xFF)),
   border_inactive: Some(Color::Rgb(0x2B, 0x38, 0x4C)),
   // menu: Some(Color::Rgb(0x28, 0x35, 0x47)),
   menu: Some(Color::Rgb(0x3c, 0x4f, 0x6a)),
  },
  ThemeColors {
   name: "synthwave".into(),
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
  ThemeColors {
   name: "cyberpunk".into(),
   window_bg: Some(Color::Rgb(0x00, 0x05, 0x12)),
   window_fg: Some(Color::Rgb(0xFF, 0xB8, 0x67)),
   cursor: Some(Color::Rgb(0xFF, 0x00, 0xFF)),
   cursor_inactive: None,
   // line_number: Some(Color::Rgb(0x2E, 0x24, 0x3A)),
   line_number: Some(Color::Rgb(0x45, 0x36, 0x57)),
   text: Some(Color::Rgb(0xFF, 0xB8, 0x67)),
   border: Some(Color::Rgb(0x00, 0xF0, 0xFF)),
   border_inactive: Some(Color::Rgb(0x2E, 0x24, 0x3A)),
   // menu: Some(Color::Rgb(0x38, 0x32, 0x48)),
   menu: Some(Color::Rgb(0x54, 0x4b, 0x6c)),
  },
  ThemeColors {
   name: "argonaut".into(),
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
  ThemeColors {
   name: "bordeaux".into(),
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
 ]
}

pub(crate) fn all_themes_skipmap() -> SkipMap<String, ThemeColors> {
 let ret = SkipMap::<String, ThemeColors>::default();
 for i in create_theme_colors().iter() {
  let name = i.name.clone();
  let mut i = i.clone();
  if i.cursor_inactive.is_none() && i.cursor.is_some() {
   i.cursor_inactive = dim_color(i.cursor)
  }

  ret.insert(name, i.clone());
 }
 ret
}

pub(crate) fn default_color_theme_name() -> String {
 "default".into()
}
