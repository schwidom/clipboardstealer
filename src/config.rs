use std::{thread, time::Duration};

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(30);

pub const USAGE  : &str = r"

clipboardstealer [--debug]

- runs in a terminal window, 
- captures the X11 clipboards named: primary, secondary and clipboard
- allows selection of all three of them
- enforces the user choice

- Keys: 

 orientation: Cursor Up, Cursor Down, PgUp, PgDown, Home, End
 orientation: Cursor Left, Cursor Right (not implemented yet)

 regex search ... / (not implemented yet)

 (h)elp ... this screen 
 (v)iew ... shows the selected entry
 (s)elect ... selects the chosen entry and 
              enforces it for the specific 
              primary, secondary or clipboard clipboards

 (q)uit ... exits a screen
 e(x)it ... exits the program
 Ctrl-C ... exits the program

Copyright : Frank Schwidom 2025
This software is licensed under the terms of the Apache-2.0 license. ";

pub fn sleep_default() {
 // dbaphuses4, a0vbfusiba
 thread::sleep(DEFAULT_TIMEOUT);
}

#[derive(Clone)]
pub struct Config {
 pub debug: bool,
}

use crate::libmain::Args;

impl Config {
 pub fn from_args(args: &Args) -> Self {
  Self { debug: args.debug }
 }
}
