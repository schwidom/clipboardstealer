use lazy_static::lazy_static;
use std::{
 sync::{
  atomic::{AtomicBool, Ordering},
  RwLock, RwLockWriteGuard,
 },
 thread,
 time::Duration,
};
use version::version;

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(30);

lazy_static! {
 pub static ref USAGE: String = format!(
  r"
clipboardstealer version {}

Usage: clipboardstealer [OPTIONS]

Options:
      --append-ndjson-bin <APPEND_NDJSON_BIN>
          appends clipboard information to file
      --load-ndjson-bin <LOAD_NDJSON_BIN>
          reads clipboard information from file
      --load-and-append-ndjson-bin <LOAD_AND_APPEND_NDJSON_BIN>
          loads clipboard information from file and appends to it
      --append-ndjson <APPEND_NDJSON>
          appends clipboard information to file (JSON String format)
      --load-ndjson <LOAD_NDJSON>
          reads clipboard information from file (JSON String format)
      --load-and-append-ndjson <LOAD_AND_APPEND_NDJSON>
          loads clipboard information from file and appends to it (JSON String format)
      --editor
          interprets the EDITOR environment variable always as editor
      --convert-bin-ndjson <CONVERT_BIN_NDJSON>
          converts bin ndjson to string ndjson (input file)
      --to-string-ndjson <TO_STRING_NDJSON>
          output file for converted bin ndjson
      --convert-string-ndjson <CONVERT_STRING_NDJSON>
          converts string ndjson to bin ndjson (input file)
      --to-bin-ndjson <TO_BIN_NDJSON>
          output file for converted string ndjson
      -c, --color <COLOR>
          select color theme (default, nord, solarized, dracula)
      --color-themes
          list available color themes
      --debug
          provides debug information
      --debugfile <DEBUGFILE>
          writes debug information into file
  -h, --help
          Print help
  -V, --version
          Print version


Overview:

- is a clipboard manager
- runs in a terminal window, 
- captures the X11 clipboards named: primary, secondary and clipboard
- works also with xwayland (tested on debian 13)
- allows selection of all three of them
- enforces the user choice (on shortcut s)

Installation:

- apt-get install libxcb1-dev # needed
- cargo install clipboardstealer

- this crate is not intended to be used as a library


Keys: 

 orientation: Up, Down, PgUp, PgDown, Home, End
 orientation: Left, Right, Shift Left, Shift Right

 / (push), r (pop) ... stacked regex search

 (h)elp   ... this screen 
 (v)iew   ... shows the selected entry
 (e)dit   ... edit the selected entry
 (d)elete ... deletes the selected entry
 (t)oggle ... toggles the contents of the clipboards 'primary' and 'clipboards'

 (s)elect ... selects the chosen entry and 
              enforces it for the specific 
              primary, secondary or clipboard clipboards
 (t)oggle ... toggles primary <-> clipboard
 (fF)lip ... the layout
 (w)rap  ... wraps the lines
 (p)ause ... pauses the clipboard scanning, continues with p

 Esc     ... discard status messages
 Esc     ... stop regex editing

 Tab    ... switch windows

 (q)uit  ... exits a screen
 e(x)it  ... exits the program
 Ctrl-C  ... exits the program

URLs : 
https://crates.io/crates/clipboardstealer
https://github.com/schwidom/clipboardstealer

Copyright : Frank Schwidom 2025 - 2026
This software is licensed under the terms of the Apache-2.0 license.

",
  version!()
 );
}

pub fn sleep_default() {
 // dbaphuses4, a0vbfusiba
 thread::sleep(DEFAULT_TIMEOUT);
}

#[derive(Default, Debug)]
pub(crate) struct Paused {
 pub paused: AtomicBool,
}

impl Paused {
 pub(crate) fn new(value: bool) -> Self {
  Self {
   paused: AtomicBool::new(value),
  }
 }
 pub(crate) fn is_paused(&self) -> bool {
  self.paused.load(Ordering::Relaxed)
 }

 pub(crate) fn toggle(&self) {
  self.paused.store(!self.is_paused(), Ordering::Relaxed);
 }
}

// #[derive(Clone)]
#[derive(Debug, Default)]
pub struct Config {
 pub debug: bool,
 pub debugfile: Option<String>,
 pub append_ndjson_bin: Option<String>,
 pub load_ndjson_bin: Vec<String>,
 pub append_ndjson_string: Option<String>,
 pub load_ndjson_string: Vec<String>,
 pub editor: bool,
 pub color_theme: crate::color_theme::ColorTheme,
 pub custom_theme_colors: Option<crate::color_theme::ThemeColors>,
 pub suspend_threads: RwLock<()>,
 pub suspended_threads: AtomicBool,
 pub paused: Paused,
}

