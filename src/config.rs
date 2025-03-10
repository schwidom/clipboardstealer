use std::{thread, time::Duration};

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(30);

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
