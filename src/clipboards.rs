// use std::borrow::Borrow; // TODO : why does this lead to an compiler error?
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::TimeDelta;
use enum_iterator::all;
use enum_iterator::Sequence;
use serde::Deserialize;
use serde::Serialize;
use tracing::trace;
use x11_clipboard as x11;
// use x11_clipboard::error::Error as X11Error;

// use x11::Atoms;

use x11::Clipboard;

// use x11::xcb::Atom;
use x11::Atom;
use x11rb_protocol::protocol::xproto::AtomEnum;

use crate::config::sleep_default;
use crate::libmain::CbsError;
// use crate::entries::Entries;
// use crate::entries::Entry;
// use crate::libmain::MyError;
// use crate::tools::cb_get_atoms;
use crate::tools::MyTime;
use crate::tools::CB_ATOMS;

use cbentry::CBEntry;

// impl From<X11Error> for MyError {
//  fn from(value: X11Error) -> Self {
//   MyError::X11Clipboard(value)
//  }
// }

// let mut cfmap = HashMap::new();

#[derive(Sequence, PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub enum CBType {
 Primary,   // mouse selection, shift-ins, middle mouse
 Secondary, // unknown keys, ancient clipboard
 Clipboard, // [shift] ( ctrl-c / ctrl-v )
}

impl CBType {
 fn from_atom(atom: Atom) -> Self {
  let p: Atom = AtomEnum::PRIMARY.into(); // 1, CB_ATOMS.primary
  let s: Atom = AtomEnum::SECONDARY.into(); // 2
  let c: Atom = CB_ATOMS.clipboard;

  if atom == p {
   Self::Primary
  } else if atom == s {
   Self::Secondary
  } else if atom == c {
   Self::Clipboard
  } else {
   unreachable!("Unknown clipboard atom received");
  }
 }

 fn get_atom(&self) -> Atom {
  let p: Atom = AtomEnum::PRIMARY.into(); // 1, CB_ATOMS.primary
  let s: Atom = AtomEnum::SECONDARY.into(); // 2
  let c: Atom = CB_ATOMS.clipboard;

  match self {
   CBType::Primary => p,
   CBType::Secondary => s,
   CBType::Clipboard => c,
  }
 }

 pub fn get_info(&self) -> String {
  match self {
   CBType::Primary => "p",
   CBType::Secondary => "s",
   CBType::Clipboard => "c",
  }
  .into()
 }
}

/** simplifies the reading / writing to a specific clipboard ( primary and clipboard) */
pub struct ClipboardReaderWriter {
 cb: Clipboard,
 atom: Atom,
 // atoms: Atoms,
 // echofree: Arc<Mutex<HashSet<String>>>,
 echofree: Arc<Mutex<HashSet<Vec<u8>>>>,
 echofree_read: AtomicBool,
}

#[derive(Default, PartialEq)]
pub struct CrwReadInfo {
 pub text: Option<String>,
 pub echofree: bool,
}

impl ClipboardReaderWriter {
 pub(crate) fn echofree(&self) -> Arc<Mutex<HashSet<Vec<u8>>>> {
  self.echofree.clone()
 }

 pub(crate) fn from_cbtype(cbtype: &CBType) -> Result<Self, CbsError> {
  let cb = Clipboard::new()?;
  Ok(Self {
   cb,
   atom: cbtype.get_atom(),
   echofree: Arc::new(Mutex::new(HashSet::new())),
   echofree_read: AtomicBool::new(false),
  })
 }

 pub(crate) fn from_cbtype_with_echofree(
  cbtype: &CBType,
  echofree: Arc<Mutex<HashSet<Vec<u8>>>>,
 ) -> Result<Self, CbsError> {
  let cb = Clipboard::new()?;
  Ok(Self {
   cb,
   atom: cbtype.get_atom(),
   echofree,
   echofree_read: AtomicBool::new(false),
  })
 }

 pub fn cbtype(&self) -> CBType {
  CBType::from_atom(self.atom)
 }

 // TODO : when I do "$echo 'secondary' | xclip -i -t secondary" then I get an error here
 // I don't exactly know how to handle that case
 // xclip -t TARGETS -o outputs : TARGETS\nUTF8_STRING
 // xclip -i -t UTF8_STRING : fills the primary clipboard
 // this fills the 3 clipboards likewise
 // $ echo primary | xclip -i -selection primary
 // $ echo clipboard | xclip -i -selection clipboard
 // $ echo secondary | xclip -i -selection secondary
 // $ xclip -o -selection primary
 // primary
 // $ xclip -o -selection secondary
 // secondary
 // $ xclip -o -selection clipboard
 // clipboard