use crate::libmain::Args;

use std::fs::OpenOptions;

use tracing::Level;

impl Config {
 pub fn from_args(
  args: &Args,
  custom_theme_colors: Option<crate::color_theme::ThemeColors>,
 ) -> Self {
  if args.debug {
   if let Some(df) = args.debugfile.clone() {
    let file = OpenOptions::new()
     .create(true)
     .append(true)
     .open(df)
     .expect("Failed to open log file");

    tracing_subscriber::fmt()
     .with_writer(file)
     .with_max_level(Level::TRACE) // TODO : setting via  clap / args
     .init(); // calls set_global_default
   } // TODO : else
  }

  // q3jhk95ow6
  let (append_ndjson_bin, load_ndjson_bin) = if let Some(file) = &args.load_and_append_ndjson_bin {
   let mut loads = args.load_ndjson_bin.clone();
   loads.push(file.clone());
   (Some(file.clone()), loads)
  } else {
   (args.append_ndjson_bin.clone(), args.load_ndjson_bin.clone())
  };

  // q3jhk95ow6
  let (append_ndjson_string, load_ndjson_string) = if let Some(file) = &args.load_and_append_ndjson
  {
   let mut loads = args.load_ndjson.clone();
   loads.push(file.clone());
   (Some(file.clone()), loads)
  } else {
   (args.append_ndjson.clone(), args.load_ndjson.clone())
  };

  Self {
   debug: args.debug,
   debugfile: args.debugfile.clone(),
   append_ndjson_bin,
   load_ndjson_bin,
   append_ndjson_string,
   load_ndjson_string,
   editor: args.editor,
   color_theme: args.color_theme,
   custom_theme_colors,
   suspend_threads: RwLock::new(()),
   suspended_threads: AtomicBool::new(false),
   paused: Paused::new(args.paused),
  }
 }

 pub(crate) fn wait_for_external_program(&self) {
  self.suspended_threads.store(true, Ordering::Relaxed);
  let _x = self.suspend_threads.read().unwrap();
  self.suspended_threads.store(false, Ordering::Relaxed);
 }
 pub(crate) fn block_threads_for_external_program(&self) -> RwLockWriteGuard<'_, ()> {
  self.suspend_threads.write().unwrap()
 }
 pub(crate) fn is_blocked_for_external_program(&self) -> bool {
  self.suspended_threads.load(Ordering::Relaxed)
 }

 pub fn save_theme_to_file(&self, path: &str) -> Result<(), String> {
  let json = self.color_theme.to_json();
  std::fs::write(path, json).map_err(|e| format!("Failed to write theme file: {}", e))
 }

 pub fn load_theme_from_file(&self, path: &str) -> Result<crate::color_theme::ColorTheme, String> {
  let content =
   std::fs::read_to_string(path).map_err(|e| format!("Failed to read theme file: {}", e))?;
  let theme_colors = crate::color_theme::ColorTheme::from_json(&content)?;
  for (_name, theme) in crate::color_theme::ColorTheme::all_themes() {
   let builtin = theme.get_colors();
   if Self::colors_equal(&builtin, &theme_colors) {
    return Ok(*theme);
   }
  }
  Err("No matching built-in theme found".to_string())
 }

 fn colors_equal(a: &crate::color_theme::ThemeColors, b: &crate::color_theme::ThemeColors) -> bool {
  Self::color_eq(&a.window_bg, &b.window_bg)
   && Self::color_eq(&a.window_fg, &b.window_fg)
   && Self::color_eq(&a.cursor, &b.cursor)
   && Self::color_eq(&a.line_number, &b.line_number)
   && Self::color_eq(&a.text, &b.text)
   && Self::color_eq(&a.border, &b.border)
   && Self::color_eq(&a.border_inactive, &b.border_inactive)
   && Self::color_eq(&a.menu, &b.menu)
 }

 fn color_eq(a: &Option<ratatui::style::Color>, b: &Option<ratatui::style::Color>) -> bool {
  match (a, b) {
   (Some(ratatui::style::Color::Rgb(r1, g1, b1)), Some(ratatui::style::Color::Rgb(r2, g2, b2))) => {
    r1 == r2 && g1 == g2 && b1 == b2
   }
   (None, None) => true,
   _ => false,
  }
 }
}
