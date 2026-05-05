use super::NextTsp;
use crate::linuxeditor;

 use std::io::{Read, Write}; // write_all

 use crate::event::MyEvent;

 use crate::libmain::StatusSeverity;

 use std::fs::OpenOptions;

 use crate::libmain::AppStateReceiverData;

 use ratatui::DefaultTerminal;

 use super::ScreenPainter;

 use std::fs::File;

 use crate::clipboards::AcbeId;

 use std::path::PathBuf;

 use mktemp::Temp;

 use crate::config::Config;

 pub(crate) struct ScreenEditorPage {
  pub(crate) config: &'static Config,
  pub(crate) tmpfile: Temp,
  pub(crate) tmpfile_path: PathBuf,
  pub(crate) edited: bool,
  pub(crate) entry_id: AcbeId,
 }

 impl ScreenEditorPage {
  pub(crate) fn new(
   config: &'static Config,
   text: String,
   entry_id: AcbeId,
  ) -> Result<Self, String> {
   let tmpfile = Temp::new_file().map_err(|e| format!("Failed to create temp file: {}", e))?;
   let tmpfile_path = tmpfile.to_path_buf();
   let mut fs = File::create(&tmpfile).map_err(|e| format!("Failed to create temp file: {}", e))?;
   fs
.write_all(text.as_bytes())
.map_err(|e| format!("Failed to write to temp file: {}", e))?;

   Ok(Self {
config,
tmpfile,
tmpfile_path,
edited: false,
entry_id,
   })
  }
 }

 impl ScreenPainter for ScreenEditorPage {
  fn paint(&mut self, _terminal: &mut DefaultTerminal, _assd: &mut AppStateReceiverData) {
   unreachable!("paint() is not used in editor page - use paint_without_terminal()");
  }

  fn paint_without_terminal(&mut self, assd: &mut AppStateReceiverData) {
   if !self.edited {
self.edited = true;

// suspend_raw_mode();

if self.config.editor {
 linuxeditor::edit_file(&self.tmpfile_path).unwrap();
} else {
 edit::edit_file(&self.tmpfile_path).ok();
}
// edit::edit_file(&self.tmpfile_path).unwrap();
// restore_raw_mode();

match OpenOptions::new().read(true).open(&self.tmpfile_path) {
 Ok(mut fh) => {
  let mut buf = Vec::new();
  match fh.read_to_end(&mut buf) {
   Ok(_) => {
    let entry_id = self.entry_id;
    //  if let Some(entry) = assd.cbs.get_cbentries().iter_mut().find(|e| e.id == entry_id) {
    //   entry.cbentry.borrow_mut().set_data(&buf);
    //  }
    if let Some(entry) = assd.cbs.get_entry_by_id(entry_id) {
     entry.borrow_mut().set_data(&buf);
    }
   }
   Err(err) => assd
    .statusline_heap
    .push(StatusSeverity::Error, err.to_string()),
  };
 }
 Err(err) => assd
  .statusline_heap
  .push(StatusSeverity::Error, err.to_string()),
};
   }
  }

  fn handle_event(&mut self, _evt: &MyEvent, _assd: &mut AppStateReceiverData) -> NextTsp {
   /*
   let edited_text = self.text.clone();
   let idx = self.index;

   if let Some(entry) = assd.cbs.cbentries.get_mut(idx) {
    let mut cbentry = (*entry.cbentry).clone();
    cbentry.text = edited_text;
    entry.cbentry = Rc::new(cbentry);
   }
   */

   // this Tsp gets automatically removed as soon as
   NextTsp::NoNextTsp
  }

  fn is_external_program(&self) -> bool {
   true
  }
 }
