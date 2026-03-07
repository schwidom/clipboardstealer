use std::{thread, time::Duration};

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(30);

// TODO : to constants
pub const USAGE: &str = r"

clipboardstealer [--debug] [--debugfile <DEBUGFILE>] [--append-ndjson <APPEND_NDJSON>] [--load-ndjson <LOAD_NDJSON> | ...]

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

 orientation: Cursor Up, Cursor Down, PgUp, PgDown, Home, End
 orientation: Cursor Left, Cursor Right (not implemented yet)

 regex search ... / (not implemented yet)

 (h)elp ... this screen 
 (v)iew ... shows the selected entry
 (s)elect ... selects the chosen entry and 
              enforces it for the specific 
              primary, secondary or clipboard clipboards
 (fF)lip ... the layout
 (w)rap ... wraps the lines

 (q)uit ... exits a screen
 e(x)it ... exits the program
 Ctrl-C ... exits the program

Copyright : Frank Schwidom 2025 - 2026
This software is licensed under the terms of the Apache-2.0 license. ";

pub fn sleep_default() {
 // dbaphuses4, a0vbfusiba
 thread::sleep(DEFAULT_TIMEOUT);
}

#[derive(Clone)]
pub struct Config {
 pub debug: bool,
 pub debugfile: Option<String>,
 pub append_ndjson: Option<String>,
 pub load_ndjson: Vec<String>,
}

use crate::libmain::Args;

use std::fs::OpenOptions;

use tracing::{event, info, span, trace, Level};

impl Config {
 pub fn from_args(args: &Args) -> Self {
  if args.debug {
   if let Some(df) = args.debugfile.clone() {
    let file = OpenOptions::new()
     .create(true)
     .write(true)
     .append(true)
     .open(df)
     .expect("Failed to open log file");

    tracing_subscriber::fmt()
     .with_writer(file)
     .with_max_level(Level::TRACE) // TODO : setting via  clap / args
     .init(); // calls set_global_default
   } // TODO : else
  }

  Self {
   debug: args.debug,
   debugfile: args.debugfile.clone(),
   append_ndjson: args.append_ndjson.clone(),
   load_ndjson: args.load_ndjson.clone(),
  }
 }
}
