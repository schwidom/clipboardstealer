use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use chrono::TimeDelta;
use x11_clipboard as x11;
// use x11_clipboard::error::Error as X11Error;

use x11::Atoms;

use x11::Clipboard;

// use x11::xcb::Atom;
use x11::Atom;

use crate::libmain::MyError;
use crate::tools::cb_get_atoms;
use crate::tools::MyTime;

/** holds the clipboard information per clipboard type */

pub struct ClipboardSelectionList {
 crw: ClipboardReaderWriter,
 pub captured_from_clipboard: Vec<(MyTime, String)>,
 pub current_selection: Option<usize>, // TODO
 last_pushed_string: Option<(MyTime, String)>,
}

/** referred to ClipboardSelectionList */
#[derive(Clone)]
pub struct ListChanged(pub bool);

impl ClipboardSelectionList {
 pub fn new(atom: Atom) -> Self {
  Self {
   crw: ClipboardReaderWriter::new(atom),
   captured_from_clipboard: vec![],
   current_selection: None,
   last_pushed_string: None,
  }
 }

 pub fn get_current_selection(&self) -> (MyTime, String) {
  match self.current_selection {
   None => (MyTime::unix_epoch(), "".into()), // TODO
   Some(idx) => self.captured_from_clipboard.get(idx).unwrap().clone(),
  }
 }

 pub fn process_clipboard_contents(&mut self) -> ListChanged {
  let default_ret = ListChanged(false);
  let s = &self.crw.read();
  if s.is_none() {
   return ListChanged(false);
  }
  let s = s.clone().unwrap();

  // println!("{s}");
  // happens in 2 cases: by selection or by the selection reset
  if self.get_current_selection().1 == s {
   // TODO
   return default_ret;
  }

  let last_pushed_string = self.last_pushed_string.clone();

  if last_pushed_string.is_none() {
   self.last_pushed_string = Some((MyTime::now(), s.into()));
   return default_ret;
  }

  if last_pushed_string.clone().unwrap().1 != s {
   self.last_pushed_string = Some((MyTime::now(), s.into()));
   return default_ret;
  } else if last_pushed_string.unwrap().0.elapsed() > TimeDelta::try_seconds(1).unwrap() {
   let insert = match self.captured_from_clipboard.last() {
    None => true,
    Some(last_string) => last_string.1 != s,
   };
   if insert {
    self
     .captured_from_clipboard
     .push((MyTime::now(), s.clone()));
   }
   if self.crw.write(self.get_current_selection().1) {
    return ListChanged(insert);
   } else {
    return default_ret;
   }
  }

  return default_ret;
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
 pub primary: Arc<Mutex<ClipboardSelectionList>>,
 pub secondary: Arc<Mutex<ClipboardSelectionList>>,
 pub clipboard: Arc<Mutex<ClipboardSelectionList>>,
}

impl Clipboards {
 pub fn new() -> Self {
  let cb_atoms = cb_get_atoms();
  Self {
   // shift einf / middle mouse
   primary: Arc::new(Mutex::new(ClipboardSelectionList::new(cb_atoms.primary))),

   // echo 123 | xclip -i -selection primary
   // see /usr/include/X11/Xatom.h : XA_SECONDARY
   secondary: Arc::new(Mutex::new(ClipboardSelectionList::new(2))),
   // ctrl-c/ctrl-v
   clipboard: Arc::new(Mutex::new(ClipboardSelectionList::new(cb_atoms.clipboard))),
  }
 }
}
