use std::ops::{Deref, DerefMut};

use crate::{
 clipboards::{ClipboardSelectionList, Clipboards},
 tools::MyTime,
};

#[derive(Clone)]
pub struct Entry {
 pub info: String, // TODO : invent struct for this functionality (p, s, c)
 pub timestamp: MyTime,
 pub string: String,
 // pub csl: &'a ClipboardSelectionList,
 // pub csladdress: usize, // funktioniert nicht // TODO : magic for cbs
 pub csl_idx: usize,
}

impl Entry {
 pub fn get_date_time(&self) -> String {
  let ret = format!("{}", self.timestamp);
  // 2025-02-24 20:25:40+01:00
  ret[0..19].into() // 2025-02-24 20:25:40
 }

 pub fn toggle_selection(&self, cbs: &mut Clipboards) {
  let csl = cbs.hm.get_mut(&self.info).unwrap();
  if csl.current_selection == Some(self.csl_idx) {
   csl.current_selection = None
  } else {
   csl.current_selection = Some(self.csl_idx);
   cbs
    .hm
    .get(&self.info)
    .unwrap()
    .crw
    .write(self.string.clone());
  }
 }

 // TODO
 pub fn deselect(&self) {
  // let mut csl = self.csl.lock().unwrap();
  // csl.current_selection = None;
 }

 pub(crate) fn is_selected(&self, cbs: &Clipboards) -> bool {
  let csl = cbs.hm.get(&self.info).unwrap();
  // assert_eq!( self.csladdress, std::ptr::addr_of!(csl) as usize);
  Some(self.csl_idx) == csl.current_selection
 }
}

// pub type Entries = Vec<Entry>;
pub struct Entries(Vec<Entry>);

impl Entries {
 pub fn from_csl(info: &str, csl: &ClipboardSelectionList) -> Self {
  // let csl_cloned = csl.clone();
  // let csl_locked = csl_cloned.lock().unwrap();
  let ret = csl
   .captured_from_clipboard
   .iter()
   .enumerate()
   .map(|(csl_idx, x)| Entry {
    info: info.into(),
    timestamp: x.0.clone(),
    string: x.1.clone(),
    // csl,
    // csladdress : std::ptr::addr_of!(csl) as usize,
    csl_idx,
   })
   .collect();
  Self(ret)
 }
}

impl<'a> Deref for Entries {
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
