use std::collections::HashMap;
use std::time::Duration;

use chrono::TimeDelta;
use x11_clipboard as x11;
// use x11_clipboard::error::Error as X11Error;

use x11::Atoms;

use x11::Clipboard;

// use x11::xcb::Atom;
use x11::Atom;

use crate::entries::Entries;
use crate::entries::Entry;
// use crate::libmain::MyError;
use crate::tools::cb_get_atoms;
use crate::tools::MyTime;

/** holds the clipboard information per clipboard type */
pub struct ClipboardSelectionList {
 pub crw: ClipboardReaderWriter,
 pub captured_from_clipboard: Vec<(MyTime, String)>,
 pub current_selection: Option<usize>, // TODO
}

/** referred to ClipboardSelectionList */
#[derive(Clone, PartialEq)]
pub struct ListChanged(pub bool);

impl ClipboardSelectionList {
 pub fn new(atom: Atom) -> Self {
  Self {
   crw: ClipboardReaderWriter::new(atom),
   captured_from_clipboard: vec![],
   current_selection: None,
  }
 }

 pub fn get_current_selection(&self) -> (MyTime, String) {
  match self.current_selection {
   None => (MyTime::unix_epoch(), "".into()), // TODO
   Some(idx) => self.captured_from_clipboard.get(idx).unwrap().clone(),
  }
 }

 fn insert(&mut self, string: Option<String>) {
  if let Some(s) = string {
   let mut insert: bool = true;

   if let Some(selection_idx) = self.current_selection {
    let selection_string = &self.captured_from_clipboard[selection_idx].1;
    if selection_string == &s {
     insert = false;
    } else {
     self.crw.write(selection_string.clone());
    }
   }

   if insert {
    let now = MyTime::now();
    if let Some(last) = self.captured_from_clipboard.last() {
     let last_time = &last.0;
     let span = now.timestamp - last_time.timestamp;
     // TODO : configurable milliseconds
     if span < TimeDelta::milliseconds(300) {
      self.captured_from_clipboard.pop();
     }
    }
    self.captured_from_clipboard.push((now, s.clone()));
   }
  }
 }
}

// impl From<X11Error> for MyError {
//  fn from(value: X11Error) -> Self {
//   MyError::X11Clipboard(value)
//  }
// }

/** simplifies the reading / writing to a specific clipboard ( primary and clipboard) */
pub struct ClipboardReaderWriter {
 cb: Clipboard,
 atom: Atom,
 atoms: Atoms,
}

impl ClipboardReaderWriter {
 pub fn new(atom: Atom) -> Self {
  let cb = Clipboard::new().unwrap(); // TODO : in Struct auslagern
  let atoms = cb.setter.atoms.clone();

  Self { cb, atom, atoms }
 }

 pub fn atom(&self) -> Atom {
  self.atom
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

 pub fn read(&self) -> Option<String> {
  let cb_atoms = &self.atoms;
  let selection = self.atom;

  match self
   .cb
   .load(selection, cb_atoms.utf8_string, cb_atoms.property, Duration::from_secs(3))
  {
   Ok(selection_u8) => Some(String::from_utf8_lossy(selection_u8.as_slice()).into()),
   Err(_) => None,
  }

  // let selection_u8 = self
  //  .cb
  //  .load(selection, cb_atoms.utf8_string, cb_atoms.property, Duration::from_secs(3))
  //  .unwrap();
  // String::from_utf8_lossy(selection_u8.as_slice()).into()
 }

 pub fn write(&self, s: String) -> bool {
  let cb_atoms = &self.atoms;
  let value = s.as_bytes();
  let selection = self.atom;

  self
   .cb
   .store(selection, cb_atoms.utf8_string, value)
   .map_or_else(|_| false, |_| true)
 }
}

/** managed clipboards by [crate::libmain::ClipboardThread] */
pub struct Clipboards {
 pub hm: HashMap<String, ClipboardSelectionList>,
}

pub fn atom_to_string(atom: u32) -> String {
 let cb_atoms = cb_get_atoms();
 match atom {
  x if x == cb_atoms.primary => "p",
  2 => "s",
  x if x == cb_atoms.clipboard => "c",
  _ => panic!(""),
 }
 .to_string()
}

impl Clipboards {
 pub fn new() -> Self {
  let cb_atoms = cb_get_atoms();
  let mut hm = HashMap::new();
  // shift ins / middle mouse
  hm.insert("p".to_string(), ClipboardSelectionList::new(cb_atoms.primary));
  // echo 123 | xclip -i -selection primary
  // see /usr/include/X11/Xatom.h : XA_SECONDARY
  hm.insert("s".to_string(), ClipboardSelectionList::new(2));
  // ctrl-c/ctrl-v
  hm.insert("c".to_string(), ClipboardSelectionList::new(cb_atoms.clipboard));
  Self { hm }
 }

 pub(crate) fn insert(&mut self, atom: u32, string: Option<String>) {
  self
   .hm
   .get_mut(&atom_to_string(atom))
   .unwrap()
   .insert(string);
 }

 pub fn get_entries(&mut self) -> Vec<Entry> {
  let mut entries = vec![];
  for (name, cb) in &self.hm {
   entries.append(&mut Entries::from_csl(name, cb));
  }

  entries.sort_by(|x, y| y.timestamp.cmp(&x.timestamp));
  entries
 }
}