 pub fn crw_read(&self) -> Option<Vec<u8>> {
  let selection = self.atom;

  match self
   .cb
   .load(selection, CB_ATOMS.utf8_string, CB_ATOMS.property, Duration::from_secs(3))
  {
   Ok(selection_u8) => {
    let mut echofree = self.echofree.lock().unwrap();
    let text = selection_u8;
    let echofree_bool = echofree.contains(&text);
    trace!("crw_read text :{:?}", text);
    trace!("crw_read echofree :{:?}", self.echofree.lock().unwrap());
    if !echofree_bool
     && self
      .echofree_read
      .load(std::sync::atomic::Ordering::Relaxed)
    {
     // self.echofree.lock().unwrap().clear();
     echofree.clear();
    }
    self
     .echofree_read
     .store(true, std::sync::atomic::Ordering::Relaxed);
    if echofree_bool {
     None
    } else {
     Some(text)
    }
   }

   Err(_) => None,
  }
 }

 pub fn crw_write(&self, s: String) -> bool {
  let value = s.as_bytes();
  let selection = self.atom;

  self
   .cb
   .store(selection, CB_ATOMS.utf8_string, value)
   .map_or_else(|_| false, |_| true)
 }

 pub fn crw_write_echofree(&self, s: Vec<u8>) -> bool {
  let mut echofree = self.echofree.lock().unwrap();
  echofree.insert(s.clone());
  self
   .echofree_read
   .store(false, std::sync::atomic::Ordering::Relaxed);
  // let x = self.echofree.lock().unwrap().insert(s.clone());
  // trace!("crw_write_echofree :{:?}", self.echofree.lock().unwrap());
  trace!("crw_write_echofree :{:?}", echofree);
  let value = s;
  let selection = self.atom;

  self
   .cb
   .store(selection, CB_ATOMS.utf8_string, value)
   .map_or_else(|_| false, |_| true)
 }
}

pub mod cbentry {
 use super::CBType;
 use super::MyTime;
 use serde::Deserialize;
 use serde::Serialize;
 use std::borrow::Cow;
 use std::cell::OnceCell;

 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct CBEntry {
  cbtype: CBType,
  timestamp: MyTime,
  data: Vec<u8>,
  #[serde(skip)]
  text: OnceCell<Vec<String>>,
 }

 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct CBEntryString {
  cbtype: CBType,
  timestamp: MyTime,
  text: String,
 }

 impl CBEntry {
  pub fn as_json_entry(&self) -> CBEntryString {
   CBEntryString {
    cbtype: self.cbtype.clone(),
    timestamp: self.timestamp.clone(),
    text: self.as_string().into_owned(),
   }
  }

  pub fn from_json_entry(json_entry: CBEntryString) -> Self {
   Self {
    cbtype: json_entry.cbtype.clone(),
    timestamp: json_entry.timestamp.clone(),
    data: json_entry.text.into_bytes(),
    text: OnceCell::default(),
   }
  }
 }

 impl CBEntry {
  pub fn new(data: &[u8]) -> Self {
   Self {
    cbtype: CBType::Clipboard,
    timestamp: MyTime::now(),
    data: Vec::from(data),
    text: OnceCell::default(),
   }
  }

  pub fn get_date_time(&self) -> String {
   let ret = format!("{}", self.timestamp);
   // 2025-02-24 20:25:40+01:00
   ret[0..19].into() // 2025-02-24 20:25:40
  }

  pub fn get_cbtype(&self) -> CBType {
   self.cbtype.clone()
  }

  pub fn get_timestamp(&self) -> MyTime {
   self.timestamp.clone()
  }

  pub fn get_data(&self) -> &Vec<u8> {
   &self.data
  }

  pub fn set_data(&mut self, data: &[u8]) {
   self.data = Vec::from(data);
   self.text = OnceCell::default();
  }
  pub fn as_string(&self) -> Cow<'_, str> {
   // besser Vec<u8> in einem CBEText unterbringen, ggf. mit einem String Cow oder gleich einem String // u79a6domic
   // dann ist aber CBEntry wieder inkompatibel zum append speicherformat
   String::from_utf8_lossy(&self.data)
  }

