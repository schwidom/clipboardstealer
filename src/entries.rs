use std::{
 ops::{Deref, DerefMut},
 sync::{Arc, Mutex},
};

use crate::{clipboards::ClipboardSelectionList, tools::MyTime};

#[derive(Clone)]
pub struct Entry {
 pub info: String,
 pub timestamp: MyTime,
 pub string: String,
 pub csl: Arc<Mutex<ClipboardSelectionList>>,
 pub csl_idx: usize,
}

impl Entry {
 pub fn get_date_time(&self) -> String {
  let ret = format!("{}", self.timestamp);
  // 2025-02-24 20:25:40+01:00
  ret[0..19].into() // 2025-02-24 20:25:40
 }

 pub fn select(&self) {
  let mut csl = self.csl.lock().unwrap();
  csl.current_selection = Some(self.csl_idx);
 }

 pub fn deselect(&self) {
  let mut csl = self.csl.lock().unwrap();
  csl.current_selection = None;
 }
}

// pub type Entries = Vec<Entry>;
pub struct Entries(Vec<Entry>);

impl Entries {
 pub fn from_csl(info: &str, csl: &Arc<Mutex<ClipboardSelectionList>>) -> Self {
  let csl_cloned = csl.clone();
  let csl_locked = csl_cloned.lock().unwrap();
  let ret = csl_locked
   .captured_from_clipboard
   .iter()
   .enumerate()
   .map(|(csl_idx, x)| Entry {
    info: info.into(),
    timestamp: x.0.clone(),
    string: x.1.clone(),
    csl: csl_cloned.clone(),
    csl_idx,
   })
   .collect();
  Self(ret)
 }
}

impl Deref for Entries {
 type Target = Vec<Entry>;

 fn deref(&self) -> &Self::Target {
  &self.0
 }
}

impl DerefMut for Entries {
 fn deref_mut(&mut self) -> &mut Self::Target {
  &mut self.0
 }
}
