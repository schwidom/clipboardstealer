use std::fmt::Display;

use chrono::DateTime;
use chrono::TimeDelta;

use chrono::Local;
use chrono::Utc;
use x11_clipboard::Atoms;
use x11_clipboard::Clipboard;

#[derive(Clone, PartialEq, Debug, PartialOrd, Eq, Ord)]
pub struct MyTime {
 pub timestamp: DateTime<Local>,
}

impl Display for MyTime {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
  self.timestamp.fmt(f)
 }
}

// cannot create a const function
pub fn create_local_unix_epoch() -> DateTime<Local> {
 const UNIX_EPOCH_UTC: DateTime<Utc> = DateTime::<Utc>::UNIX_EPOCH;
 let timestamp: DateTime<Local> = DateTime::from(UNIX_EPOCH_UTC);
 timestamp
}

impl MyTime {
 pub fn unix_epoch() -> Self {
  Self {
   timestamp: create_local_unix_epoch(),
  }
 }
 pub fn now() -> Self {
  Self {
   timestamp: Local::now(),
  }
 }

 pub fn elapsed(&self) -> TimeDelta {
  Local::now() - self.timestamp
 }
}

use lazy_static::lazy_static;

lazy_static! {
 pub static ref CB_ATOMS: Atoms = {
  let cb = Clipboard::new().unwrap();
  cb.setter.atoms.clone()
 };
}

pub fn flatline(string: &str) -> String {
 string.replace("\n", "\\n") // lcibiwnao0
}

pub fn tabfix(string: &str) -> String {
 string.replace("\t", "   ")
}