  pub fn get_text(&self) -> &Vec<String> {
   self.text.get_or_init(|| {
    self
     .as_string()
     .lines()
     .map(|x| x.to_string())
     .collect::<Vec<String>>()
   })
  }

  pub fn swap_data(&mut self, other: &mut Self) {
   std::mem::swap(&mut self.data, &mut other.data);
   self.text = OnceCell::default();
   other.text = OnceCell::default();
  }

  pub fn from_cbtype_timestamp_data(cbtype: &CBType, timestamp: &MyTime, data: &[u8]) -> Self {
   Self {
    cbtype: cbtype.clone(),
    timestamp: timestamp.clone(),
    data: Vec::from(data),
    text: OnceCell::default(),
   }
  }
 }
}
pub(crate) struct ClipboardFixation {
 pub crw: ClipboardReaderWriter,
 pub fixation: Option<AppendedCBEntry>,
}

impl ClipboardFixation {
 fn from_cbtype(cbtype: &CBType) -> Result<Self, CbsError> {
  Ok(Self {
   crw: ClipboardReaderWriter::from_cbtype(cbtype)?,
   fixation: None,
  })
 }

 /// writes the values back to its X11 clipboards
 fn restore(&self) {
  if let Some(fixation) = &self.fixation {
   self
    .crw
    .crw_write_echofree(fixation.cbentry.borrow().get_data().clone());
  }
 }
}

#[derive(Clone, Debug)]
pub struct AppendedCBEntry {
 pub appended_bin: bool,
 pub appended_string: bool,
 pub cbentry: Rc<RefCell<CBEntry>>,
 pub seq: usize,
}

/** managed clipboards by [crate::libmain::ClipboardThread] */
pub struct Clipboards {
 // pub hm: HashMap<String, ClipboardSelectionList>,
 // pub crw: ClipboardReaderWriter,
 pub cbentries: VecDeque<AppendedCBEntry>,
 pub last_entries: HashMap<CBType, AppendedCBEntry>,
 // NOTE : no weak pointer here, Optional<Rc> is better,
 // even if entry disappears from the list (currently not possible but maybe later)
 // it can still be selected
 // pub fixation: HashMap< String, Option<Rc<CBEntry>>>,
 // pub cfmap: HashMap<&'static str, ClipboardFixation>, // macht probleme beim indexieren
 // pub cfmap: HashMap<String, ClipboardFixation>,
 pub(crate) cfmap: HashMap<CBType, ClipboardFixation>,
 append_file_bin: Option<File>,
 append_file_bin_error_reported: bool,
 append_file_string: Option<File>,
 append_file_string_error_reported: bool,
 pub(crate) seq_counter: usize,
}

impl Default for Clipboards {
 fn default() -> Self {
  Self::new()
 }
}

impl Clipboards {
 pub fn new() -> Self {
  let cfmap: HashMap<CBType, ClipboardFixation> = all::<CBType>()
   .filter_map(|cbtype| match ClipboardFixation::from_cbtype(&cbtype) {
    Ok(cf) => Some((cbtype.clone(), cf)),
    Err(_) => None,
   })
   .collect();

  Self {
   cbentries: VecDeque::new(),
   last_entries: HashMap::new(),
   cfmap,
   append_file_bin: None,
   append_file_bin_error_reported: false,
   append_file_string: None,
   append_file_string_error_reported: false,
   seq_counter: 0,
  }
 }

 pub(crate) fn insert(&mut self, cbtype: &CBType, string: Option<Vec<u8>>) {
  if let Some(s) = string {
   let mut insert: bool = true;

   {
    let cf: &ClipboardFixation = &self.cfmap[cbtype];

    if let Some(fixation) = &cf.fixation {
     if fixation.cbentry.borrow().get_data() == &s {
      insert = false;
     } else {
      sleep_default();
      sleep_default();
      sleep_default();
      // TODO : configurable rewrite delay
      cf.restore();
     }
    }
   }

   {
    if let Some(appended_cbentry) = self.last_entries.get(cbtype) {
     if appended_cbentry.cbentry.borrow().get_data() == &s {
      insert = false;
     }
    }
   }

   if insert {
    let now = MyTime::now();
    let should_pop_front = if let Some(last) = self.cbentries.front() {
     let last_time = &last.cbentry.borrow().get_timestamp();
     let span = now.timestamp - last_time.timestamp;
     cbtype == &last.cbentry.borrow().get_cbtype() && span < TimeDelta::milliseconds(300)
    } else {
     false
    };
    if should_pop_front {
     self.cbentries.pop_front();
    }

    let cbentry = CBEntry::from_cbtype_timestamp_data(cbtype, &now, &s);

    let cbentry = Rc::new(RefCell::new(cbentry));
    let seq = self.seq_counter;
    self.cbentries.push_front(AppendedCBEntry {
     appended_bin: false,
     appended_string: false,
     cbentry: cbentry.clone(), // (now, s.clone())
     seq,
    });
    // self.last_entries.get_mut(&cbentry.borrow().cbtype) = cbentry;
    self.last_entries.insert(
     cbentry.borrow().get_cbtype(),
     AppendedCBEntry {
      appended_bin: false,
      appended_string: false,
      cbentry: cbentry.clone(),
      seq,
     },
    );
    self.seq_counter += 1;
   }
  }
 }

