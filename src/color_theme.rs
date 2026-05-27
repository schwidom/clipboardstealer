use crossbeam_skiplist::SkipMap;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

use std::concat;
use std::stringify;

// #[macro_export]
// macro_rules! hex {
// // Stringify erzeugt "#" und "292E3A", concat! verbindet sie zu "#292E3A"
//     (# $color:tt) => {
//         // Stringify erzeugt "#" und "292E3A", concat! verbindet sie zu "#292E3A"
//         parse_hex_color_or_panic(concat!(stringify!(#), stringify!($color)))
//     };
// }

#[test]
fn test_parse_hex_color() {
 assert_eq!(parse_hex_color("#123456"), Some(Color::Rgb(0x12, 0x34, 0x56)));
 let x = parse_hex_color_or_panic("#123456");
 assert_eq!(x, Some(Color::Rgb(0x12, 0x34, 0x56)));
 assert_eq!(parse_hex_color_or_panic("#123456"), Some(Color::Rgb(0x12, 0x34, 0x56)));
 assert_eq!("123", stringify!(123));
 assert_eq!("123456", concat!(stringify!(123), stringify!(456)));
 // assert_eq!("", stringify!(#123e)); // error: expected at least one digit in exponent
 assert_eq!(Some(Color::Rgb(0x12, 0x34, 0x5e)), parse_hex_color("#12345e"));
}

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
 pub(crate) window_bg_header_footer: Option<String>,
 pub(crate) cursor: Option<String>,
 pub(crate) cursor_inactive: Option<String>,
 pub(crate) line_number: Option<String>,
 pub(crate) line_number_inactive: Option<String>,
 pub(crate) text: Option<String>,
 pub(crate) border: Option<String>,
 pub(crate) border_inactive: Option<String>,
 pub(crate) menu: Option<String>,
 pub(crate) pause: Option<String>,
 pub(crate) selection_star: Option<String>,
 pub(crate) cb_type: Option<String>,
 pub(crate) cb_type_inactive: Option<String>,
 pub(crate) date_time: Option<String>,
 pub(crate) date_time_inactive: Option<String>,
}

