use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
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
   panic!()
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
}

impl ClipboardReaderWriter {

 pub(crate) fn from_cbtype(cbtype: &CBType) -> Result<Self, CbsError> {
  let cb = Clipboard::new()?;
  Ok(Self {
   cb,
   atom: cbtype.get_atom(),
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

 pub fn read(&self) -> Option<String> {
  let selection = self.atom;

  match self
   .cb
   .load(selection, CB_ATOMS.utf8_string, CB_ATOMS.property, Duration::from_secs(3))
  {
   Ok(selection_u8) => Some(String::from_utf8_lossy(selection_u8.as_slice()).into()),
   Err(_) => None,
  }
 }

 pub fn write(&self, s: String) -> bool {
  let value = s.as_bytes();
  let selection = self.atom;

  self
   .cb
   .store(selection, CB_ATOMS.utf8_string, value)
   .map_or_else(|_| false, |_| true)
 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CBEntry {
 // see old Entry from entries.rs
 pub cbtype: CBType,
 pub timestamp: MyTime,
 pub text: String,
}

impl CBEntry {
 pub fn get_date_time(&self) -> String {
  let ret = format!("{}", self.timestamp);
  // 2025-02-24 20:25:40+01:00
  ret[0..19].into() // 2025-02-24 20:25:40
 }
}
pub(crate) struct ClipboardFixation {
 pub crw: ClipboardReaderWriter,
 pub fixation: Option<Rc<CBEntry>>,
}

impl ClipboardFixation {

 fn from_cbtype(cbtype: &CBType) -> Result<Self, CbsError> {
  Ok(Self {
   crw: ClipboardReaderWriter::from_cbtype(cbtype)?,
   fixation: None,
  })
 }

 fn restore(&self) {
  if let Some(v) = &self.fixation {
   self.crw.write(v.text.clone());
  }
 }
}

#[derive(Clone)]
pub struct AppendedCBEntry {
 pub appended: bool,
 pub line_count: usize,
 pub cbentry: Rc<CBEntry>,
}

/** managed clipboards by [crate::libmain::ClipboardThread] */
pub struct Clipboards {
 // pub hm: HashMap<String, ClipboardSelectionList>,
 // pub crw: ClipboardReaderWriter,
 pub cbentries: VecDeque<AppendedCBEntry>,
 // NOTE : no weak pointer here, Optional<Rc> is better,
 // even if entry disappears from the list (currently not possible but maybe later)
 // it can still be selected
 // pub fixation: HashMap< String, Option<Rc<CBEntry>>>,
 // pub cfmap: HashMap<&'static str, ClipboardFixation>, // macht probleme beim indexieren
 // pub cfmap: HashMap<String, ClipboardFixation>,
 pub(crate) cfmap: HashMap<CBType, ClipboardFixation>,
}

pub fn atom_to_string(atom: u32) -> String {
 match atom {
  x if x == CB_ATOMS.primary => "p",
  2 => "s",
  x if x == CB_ATOMS.clipboard => "c",
  _ => panic!(""),
 }
 .to_string()
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
   cfmap,
  }
 }

 pub(crate) fn insert(&mut self, cbtype: &CBType, string: Option<String>) {
  if let Some(s) = string {
   let mut insert: bool = true;

   {
    // let cf: &ClipboardFixation = &self.cfmap[AsRef::<String>::as_ref(&atom_string)];
    let cf: &ClipboardFixation = &self.cfmap[&cbtype];

    trace!("fixation : {:?}", cf.fixation);

    if let Some(fixation) = &cf.fixation {
     if fixation.text == s {
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

   if insert {
    let now = MyTime::now();
    if let Some(last) = self.cbentries.front() {
     let last_time = &last.cbentry.timestamp;
     let span = now.timestamp - last_time.timestamp;
     // TODO : configurable milliseconds
     if cbtype == &last.cbentry.cbtype && span < TimeDelta::milliseconds(300) {
      self.cbentries.pop_front();
     }
    }
    self.cbentries.push_front(AppendedCBEntry {
     appended: false,
     line_count: s.lines().count(),
     cbentry: Rc::new(CBEntry {
      cbtype: cbtype.clone(),
      timestamp: now,
      text: s.clone(),
     }), // (now, s.clone())
    });
   }
  }
 }

 pub fn get_entries(&self) -> &VecDeque<AppendedCBEntry> {
  &self.cbentries
 }

 pub(crate) fn append_ndjson(&mut self, append_ndjson_filename: &str) {
  // panic!("append_ndjson_filename {}", append_ndjson_filename);
  let mut fd = OpenOptions::new()
   .create(true)
   .append(true)
   .open(Path::new(append_ndjson_filename))
   .unwrap();

  let now = MyTime::now();

  for cbentry in &mut self.cbentries {
   if cbentry.appended {
    break;
   } else {
    // Serialize
    let span = now.timestamp - cbentry.cbentry.timestamp.timestamp;
    if span > TimeDelta::milliseconds(300) {
     write!(fd, "{}\n", serde_json::to_string(&*cbentry.cbentry).unwrap()).unwrap();
     cbentry.appended = true;
    }
   }
  }
 }

 pub fn is_fixated(&self, cbentry: &Rc<CBEntry>) -> bool {
  self
   .cfmap
   .iter()
   .filter(|x| match &x.1.fixation {
    Some(f) => Rc::<CBEntry>::ptr_eq(&f, cbentry),
    None => false,
   })
   .count()
   != 0
 }

 // pub fn is_fixated_atom_text(&self, atom_string: &str, text: &str) -> bool {
 //  let cf: &ClipboardFixation = &self.cfmap[atom_string];

 //  match &cf.fixation {
 //   Some(f) => f.string == text,
 //   None => false,
 //  }
 // }

 // pub fn get_fixation_atom_text(&self, atom_string: &str) -> Option<&str> {
 //  let cf: &ClipboardFixation = &self.cfmap[atom_string];

 //  match &cf.fixation {
 //   Some(f) => Some(&f.string),
 //   None => None,
 //  }
 // }

 pub(crate) fn toggle_selection(&mut self, cbentry: &Rc<CBEntry>) {
  let cf = &mut self.cfmap.get_mut(&cbentry.cbtype).unwrap();

  trace!("toggle_selection");

  let insert = match &cf.fixation {
   Some(f) => !Rc::<CBEntry>::ptr_eq(&f, cbentry),
   None => true,
  };

  trace!("toggle_selection : insert : {insert}");

  if insert {
   cf.fixation = Some(Rc::clone(cbentry));
   cf.restore();
  } else {
   cf.fixation = None
  }
 }
}