 pub fn get_entries(&self) -> &VecDeque<AppendedCBEntry> {
  &self.cbentries
 }

 pub(crate) fn append_ndjson_bin(&mut self, append_filename_string: &str) -> Result<(), String> {
  if self.append_file_bin.is_none() && !self.append_file_bin_error_reported {
   match OpenOptions::new()
    .create(true)
    .append(true)
    .open(Path::new(append_filename_string))
   {
    Ok(file) => {
     self.append_file_bin = Some(file);
    }
    Err(e) => {
     self.append_file_bin_error_reported = true;
     return Err(format!("Failed to open append bin file: {:?} - {}", append_filename_string, e));
    }
   }
  }

  if self.append_file_bin_error_reported {
   return Ok(());
  }

  let now = MyTime::now();

  let _: () = if let Some(ref mut fd) = self.append_file_bin {
   for cbentry in &mut self.cbentries {
    if cbentry.appended_bin {
     break;
    } else {
     let span = now.timestamp - cbentry.cbentry.borrow().get_timestamp().timestamp;
     if span > TimeDelta::milliseconds(300) {
      let json_str = serde_json::to_string(&*cbentry.cbentry)
       .map_err(|e| format!("Serialization error: {}", e))?;
      writeln!(fd, "{}", json_str).map_err(|e| format!("Write error: {}", e))?;
      cbentry.appended_bin = true;
     }
    }
   }
   fd.flush().map_err(|e| format!("flush : {}", e))?
  };
  Ok(())
 }

 pub fn append_ndjson_string(&mut self, append_filename_string: &str) -> Result<(), String> {
  if self.append_file_string.is_none() && !self.append_file_string_error_reported {
   match OpenOptions::new()
    .create(true)
    .append(true)
    .open(Path::new(append_filename_string))
   {
    Ok(file) => {
     self.append_file_string = Some(file);
    }
    Err(e) => {
     self.append_file_string_error_reported = true;
     return Err(format!(
      "Failed to open append string file: {:?} - {}",
      append_filename_string, e
     ));
    }
   }
  }

  if self.append_file_string_error_reported {
   return Ok(());
  }

  let now = MyTime::now();

  let _: () = if let Some(ref mut fd) = self.append_file_string {
   for cbentry in &mut self.cbentries {
    if cbentry.appended_string {
     break;
    } else {
     let span = now.timestamp - cbentry.cbentry.borrow().get_timestamp().timestamp;
     if span > TimeDelta::milliseconds(300) {
      let json_entry = cbentry.cbentry.borrow().as_json_entry();
      let json_str =
       serde_json::to_string(&json_entry).map_err(|e| format!("Serialization error: {}", e))?;
      writeln!(fd, "{}", json_str).map_err(|e| format!("Write error: {}", e))?;
      cbentry.appended_string = true;
     }
    }
   }
   fd.flush().map_err(|e| format!("flush : {}", e))?
  };
  Ok(())
 }

 pub fn is_fixated(&self, cbentry: &Rc<RefCell<CBEntry>>) -> bool {
  self
   .cfmap
   .iter()
   .filter(|x| match &x.1.fixation {
    Some(f) => Rc::<RefCell<CBEntry>>::ptr_eq(&f.cbentry, cbentry),
    None => false,
   })
   .count()
   != 0
 }