impl ThemeColorsJson {
 fn to_theme_colors(&self) -> ThemeColors {
  ThemeColors {
   name: self.name.clone(),
   window_bg: self.window_bg.as_ref().and_then(|s| parse_hex_color(s)),
   window_fg: self.window_fg.as_ref().and_then(|s| parse_hex_color(s)),
   window_bg_header_footer: self
    .window_bg_header_footer
    .as_ref()
    .and_then(|s| parse_hex_color(s)),
   cursor: self.cursor.as_ref().and_then(|s| parse_hex_color(s)),
   cursor_inactive: self
    .cursor_inactive
    .as_ref()
    .and_then(|s| parse_hex_color(s)),
   line_number: self.line_number.as_ref().and_then(|s| parse_hex_color(s)),
   line_number_inactive: self
    .line_number_inactive
    .as_ref()
    .and_then(|s| parse_hex_color(s)),
   text: self.text.as_ref().and_then(|s| parse_hex_color(s)),
   border: self.border.as_ref().and_then(|s| parse_hex_color(s)),
   border_inactive: self
    .border_inactive
    .as_ref()
    .and_then(|s| parse_hex_color(s)),
   menu: self.menu.as_ref().and_then(|s| parse_hex_color(s)),
   pause: self.pause.as_ref().and_then(|s| parse_hex_color(s)),
   selection_star: self
    .selection_star
    .as_ref()
    .and_then(|s| parse_hex_color(s)),
   cb_type: self.cb_type.as_ref().and_then(|s| parse_hex_color(s)),
   cb_type_inactive: self
    .cb_type_inactive
    .as_ref()
    .and_then(|s| parse_hex_color(s)),
   date_time: self.date_time.as_ref().and_then(|s| parse_hex_color(s)),
   date_time_inactive: self
    .date_time_inactive
    .as_ref()
    .and_then(|s| parse_hex_color(s)),
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

fn parse_hex_color_or_panic(s: &str) -> Option<Color> {
 Some(parse_hex_color(s).expect(&format!("wrong color : {}", s)))
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
   let dim_factor = 0.7;
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
 pub(crate) window_bg_header_footer: Option<Color>,
 pub(crate) cursor: Option<Color>,
 pub(crate) cursor_inactive: Option<Color>,
 pub(crate) line_number: Option<Color>,
 pub(crate) line_number_inactive: Option<Color>,
 pub(crate) text: Option<Color>,
 pub(crate) border: Option<Color>,
 pub(crate) border_inactive: Option<Color>,
 pub(crate) menu: Option<Color>,
 pub(crate) pause: Option<Color>,
 pub(crate) selection_star: Option<Color>,
 pub(crate) cb_type: Option<Color>,
 pub(crate) cb_type_inactive: Option<Color>,
 pub(crate) date_time: Option<Color>,
 pub(crate) date_time_inactive: Option<Color>,
}

impl Default for ThemeColors {
 fn default() -> Self {
  Self {
   name: "default".into(),
   window_bg: Default::default(),
   window_fg: Default::default(),
   window_bg_header_footer: Default::default(),
   cursor: Default::default(),
   cursor_inactive: Default::default(),
   line_number: Default::default(),
   line_number_inactive: Default::default(),
   text: Default::default(),
   border: Default::default(),
   border_inactive: Default::default(),
   menu: Default::default(),
   pause: Default::default(),
   selection_star: Default::default(),
   cb_type: Default::default(),
   cb_type_inactive: Default::default(),
   date_time: Default::default(),
   date_time_inactive: Default::default(),
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
   window_bg_header_footer: color_to_hex(&colors.window_bg_header_footer),
   cursor: color_to_hex(&colors.cursor),
   cursor_inactive: color_to_hex(&colors.cursor_inactive),
   line_number: color_to_hex(&colors.line_number),
   line_number_inactive: color_to_hex(&colors.line_number_inactive),
   text: color_to_hex(&colors.text),
   border: color_to_hex(&colors.border),
   border_inactive: color_to_hex(&colors.border_inactive),
   menu: color_to_hex(&colors.menu),
   pause: color_to_hex(&colors.pause),
   selection_star: color_to_hex(&colors.selection_star),
   cb_type: color_to_hex(&colors.cb_type),
   cb_type_inactive: color_to_hex(&colors.cb_type_inactive),
   date_time: color_to_hex(&colors.date_time),
   date_time_inactive: color_to_hex(&colors.date_time_inactive),
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

const COLOR_RED: Color = Color::Rgb(0xff, 0, 0);
const COLOR_GREEN: Color = Color::Rgb(0, 0xff, 0);
#[allow(unused)]
const COLOR_BLUE: Color = Color::Rgb(0, 0, 0xff);
const COLOR_BRIGHT_BLUE: Color = Color::Rgb(0x7f, 0x7f, 0xff);
const COLOR_CYAN: Color = Color::Rgb(0, 0xff, 0xff);
const COLOR_YELLOW: Color = Color::Rgb(0xff, 0xff, 0);

fn create_theme_colors() -> Vec<ThemeColors> {
 vec![
  ThemeColors::default(),
  ThemeColors {
   name: "navy".into(),
   window_bg: parse_hex_color_or_panic( "#272935"),
   window_fg: parse_hex_color_or_panic( "#D9DEE9"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#BF616A"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#4C566A"),
   text: parse_hex_color_or_panic( "#D8DEE9"),
   border: parse_hex_color_or_panic( "#546e7a"),
   border_inactive: parse_hex_color_or_panic( "#4C566A"),
   menu: parse_hex_color_or_panic( "#434C5E"),
   pause: Some(COLOR_GREEN),
   selection_star: parse_hex_color_or_panic( "#BF616A"),
   cb_type: parse_hex_color_or_panic( "#81A1C1"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#4C566A"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "nordiclights".into(),
   window_bg: parse_hex_color_or_panic( "#145270"),
   window_fg: parse_hex_color_or_panic( "#ebe3ce"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#c97b47"),
   cursor_inactive: parse_hex_color_or_panic( "#9aacb8"),
   line_number: parse_hex_color_or_panic( "#c4d4d7"),
   text: parse_hex_color_or_panic( "#ebe3ce"),
   border_inactive: parse_hex_color_or_panic( "#aaae97"),
   border: parse_hex_color_or_panic( "#d5cfc6"),
   menu: parse_hex_color_or_panic( "#434C5E"),
   pause: Some(COLOR_CYAN),
   selection_star: parse_hex_color_or_panic( "#9aacb8"),
   cb_type: parse_hex_color_or_panic( "#03e3e3"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#07b1cd"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "nord".into(),
   window_bg: parse_hex_color_or_panic( "#292E3A"),
   window_fg: parse_hex_color_or_panic( "#D8DEE9"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#BF616A"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#4C566A"),
   text: parse_hex_color_or_panic( "#D8DEE9"),
   border: parse_hex_color_or_panic( "#81A1C1"),
   border_inactive: parse_hex_color_or_panic( "#4C566A"),
   menu: parse_hex_color_or_panic( "#434C5E"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#BF616A"),
   cb_type: parse_hex_color_or_panic( "#81A1C1"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#4C566A"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "alaska".into(),
   // Basis-Hintergrund (Nord0 - tiefes Dunkelblau)
   window_bg: parse_hex_color_or_panic( "#2E3440"),
   // UI-Vordergrund für Labels/Metadaten (Nord6 - hellstes Arktisweiß)
   window_fg: parse_hex_color_or_panic( "#ECEFF4"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   // Aktiver Cursor (Nord11 - Aurora Rot für maximale Sichtbarkeit)
   cursor: parse_hex_color_or_panic( "#cb6c76"),
   // Inaktiver Cursor (Nord4 - dezentes Hellgrau, damit er nicht flimmert)
   cursor_inactive: parse_hex_color_or_panic( "#8d9ab1"),
   // Zeilennummern (Nord3 - gedämpftes Graublau, tritt in den Hintergrund)
   line_number: parse_hex_color_or_panic( "#4C566A"),
   // Reiner Fließtext (Nord5 - Standardweiß, minimal dunkler als window_fg für Hierarchie)
   text: parse_hex_color_or_panic( "#E5E9F0"),
   // Aktiver Fokus-Rahmen (Nord9 - helles Frost-Blau)
   border: parse_hex_color_or_panic( "#81A1C1"),
   // Inaktiver Rahmen (Nord1 - dunkleres UI-Grau, deutlich unter line_number anzusiedeln)
   border_inactive: parse_hex_color_or_panic( "#3B4252"),
   // Menü-Hintergrund (Nord2 - setzt sich leicht vom window_bg ab)
   menu: parse_hex_color_or_panic( "#434C5E"),
   // Pause-Zustand (Nord12 - Aurora Orange, hebt sich vom Cursor-Rot ab)
   pause: parse_hex_color_or_panic( "#D08770"),
   // Auswahl/Sterne (Nord13 - Aurora Gelb, setzt einen klaren Akzent zum Text)
   selection_star: parse_hex_color_or_panic( "#EBCB8B"),
   // Aktiver Code-Typ/Symbol (Nord8 - frisches Frost-Türkis)
   cb_type: parse_hex_color_or_panic( "#88C0D0"),
   // Inaktiver Code-Typ (Nord10 - dunkleres Frost-Blau)
   cb_type_inactive: parse_hex_color_or_panic( "#5E81AC"),
   // Zeitstempel (Nord7 - feines, unaufdringliches Frost-Grünblau)
   date_time: parse_hex_color_or_panic( "#8FBCBB"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "nord_bright".into(),
   window_bg: parse_hex_color_or_panic( "#3A4250"), // brighter than 0x29,0x2E,0x3A
   window_fg: parse_hex_color_or_panic( "#D8DEE9"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#BF616A"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#5A6378"), // brighter than 0x4C,0x56,0x6A
   text: parse_hex_color_or_panic( "#D8DEE9"),
   border: parse_hex_color_or_panic( "#81A1C1"),
   border_inactive: parse_hex_color_or_panic( "#5A6378"), // match line_number brightness
   menu: parse_hex_color_or_panic( "#525C70"),            // brighter than 0x43,0x4C,0x5E
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#BF616A"),
   cb_type: parse_hex_color_or_panic( "#81A1C1"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#5A6378"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "solarized".into(),
   window_bg: parse_hex_color_or_panic( "#00242F"),
   window_fg: parse_hex_color_or_panic( "#839496"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#B58900"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#586E75"),
   text: parse_hex_color_or_panic( "#839496"),
   border: parse_hex_color_or_panic( "#268BD2"),
   border_inactive: parse_hex_color_or_panic( "#586E75"),
   // menu: parse_hex_color_or_panic( "#0F4251"),
   menu: parse_hex_color_or_panic( "#124f61"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#B58900"),
   cb_type: parse_hex_color_or_panic( "#268BD2"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#586E75"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "solarized_bright".into(),
   window_bg: parse_hex_color_or_panic( "#023847"), // slightly brighter than 0x00,0x24,0x2F
   window_fg: parse_hex_color_or_panic( "#839496"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#B58900"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#667C83"), // brighter than 0x58,0x6E,0x75
   text: parse_hex_color_or_panic( "#839496"),
   border: parse_hex_color_or_panic( "#3A9FE0"),          // a touch brighter than 0x26,0x8B,0xD2
   border_inactive: parse_hex_color_or_panic( "#667C83"), // match line_number brightness
   menu: parse_hex_color_or_panic( "#1E6174"),            // brighter than 0x12,0x4F,0x61
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#B58900"),
   cb_type: parse_hex_color_or_panic( "#3A9FE0"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#667C83"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "dracula".into(),
   window_bg: parse_hex_color_or_panic( "#222430"),
   window_fg: parse_hex_color_or_panic( "#F8F8F2"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FF79C6"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#6272A4"),
   text: parse_hex_color_or_panic( "#F8F8F2"),
   border: parse_hex_color_or_panic( "#BD93F9"),
   border_inactive: parse_hex_color_or_panic( "#6272A4"),
   // menu: parse_hex_color_or_panic( "#4D5166"),
   menu: parse_hex_color_or_panic( "#5c617a"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#FF79C6"),
   cb_type: parse_hex_color_or_panic( "#BD93F9"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#6272A4"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "gruvbox".into(),
   window_bg: parse_hex_color_or_panic( "#1D2021"),
   window_fg: parse_hex_color_or_panic( "#EBDBB2"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FB4934"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#66554B"),
   text: parse_hex_color_or_panic( "#EBDBB2"),
   border: parse_hex_color_or_panic( "#FE8629"),
   border_inactive: parse_hex_color_or_panic( "#66554B"),
   // menu: parse_hex_color_or_panic( "#3C3836"),
   // menu: parse_hex_color_or_panic( "#544e4b"),
   menu: parse_hex_color_or_panic( "#5a5451"),
   pause: Some(COLOR_GREEN),
   selection_star: parse_hex_color_or_panic( "#FB4934"),
   cb_type: parse_hex_color_or_panic( "#FE8629"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#66554B"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "gruvbox_bright".into(),
   window_bg: parse_hex_color_or_panic( "#272A2B"), // slightly lighter than 0x1D,0x20,0x21
   window_fg: parse_hex_color_or_panic( "#EBDBB2"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FB4934"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#7A665B"), // brighter than 0x66,0x55,0x4B
   text: parse_hex_color_or_panic( "#EBDBB2"),
   border: parse_hex_color_or_panic( "#FE8629"),
   border_inactive: parse_hex_color_or_panic( "#7A665B"), // match line_number
   menu: parse_hex_color_or_panic( "#665F5C"),            // brighter than 0x5A,0x54,0x51
   pause: Some(COLOR_GREEN),
   selection_star: parse_hex_color_or_panic( "#FB4934"),
   cb_type: parse_hex_color_or_panic( "#FE8629"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#7A665B"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "monokai".into(),
   window_bg: parse_hex_color_or_panic( "#272822"),
   window_fg: parse_hex_color_or_panic( "#F8F8F2"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#F92672"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#585E5E"),
   text: parse_hex_color_or_panic( "#F8F8F2"),
   border: parse_hex_color_or_panic( "#A6E22E"),
   border_inactive: parse_hex_color_or_panic( "#585E5E"),
   // menu: parse_hex_color_or_panic( "#3B3C34"),
   menu: parse_hex_color_or_panic( "#585a4e"),
   pause: Some(COLOR_BRIGHT_BLUE),
   selection_star: parse_hex_color_or_panic( "#F92672"),
   cb_type: parse_hex_color_or_panic( "#A6E22E"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#585E5E"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "monokai_bright".into(),
   window_bg: parse_hex_color_or_panic( "#32332B"), // brighter than 0x27,0x28,0x22
   window_fg: parse_hex_color_or_panic( "#F8F8F2"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#F92672"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#697070"), // brighter than 0x58,0x5E,0x5E
   text: parse_hex_color_or_panic( "#F8F8F2"),
   border: parse_hex_color_or_panic( "#A6E22E"),
   border_inactive: parse_hex_color_or_panic( "#697070"), // match line_number brightness
   menu: parse_hex_color_or_panic( "#696B60"),            // brighter than 0x58,0x5A,0x4E
   pause: Some(COLOR_BRIGHT_BLUE),
   selection_star: parse_hex_color_or_panic( "#F92672"),
   cb_type: parse_hex_color_or_panic( "#A6E22E"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#697070"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "onedark".into(),
   window_bg: parse_hex_color_or_panic( "#282C34"),
   window_fg: parse_hex_color_or_panic( "#ABB2BF"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#E5C07B"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#4B5263"),
   text: parse_hex_color_or_panic( "#ABB2BF"),
   border: parse_hex_color_or_panic( "#61AFEF"),
   border_inactive: parse_hex_color_or_panic( "#4B5263"),
   // menu: parse_hex_color_or_panic( "#3E4452"),
   menu: parse_hex_color_or_panic( "#5d667b"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#E5C07B"),
   cb_type: parse_hex_color_or_panic( "#61AFEF"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#4B5263"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "onedark_bright".into(),
   window_bg: parse_hex_color_or_panic( "#4C5664"), // Lighter background
   window_fg: parse_hex_color_or_panic( "#D0D5DF"), // Brighter foreground
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#F5B05E"),    // Brighter cursor
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#6B7281"), // Lighter line numbers
   text: parse_hex_color_or_panic( "#D0D5DF"),        // Brighter text
   border: parse_hex_color_or_panic( "#89C4F3"),      // Brighter border
   border_inactive: parse_hex_color_or_panic( "#6B7281"),
   // menu: parse_hex_color_or_panic( "#7A8A97"),
   menu: parse_hex_color_or_panic( "#7A8A97"), // Brighter menu
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#F5B05E"),
   cb_type: parse_hex_color_or_panic( "#89C4F3"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#6B7281"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "catppuccin".into(),
   window_bg: parse_hex_color_or_panic( "#1E1E2E"),
   window_fg: parse_hex_color_or_panic( "#CDD6F4"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#F5C2E7"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#45455A"),
   text: parse_hex_color_or_panic( "#CDD6F4"),
   border: parse_hex_color_or_panic( "#89B4FA"),
   border_inactive: parse_hex_color_or_panic( "#45455A"),
   // menu: parse_hex_color_or_panic( "#31313F"),
   // menu: parse_hex_color_or_panic( "#49495e"),
   menu: parse_hex_color("#49495e"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#F5C2E7"),
   cb_type: parse_hex_color_or_panic( "#89B4FA"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#45455A"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "catppuccin_bright".into(),
   window_bg: parse_hex_color_or_panic( "#3A3A4A"), // Lighter background
   window_fg: parse_hex_color_or_panic( "#E6E9FF"), // Brighter foreground
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#F5C2E7"),    // Keep original cursor color for contrast
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#66666A"), // Brighter line numbers
   text: parse_hex_color_or_panic( "#E6E9FF"),        // Brighter text
   border: parse_hex_color_or_panic( "#A0BAFC"),      // Brighter border
   border_inactive: parse_hex_color_or_panic( "#66666A"),
   // menu: parse_hex_color_or_panic( "#4A4A5F"),
   menu: parse_hex_color_or_panic( "#606070"), // Brighter menu
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#F5C2E7"),
   cb_type: parse_hex_color_or_panic( "#A0BAFC"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#66666A"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "tokyonight".into(),
   window_bg: parse_hex_color_or_panic( "#1A1B26"),
   window_fg: parse_hex_color_or_panic( "#A9B1D6"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#BB9AF7"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#364356"),
   text: parse_hex_color_or_panic( "#A9B1D6"),
   border: parse_hex_color_or_panic( "#7AA2E3"),
   border_inactive: parse_hex_color_or_panic( "#364356"),
   menu: parse_hex_color_or_panic( "#444553"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#BB9AF7"),
   cb_type: parse_hex_color_or_panic( "#7AA2E3"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#364356"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "tokyonight_bright".into(),
   window_bg: parse_hex_color_or_panic( "#3C3D4A"), // Brighter background
   window_fg: parse_hex_color_or_panic( "#C0C5D8"), // Brighter foreground
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#D8B0F7"),    // Slightly brighter cursor
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#4A5560"), // Brighter line numbers
   text: parse_hex_color_or_panic( "#C0C5D8"),        // Brighter text
   border: parse_hex_color_or_panic( "#89B0F0"),      // Brighter border
   border_inactive: parse_hex_color_or_panic( "#4A5560"),
   menu: parse_hex_color_or_panic( "#585966"), // Brighter menu
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#D8B0F7"),
   cb_type: parse_hex_color_or_panic( "#89B0F0"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#4A5560"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "ayu".into(),
   window_bg: parse_hex_color_or_panic( "#0A0E14"),
   window_fg: parse_hex_color_or_panic( "#B3B1AD"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FF9900"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#4A4D52"),
   text: parse_hex_color_or_panic( "#B3B1AD"),
   border: parse_hex_color_or_panic( "#39BAE6"),
   border_inactive: parse_hex_color_or_panic( "#4A4D52"),
   // menu: parse_hex_color_or_panic( "#34383F"),
   menu: parse_hex_color_or_panic( "#4e545e"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#FF9900"),
   cb_type: parse_hex_color_or_panic( "#39BAE6"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#4A4D52"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "githubdark".into(),
   window_bg: parse_hex_color_or_panic( "#0D1117"),
   window_fg: parse_hex_color_or_panic( "#C9D1D9"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#58A6FF"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#484F5A"),
   text: parse_hex_color_or_panic( "#C9D1D9"),
   border: parse_hex_color_or_panic( "#1F6FEB"),
   border_inactive: parse_hex_color_or_panic( "#484F5A"),
   // menu: parse_hex_color_or_panic( "#363C44"),
   menu: parse_hex_color_or_panic( "#515a66"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#58A6FF"),
   cb_type: parse_hex_color_or_panic( "#1F6FEB"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#484F5A"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "materialdark".into(),
   window_bg: parse_hex_color_or_panic( "#263238"),
   window_fg: parse_hex_color_or_panic( "#EEFFFF"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#80CBFC"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#546775"),
   text: parse_hex_color_or_panic( "#EEFFFF"),
   border: parse_hex_color_or_panic( "#82AAFF"),
   border_inactive: parse_hex_color_or_panic( "#546775"),
   // menu: parse_hex_color_or_panic( "#374450"),
   menu: parse_hex_color_or_panic( "#526678"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#80CBFC"),
   cb_type: parse_hex_color_or_panic( "#82AAFF"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#546775"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "oceanicnext".into(),
   window_bg: parse_hex_color_or_panic( "#1B2B34"),
   window_fg: parse_hex_color_or_panic( "#C0C5CE"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#EC5F67"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#3B5062"),
   text: parse_hex_color_or_panic( "#C0C5CE"),
   border: parse_hex_color_or_panic( "#6699BB"),
   border_inactive: parse_hex_color_or_panic( "#3B5062"),
   menu: parse_hex_color("#41515E"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#EC5F67"),
   cb_type: parse_hex_color_or_panic( "#6699BB"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#3B5062"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "palenight".into(),
   window_bg: parse_hex_color_or_panic( "#29273D"),
   window_fg: parse_hex_color_or_panic( "#A7A6B2"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#C33DEE"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#625D7A"),
   text: parse_hex_color_or_panic( "#A7A6B2"),
   border: parse_hex_color_or_panic( "#89DDFF"),
   border_inactive: parse_hex_color_or_panic( "#625D7A"),
   menu: parse_hex_color_or_panic( "#5B586E"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#C33DEE"),
   cb_type: parse_hex_color_or_panic( "#89DDFF"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#625D7A"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "deepocean".into(),
   window_bg: parse_hex_color_or_panic( "#0A192B"),
   window_fg: parse_hex_color_or_panic( "#BBD0E0"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#4FB6DB"),
   cursor_inactive: None,
   // line_number: parse_hex_color_or_panic( "#1C2D42"),
   line_number: parse_hex_color_or_panic( "#2a4363"),
   text: parse_hex_color_or_panic( "#BBD0E0"),
   border: parse_hex_color_or_panic( "#3689B0"),
   border_inactive: parse_hex_color_or_panic( "#1C2D42"),
   // menu: parse_hex_color_or_panic( "#304458"),
   menu: parse_hex_color_or_panic( "#486684"),
   pause: Some(COLOR_GREEN),
   selection_star: parse_hex_color_or_panic( "#4FB6DB"),
   cb_type: parse_hex_color_or_panic( "#3689B0"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#2a4363"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "vscodium".into(),
   window_bg: parse_hex_color_or_panic( "#1E1E1E"),
   window_fg: parse_hex_color_or_panic( "#D4D4D4"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#009ACE"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#858585"),
   text: parse_hex_color_or_panic( "#D4D4D4"),
   border: parse_hex_color_or_panic( "#007FC8"),
   border_inactive: parse_hex_color_or_panic( "#858585"),
   menu: parse_hex_color_or_panic( "#454545"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#009ACE"),
   cb_type: parse_hex_color_or_panic( "#007FC8"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#858585"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "horizon".into(),
   window_bg: parse_hex_color_or_panic( "#1C1E28"),
   window_fg: parse_hex_color_or_panic( "#CBD0DC"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#EFB493"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#4B4E5F"),
   text: parse_hex_color_or_panic( "#CBD0DC"),
   border: parse_hex_color_or_panic( "#2AB3BD"),
   border_inactive: parse_hex_color_or_panic( "#4B4E5F"),
   menu: parse_hex_color_or_panic( "#4B4D5A"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#EFB493"),
   cb_type: parse_hex_color_or_panic( "#2AB3BD"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#4B4E5F"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "nightowl".into(),
   window_bg: parse_hex_color_or_panic( "#0B141F"),
   window_fg: parse_hex_color_or_panic( "#D270A1"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FCA47C"),
   cursor_inactive: None,
   // line_number: parse_hex_color_or_panic( "#2B384C"),
   line_number: parse_hex_color_or_panic( "#405472"),
   text: parse_hex_color_or_panic( "#D270A1"),
   border: parse_hex_color_or_panic( "#82AAFF"),
   border_inactive: parse_hex_color_or_panic( "#2B384C"),
   // menu: parse_hex_color_or_panic( "#283547"),
   menu: parse_hex_color_or_panic( "#3c4f6a"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#FCA47C"),
   cb_type: parse_hex_color_or_panic( "#82AAFF"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#405472"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "synthwave".into(),
   window_bg: parse_hex_color_or_panic( "#26173A"),
   window_fg: parse_hex_color_or_panic( "#F7EEE8"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FF79C6"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#ffd319"),
   text: parse_hex_color_or_panic( "#F7EEE8"),
   border: parse_hex_color_or_panic( "#00D8FF"),
   border_inactive: parse_hex_color_or_panic( "#523366"),
   menu: parse_hex_color_or_panic( "#4A385C"),
   pause: Some(COLOR_YELLOW),
   selection_star: parse_hex_color_or_panic( "#FF79C6"),
   cb_type: parse_hex_color_or_panic( "#00D8FF"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#ff901f"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "cyberpunk".into(),
   window_bg: parse_hex_color_or_panic( "#000512"),
   window_fg: parse_hex_color_or_panic( "#FFB867"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FF00FF"),
   cursor_inactive: None,
   // line_number: parse_hex_color_or_panic( "#2E243A"),
   line_number: parse_hex_color_or_panic( "#453657"),
   text: parse_hex_color_or_panic( "#FFB867"),
   border: parse_hex_color_or_panic( "#00F0FF"),
   border_inactive: parse_hex_color_or_panic( "#2E243A"),
   // menu: parse_hex_color_or_panic( "#383248"),
   menu: parse_hex_color_or_panic( "#544b6c"),
   pause: Some(COLOR_YELLOW),
   selection_star: parse_hex_color_or_panic( "#FF00FF"),
   cb_type: parse_hex_color_or_panic( "#00F0FF"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#453657"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "argonaut".into(),
   window_bg: parse_hex_color_or_panic( "#26262D"),
   window_fg: parse_hex_color_or_panic( "#EAE6DE"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FF0041"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#464450"),
   text: parse_hex_color_or_panic( "#EAE6DE"),
   border: parse_hex_color_or_panic( "#2BB5C6"),
   border_inactive: parse_hex_color_or_panic( "#464450"),
   menu: parse_hex_color_or_panic( "#53515A"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#FF0041"),
   cb_type: parse_hex_color_or_panic( "#2BB5C6"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#464450"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "bordeaux".into(),
   window_bg: parse_hex_color_or_panic( "#261C1C"),
   window_fg: parse_hex_color_or_panic( "#F59CA4"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FF728E"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#523436"),
   text: parse_hex_color_or_panic( "#F59CA4"),
   border: parse_hex_color_or_panic( "#DA6F7C"),
   border_inactive: parse_hex_color_or_panic( "#523436"),
   menu: parse_hex_color_or_panic( "#564646"),
   pause: Some(COLOR_CYAN),
   selection_star: parse_hex_color_or_panic( "#FF728E"),
   cb_type: parse_hex_color_or_panic( "#DA6F7C"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#523436"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "new_chinese".into(),
   window_bg: parse_hex_color_or_panic( "#1C1A20"),
   window_fg: parse_hex_color_or_panic( "#FF6B9D"), // orange
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FFD100"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#FF3D3D"), // red
   text: parse_hex_color_or_panic( "#FF8B4B"),        // orange
   border: parse_hex_color_or_panic( "#FFD4FF"),
   border_inactive: parse_hex_color_or_panic( "#FF1C1C"),
   menu: parse_hex_color_or_panic( "#FFF4A9"), // yellow
   pause: Some(COLOR_CYAN),
   selection_star: parse_hex_color_or_panic( "#FF6B3D"), // red
   cb_type: parse_hex_color_or_panic( "#FFD78F"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#FF3C3C"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "eastern_red".into(),
   window_bg: parse_hex_color_or_panic( "#1F2025"),
   window_fg: parse_hex_color_or_panic( "#FF5F70"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FF8C66"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#FF3035"),
   text: parse_hex_color_or_panic( "#FF8A6D"),
   border: parse_hex_color_or_panic( "#FFB24F"),
   border_inactive: parse_hex_color_or_panic( "#304048"),
   menu: parse_hex_color_or_panic( "#FF6B8F"),
   pause: Some(COLOR_RED),
   selection_star: parse_hex_color_or_panic( "#FF5F7D"),
   cb_type: parse_hex_color_or_panic( "#FFFF99"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#FF3035"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "ink_blue".into(),
   window_bg: parse_hex_color_or_panic( "#303338"),
   window_fg: parse_hex_color_or_panic( "#FF6B9D"),
   window_bg_header_footer: parse_hex_color_or_panic( "#000000"),
   cursor: parse_hex_color_or_panic( "#FF8C66"),
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#FF6B6B"),
   text: parse_hex_color_or_panic( "#FFD6D6"),
   border: parse_hex_color_or_panic( "#FF994F"),
   border_inactive: parse_hex_color_or_panic( "#404048"),
   menu: parse_hex_color_or_panic( "#FF6B8F"),
   pause: Some(COLOR_CYAN),
   selection_star: parse_hex_color_or_panic( "#FF8A7A"),
   cb_type: parse_hex_color_or_panic( "#FFE6E9"),
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#FF4048"),
   line_number_inactive: None,
   date_time_inactive: None,
  },
  ThemeColors {
   name: "palenight_light".into(),
   window_bg: parse_hex_color_or_panic( "#F2F1F8"), // Sehr helles, kühles Flieder-Weiß (Invertiert von #29273D")
   window_fg: parse_hex_color_or_panic( "#43415A"), // Tiefes, gut lesbares Dunkel-Violett (Invertiert von #A7A6B2")
   window_bg_header_footer: parse_hex_color_or_panic( "#ffffff"),
   cursor: parse_hex_color_or_panic( "#9C00CA"),    // Abgedunkeltes Magenta-Lila für hohen Kontrast
   cursor_inactive: None,
   line_number: parse_hex_color_or_panic( "#8D88A5"), // Mittleres Lavendel-Grau für dezente Zeilennummern
   text: parse_hex_color_or_panic( "#43415A"),        // Identisch mit window_fg für konsistenten Text
   border: parse_hex_color("#007EA7"), // Abgedunkeltes, kräftiges Cyan (Invertiert von #89DDFF)
   border_inactive: parse_hex_color_or_panic( "#B6B2CB"), // Helles, unauffälliges Grau-Violett
   menu: parse_hex_color_or_panic( "#74737d"),        // Heller, abgehobener Hintergrund für Menüs
   pause: Some(COLOR_RED),     // Bleibt rot, hebt sich auf hellem Grund gut ab
   selection_star: parse_hex_color_or_panic( "#9C00CA"), // Identisch mit dem aktiven Cursor
   cb_type: parse_hex_color("#007EA7"), // Identisch mit dem aktiven Rahmen
   cb_type_inactive: None,
   date_time: parse_hex_color_or_panic( "#8D88A5"), // Identisch mit line_number für dezente Metadaten
   line_number_inactive: None,
   date_time_inactive: None,
  },
 ]
}

pub(crate) fn all_themes_skipmap() -> SkipMap<String, ThemeColors> {
 let ret = SkipMap::<String, ThemeColors>::default();
 let tcs = create_theme_colors();
 for tc in tcs.iter() {
  let name = tc.name.clone();
  let mut tc = tc.clone();

  if tc.cursor_inactive.is_none() && tc.cursor.is_some() {
   tc.cursor_inactive = dim_color(tc.cursor)
  }

  if tc.cb_type_inactive.is_none() && tc.cb_type.is_some() {
   tc.cb_type_inactive = dim_color(tc.cb_type)
  }

  if tc.line_number_inactive.is_none() && tc.line_number.is_some() {
   tc.line_number_inactive = dim_color(tc.line_number)
  }

  if tc.date_time_inactive.is_none() && tc.date_time.is_some() {
   tc.date_time_inactive = dim_color(tc.date_time)
  }

  ret.insert(name.clone(), tc.clone());

  if name != "default" {
   let tc = brighten_tc_for_daylight(tc);
   ret.insert(tc.name.clone(), tc);
  }
 }
 ret
}

fn brighten_tc_for_daylight(mut tc: ThemeColors) -> ThemeColors {
 tc.name.push_str("_dl");
 tc.menu = brighten_for_daylight(tc.menu);
 tc.line_number = brighten_for_daylight(tc.line_number);
 tc.text = brighten_for_daylight(tc.text);
 tc.window_fg = brighten_for_daylight(tc.window_fg);
 tc.border = brighten_for_daylight(tc.border);
 tc.border_inactive = brighten_for_daylight(tc.border_inactive);
 tc.cb_type = brighten_for_daylight(tc.cb_type);
 tc.cb_type_inactive = brighten_for_daylight(tc.cb_type_inactive);
 tc.date_time = brighten_for_daylight(tc.date_time);
 tc
}

fn brighten_for_daylight(color: Option<Color>) -> Option<Color> {
 color.map(|mut x: Color| {
  const THRESHOLD: u8 = 175;
  if let Color::Rgb(r, g, b) = x {
   // ratatui provides no conversion from the enum to u8 u8 u8 for all enums
   let rgb_max = max(max(r, g), b);
   if rgb_max == 0 {
   } else if rgb_max < THRESHOLD {
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;
    let f = THRESHOLD as f32 / rgb_max as f32;
    let r = (f * r).min(255.) as u8;
    let g = (f * g).min(255.) as u8;
    let b = (f * b).min(255.) as u8;
    x = Color::Rgb(r, g, b);
   }
  }
  x
 })
}

pub(crate) fn default_color_theme_name() -> String {
 "default".into()
}