 pub(crate) fn toggle_fixation(&mut self, appended_cbentry: &AppendedCBEntry) {
  let cbentry = &appended_cbentry.cbentry;
  let cf = match self.cfmap.get_mut(&cbentry.borrow().get_cbtype()) {
   Some(cf) => cf,
   None => {
    trace!("toggle_selection: cbtype not found in cfmap {:?}", &cbentry.borrow().get_cbtype());
    return;
   }
  };

  trace!("toggle_fixation");

  let insert = match &cf.fixation {
   Some(f) => !Rc::<RefCell<CBEntry>>::ptr_eq(&f.cbentry, cbentry),
   None => true,
  };

  trace!("toggle_fixation : insert : {insert}");

  if insert {
   cf.fixation = Some(appended_cbentry.clone());
   cf.restore();
   self
    .last_entries
    .insert(appended_cbentry.cbentry.borrow().get_cbtype(), appended_cbentry.clone());
  } else {
   cf.fixation = None
  }
 }

 pub(crate) fn refresh_fixation(&self) {
  for cf in self.cfmap.values() {
   cf.restore();
  }
 }

 // fn get_clipboard_contents_of_cbtype(&self, cbtype: &CBType) -> Option<Rc<RefCell<CBEntry>>> {
 //  self.last_entries.get(cbtype).map(|x| x.cbentry.clone())
 // }

 pub(crate) fn toggle_clipboards(&mut self) {
  let primary_content: Option<Rc<RefCell<CBEntry>>> = self
   .last_entries
   .get(&CBType::Primary)
   .map(|x| Rc::clone(&x.cbentry));
  let clipboard_content: Option<Rc<RefCell<CBEntry>>> = self
   .last_entries
   .get(&CBType::Clipboard)
   .map(|x| Rc::clone(&x.cbentry));

  let (primary_content, clipboard_content) = match (primary_content, clipboard_content) {
   (Some(pc), Some(cc)) => (pc, cc),
   _ => {
    return;
   }
  };

  if primary_content.borrow().get_data() == clipboard_content.borrow().get_data() {
   return;
  }

  {
   primary_content
    .borrow_mut()
    .swap_data(&mut clipboard_content.borrow_mut());
   // std::mem::swap(&mut primary_content.borrow_mut().data, &mut clipboard_content.borrow_mut().data);
   // std::mem::swap(&mut primary_content.borrow_mut().text, &mut clipboard_content.borrow_mut().text);

   // TODO : crw_write_echofree(&) // avoid clone
   self.cfmap[&CBType::Primary]
    .crw
    .crw_write_echofree(primary_content.borrow().get_data().clone());
   self.cfmap[&CBType::Clipboard]
    .crw
    .crw_write_echofree(clipboard_content.borrow().get_data().clone());
  }
 }

 pub(crate) fn remove_by_seq(&mut self, seq: usize) -> Option<AppendedCBEntry> {
  if let Some(pos) = self.cbentries.iter().position(|e| e.seq == seq) {
   self.cbentries.remove(pos)
  } else {
   None
  }
 }
}

#[cfg(test)]
mod tests {
 use super::ClipboardReaderWriter;
 use std::{
  sync::Mutex,
  thread,
  time::{Duration, Instant},
 };

 #[ignore]
 #[test]
 fn test_mutex() {
  let _start = Instant::now();
  let handle = thread::spawn(|| {
   let m = Mutex::new("");
   let x = m.lock();
   let y = m.lock();
   drop(x);
   drop(y);
  });

  let _ = handle.join();
 }

 #[test]
 fn test_001() {
  let sleep_msecs = 0;
  let cbrw_s = ClipboardReaderWriter::from_cbtype(&super::CBType::Primary).unwrap();
  cbrw_s.crw_write_echofree("abc".into());
  // cbrw_s.crw_write("abc".into());
  let x = cbrw_s.crw_read();
  assert_eq!(None, x);
  std::thread::sleep(Duration::from_millis(sleep_msecs));
  let x = cbrw_s.crw_read();
  assert_eq!(None, x);
  std::thread::sleep(Duration::from_millis(sleep_msecs));
  let x = cbrw_s.crw_read();
  assert_eq!(None, x);

  cbrw_s.crw_write("def".into());

  let x = cbrw_s.crw_read();
  assert_eq!(Some("def".into()), x);
  std::thread::sleep(Duration::from_millis(sleep_msecs));
  let x = cbrw_s.crw_read();
  assert_eq!(Some("def".into()), x);
  std::thread::sleep(Duration::from_millis(sleep_msecs));
  let x = cbrw_s.crw_read();
  assert_eq!(Some("def".into()), x);

  // assert_eq!("abc", x.unwrap());
 }
}
