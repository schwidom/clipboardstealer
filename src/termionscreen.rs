#![allow(dead_code)]
#![allow(unused)]

use std::cell::RefCell;
use std::cmp::min;
use std::collections::VecDeque;

use std::fs::{self, File};
use std::io::{stdout, Stdout, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use std::rc::Rc;

use crate::clipboards::{AppendedCBEntry, CBEntry};
use crate::config::{self, Config};
use crate::constants::{HELP_FIRST_PAGE, HELP_QX};
use crate::event::MyEvent;
use crate::layout::Layout;
use crate::layout_ratatui::{PagerLayout, PagerLayoutBase, PagerLayoutLR, PagerLayoutTB};
// use crate::libmain::SyncStuff;
use crate::libmain::AppStateReceiverData;
use crate::linuxeditor;
use crate::pager::Pager;
use crate::scroller::Scroller;
use crate::tools::tabfix;

use mktemp::Temp;
use ratatui::crossterm::terminal;
use ratatui::layout::{Alignment, Margin, Position, Rect};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::Stylize;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap};
use ratatui::{DefaultTerminal, Terminal};
use termion::event::{Event, Key};
use termion::{self, scroll};

use tracing::{event, info, span, trace, Instrument, Level};

use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr; // extends &str by width, width_cjk // extends char by width, width_cjk

use regex::Match;
use regex::Regex;

// TODO : into tools.rs
fn truncate_before_or_at_display_width(text: &str, width: usize) -> &str {
 // let mut current_width: usize = 0;
 let last_idx = text
  .char_indices()
  .map(|(pos, char)| {
   let w = pos + char.len_utf8();
   let w_cjk = text[0..pos].width_cjk() + char.width_cjk().unwrap_or(0);
   (w, w_cjk)
  })
  .take_while(|(w, w_cjk)| *w_cjk <= width)
  .map(|(w, _w_cjk)| w)
  .last()
  .unwrap_or(0);

 // println!("last_idx : {:?}", last_idx);
 &text[0..last_idx]
}

#[cfg(test)]
mod tests {
 use unicode_width::UnicodeWidthChar; // extends char by width, width_cjk
 use unicode_width::UnicodeWidthStr; // extends &str by width, width_cjk

 use super::truncate_before_or_at_display_width;

 #[test]
 fn test_001() {
  {
   let x = "🧑";
   assert_eq!(4, x.len());
   assert_eq!(2, x.width());
   assert_eq!(2, x.width_cjk()); // displayed width
   let y = x.chars().collect::<Vec<_>>();
   assert_eq!(1, y.len());
   assert_eq!(x, &x[0..4]);
   assert_eq!(Some(x), x.get(0..4));
  }
 }
 #[test]
 fn test_002() {
  assert_eq!("", truncate_before_or_at_display_width("", 10));
  assert_eq!("", truncate_before_or_at_display_width("", 0));
  assert_eq!("", truncate_before_or_at_display_width("a", 0));
  assert_eq!("a", truncate_before_or_at_display_width("a", 1));
  assert_eq!("a", truncate_before_or_at_display_width("a", 2));
  assert_eq!("", truncate_before_or_at_display_width("🧑", 0));
  assert_eq!("", truncate_before_or_at_display_width("🧑", 1));
  assert_eq!("🧑", truncate_before_or_at_display_width("🧑", 2));
  assert_eq!("🧑", truncate_before_or_at_display_width("🧑", 3));
 }
}

// TODO : rename  to trim_text_to_rect
fn trim_text_to_rect_with(text: &str, rect: ratatui::layout::Rect) -> String {
 // trace!("trim_text_to_rect_with: rect {:?}", rect);
 // trace!("trim_text_to_rect_with: text {:?}", text);

 let max_width = rect.width as usize;
 let max_height = rect.height as usize;

 let wrapped = text.split("\n");

 let trimmed = wrapped
  .into_iter()
  .take(max_height)
  .map(|x| truncate_before_or_at_display_width(x, max_width))
  .collect::<Vec<_>>();

 let ret = trimmed.join("\n");
 // trace!("trim_text_to_rect_with: ret {:?}", ret);
 ret
}

struct RatatuiVariables {
 pl: Box<dyn PagerLayout>,
}

impl RatatuiVariables {
 fn new<T: PagerLayout + 'static>(terminal: &mut DefaultTerminal) -> Self {
  let pl = Box::new(T::new(&terminal.get_frame()));
  Self { pl }
 }
}

/// the TwoScreenDefaultWidget paints in the areas of the
/// rv.pl (RatatuiVariables . PagerLayout)

struct TwoScreenDefaultWidget<'a> {
 helpline: &'a str,
 main_title: &'a str,
 second_title: &'a str,
 rv: &'a RatatuiVariables,
 all_lines: &'a str,
 all_lines2: &'a str,
 wrapped1: bool,
 wrapped2: bool,
 regex_edit_mode: Option<String>,
 regex_edit_mode_state: String,
 regex_count: usize,
 line_count: usize,
 line_count2: Option<usize>,
 statusline_vector: Rc<RefCell<Vec<String>>>,
}

impl<'a> Widget for TwoScreenDefaultWidget<'a> {
 fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
 where
  Self: Sized,
 {
  let regex_count_indicator =
   if 0 != self.regex_count { &format!(" r({})", self.regex_count) } else { "" };
  let title = self.main_title.to_string()
   + &format!(" l({})", self.line_count)
   + if self.wrapped1 { " (w)" } else { "" }
   + regex_count_indicator;

  let block = Block::bordered()
   .title(title)
   .title_alignment(Alignment::Left)
   // .style(Style::new().fg(Color::Blue))
   .border_type(BorderType::Rounded);

  // let rect1 = self.rv.pl.get_main_area().inner(Margin::new(0, 0));
  let rect1 = *self.rv.pl.get_main_area();
  let safe_area = rect1.intersection(buf.area); // avoids crash

  let all_lines = self.all_lines;
  let all_lines = tabfix(&all_lines);
  let all_lines =
   if self.wrapped1 { all_lines } else { trim_text_to_rect_with(self.all_lines, safe_area) };

  // trace!( "TwoScreenDefaultWidget all_lines : {}", all_lines);

  let paragraph = Paragraph::new(all_lines).block(block).left_aligned();

  // weue806j1y
  let paragraph = if !self.wrapped1 { paragraph } else { paragraph.wrap(Wrap { trim: false }) };

  Text::raw(self.helpline).render(*self.rv.pl.get_title_area(), buf);
  // Text::raw(self.all_lines).render(self.rv.pl.main_area.inner(Margin::new(0, 1)), buf);
  // block.render(self.rv.pl.main_area.inner(Margin::new(0, 1)), buf);
  // paragraph.render(rect1, buf);
  // Clear.render(safe_area, buf); // doesn't fix the tab problem
  paragraph.render(safe_area, buf);

  if let Some(sma) = self.rv.pl.get_second_main_area() {
   let title2 = self.second_title.to_string()
    + &self
     .line_count2
     .map_or("".to_string(), |x| format!(" l({})", x))
    + if self.wrapped2 { " (w)" } else { "" };

   // &format!(" l({})", self.line_count2);

   let block2 = Block::bordered()
    .title(title2)
    .title_alignment(Alignment::Left)
    .border_type(BorderType::Rounded);

   // let rect2 = sma.inner(Margin::new(0, 1));
   let rect2 = *sma;
   let safe_area2 = rect2.intersection(buf.area); // avoids crash

   let all_lines2 = self.all_lines2;
   let all_lines2 = tabfix(&all_lines2);
   let all_lines2 = if self.wrapped2 {
    self.all_lines2.to_owned()
   } else {
    trim_text_to_rect_with(self.all_lines2, safe_area2)
   };

   let paragraph2 = Paragraph::new(all_lines2).block(block2).left_aligned();

   // weue806j1y
   let paragraph2 = if !self.wrapped2 { paragraph2 } else { paragraph2.wrap(Wrap { trim: false }) };

   // Clear.render(safe_area2, buf); // doesn't fix the tab problem
   paragraph2.render(safe_area2, buf);
  }
  // Paragraph::new("statusline").render( self.rv.pl.get_status_area().intersection(buf.area), buf);
  let statusline = self.statusline_vector.borrow();
  if let Some(regex_edit_mode) = &self.regex_edit_mode {
   Paragraph::new("/".to_string() + regex_edit_mode + &self.regex_edit_mode_state)
    .render(self.rv.pl.get_status_area().intersection(buf.area), buf);
  } else if let Some(status_message) = statusline.first() {
   Paragraph::new(status_message.clone())
    .render(self.rv.pl.get_status_area().intersection(buf.area), buf);
  }
 }
}

/// Tsp ... TermionScreenPainter
///
/// returned from TermionScreenPainter::handle_event determines the next TermionScreenPainter
///
/// replaces the current Tsp or lays it onto the stack

pub enum NextTsp {
 NoNextTsp,
 Replace(Rc<RefCell<dyn TermionScreenPainter>>),
 Stack(Rc<RefCell<dyn TermionScreenPainter>>),
 Quit,
 PopThis,
 IgnoreBasicEvents,
}

pub trait TermionScreenPainter {
 // fn new(config: &'static Config) -> Self
 // where
 //  Self: Sized;

 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData);

 fn paint_without_terminal(&mut self, assd: &mut AppStateReceiverData) {
  panic!("only used if overridden");
 }

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp;

 /// a dialog is sticky if you cannot quit (q) or exit (x) from it
 /// or put another dialog on top of the dialogstack
 ///
 /// it is currently used for the exit dialog
 fn is_sticky_dialog(&self) -> bool {
  false
 }

 /// that means: this TermionScreenPainter does not handle events
 ///
 /// if the paint method is done the previous screen is visible again
 ///
 /// before and after the page the raw mode gets disabled and reenabled again
 ///
 /// before and after the page the controlling events gets discarded
 ///
 /// events are kept : mouse, shift, clipboards are still getting read out
 ///
 ///
 fn is_external_program(&self) -> bool {
  false
 }
}

pub struct TermionScreenStatusBarDialogYN {
 config: &'static Config,
 /// tsp_before is intended to allow the display of the previous dialog in a frozen state while the exit dialog is in effect
 ///
 /// currently is it not used
 tsp_before: Rc<RefCell<dyn TermionScreenPainter>>,
 question: String,
}

impl TermionScreenStatusBarDialogYN {
 pub fn new(
  config: &'static Config,
  tsp_before: Rc<RefCell<dyn TermionScreenPainter>>,
  question: String,
 ) -> Self {
  Self {
   config,
   tsp_before,
   question,
  }
 }
}

impl TermionScreenPainter for TermionScreenStatusBarDialogYN {
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let rv = &RatatuiVariables::new::<PagerLayoutBase>(terminal);

  // if let Some(rc) = &self.tsp_before {
  //  // rc.borrow_mut().handle_event(&MyEvent::Tick, assd);
  //  rc.borrow_mut().paint(terminal, assd);
  // }

  //  writes in the correct area but overwrites the upper part
  terminal.draw(|frame| {
   frame.render_widget(
    Paragraph::new(self.question.clone()),
    rv.pl.get_status_area().intersection(frame.area()),
   )
  });
 }

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
  // if let Some(rc) = &self.tsp_before {
  //  // rc.borrow_mut().handle_event(evt, assd); // TODO : filter events
  // }

  match evt {
   MyEvent::Termion(Event::Key(Key::Char('y'))) => NextTsp::Quit,
   MyEvent::Termion(Event::Key(Key::Char('n'))) => NextTsp::PopThis,
   _ => NextTsp::NoNextTsp,
  }
 }

 fn is_sticky_dialog(&self) -> bool {
  true
 }
}

pub struct TermionScreenFirstPage {
 config: &'static Config,
 scroller: Scroller,
 layout: Layout,
 flipstate: u8,
 wrapped: bool,
 regex_edit_mode: Option<String>,
 regex_edit_mode_state: String,
 regex_edit_mode_last_working: Option<Regex>,
 regex: Vec<Regex>,
 regex_filtered_cbs_entries: VecDeque<AppendedCBEntry>,
}

// TODO : mode in the vicinity of first_page() definition (maybe inside)
impl TermionScreenFirstPage {
 pub fn new(config: &'static Config, statusline_vector: Rc<RefCell<Vec<String>>>) -> Self {
  Self {
   config,
   scroller: Scroller::new(),
   layout: Layout::new(),
   flipstate: 1,
   wrapped: false,
   regex_edit_mode: None,
   regex_edit_mode_state: "".to_string(),
   regex_edit_mode_last_working: None,
   regex: vec![],
   regex_filtered_cbs_entries: VecDeque::new(),
  }
 }

 fn flipstate_next(&mut self) {
  self.flipstate = (self.flipstate + 1) % 3;
 }
 fn flipstate_prev(&mut self) {
  self.flipstate = (self.flipstate + 2) % 3;
 }
}

impl TermionScreenPainter for TermionScreenFirstPage {
 /// the paint method opens a TwoScreenDefaultWidget which is later painted
 /// by the terminal.draw method
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let scroller = &mut self.scroller;
  let layout = &mut self.layout;

  let cbs = &mut assd.cbs;

  let rv = if self.flipstate == 0 {
   &RatatuiVariables::new::<PagerLayoutBase>(terminal)
  } else if self.flipstate == 1 {
   &RatatuiVariables::new::<PagerLayoutTB>(terminal)
  } else {
   &RatatuiVariables::new::<PagerLayoutLR>(terminal)
  };

  {
   let (width, height) = termion::terminal_size().unwrap();
   layout.set_width_height(width, height);

   let mut lines = vec![];

   {
    let entries = cbs.get_entries();

    // gtewxxi8oh
    let entries = entries
     .iter()
     .filter(|line| {
      let mut res = true;
      // TODO : regex_edit_mode_last_working
      let mut r = self.regex.clone();
      r.extend(self.regex_edit_mode_last_working.iter().cloned());
      for r in r {
       if !r.is_match(&line.cbentry.text) {
        res = false;
        break;
       }
      }
      res
     })
     .collect::<VecDeque<_>>();

    self.regex_filtered_cbs_entries = entries
     .iter()
     .map(|x| (**x).clone())
     .collect::<VecDeque<AppendedCBEntry>>();

    let mut selected_string = &String::default();
    let mut line_count2 = None;

    if self.config.debug {
     trace!("scroller.set_content_length(entries.len()) : {}", entries.len());
    }
    scroller.set_content_length(entries.len());
    // scroller.set_windowlength(height + 1 - layout.get_current_line());
    scroller.set_windowlength(rv.pl.get_main_area().inner(Margin::new(0, 1)).height as usize);

    let numbers_width = (entries.len() as f64).log10().ceil() as usize;

    if self.config.debug {
     trace!("scroller.get_safe_windowrange() : {:?}", scroller.get_safe_windowrange());
    }

    for (idx, entry) in entries.range(scroller.get_safe_windowrange()).enumerate() {
     let cbentry = &entry.cbentry;
     let is_cursor = match scroller.get_cursor() {
      None => false,
      Some(value) => idx == value,
     };

     let cursor_star = if is_cursor { ">" } else { " " };

     // let is_selected = entry.is_selected(cbs);
     let is_selected = cbs.is_fixated(cbentry);

     let selection_star = if is_selected { "*" } else { " " };

     if is_cursor {
      selected_string = &cbentry.text;
      line_count2.insert(entry.line_count);
     }

     {
      let s002 = format!(
       "{} {} {:width$} {} {} : {}",
       cursor_star,
       selection_star,
       idx + scroller.get_windowposition(), // mqbojcmkot
       cbentry.cbtype.get_info(),
       cbentry.get_date_time(),
       cbentry.text,
       width = numbers_width,
      );
      lines.push(layout.fixline(&s002));
     }
    }

    let all_lines = lines.join("\n");

    let sw = TwoScreenDefaultWidget {
     helpline: HELP_FIRST_PAGE,
     main_title: "entry list",
     second_title: "selected content",
     rv,
     // tsfp: &self,
     all_lines: &all_lines,
     all_lines2: &selected_string,
     wrapped1: false,
     wrapped2: self.wrapped,
     regex_edit_mode: self.regex_edit_mode.clone(),
     regex_edit_mode_state: self.regex_edit_mode_state.clone(),
     regex_count: self.regex.len() + self.regex_edit_mode.is_some() as usize,
     line_count: entries.len(),
     // line_count2: selected_string.lines().count(),
     line_count2,
     statusline_vector: Rc::clone(&assd.statusline_vector),
    };

    terminal.draw(|frame| frame.render_widget(sw, frame.area()));
   }
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
  let mut cbs = &mut assd.cbs;

  if let Some(mut regex_edit_mode) = self.regex_edit_mode.clone() {
   let regex = Regex::new(&regex_edit_mode);
   match regex {
    Ok(regex) => {
     self.regex_edit_mode_state = "".to_string();
     self.regex_edit_mode_last_working = Some(regex);
    }
    Err(_) => self.regex_edit_mode_state = "  < buggy regex".to_string(),
   }

   match evt {
    MyEvent::Termion(Event::Key(Key::Esc)) => {
     self.regex_edit_mode = None;
     self.regex_edit_mode_last_working = None;
    }
    MyEvent::Termion(Event::Key(Key::Char('\n'))) => {
     if let Ok(regex) = Regex::new(&regex_edit_mode) {
      self.regex_edit_mode = None;
      self.regex_edit_mode_last_working = None;
      self.regex.push(regex);
     }
    }
    MyEvent::Termion(Event::Key(Key::Backspace)) => {
     regex_edit_mode.pop();
     self.regex_edit_mode.insert(regex_edit_mode);
    }
    MyEvent::Termion(Event::Key(Key::Char(char))) => {
     regex_edit_mode.push(*char);
     self.regex_edit_mode.insert(regex_edit_mode);
    }
    _ => {}
   }
   return NextTsp::IgnoreBasicEvents;
  } else {
   match evt {
    MyEvent::Termion(Event::Key(Key::Esc)) => {
     self.regex.pop();
    }
    //  MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
    MyEvent::Termion(Event::Key(Key::Char('h'))) => {
     return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenViewPage::new(
      self.config,
      "help".to_string(),
      config::USAGE.to_string(),
     ))));
    }
    MyEvent::Termion(Event::Key(Key::Char('f'))) => {
     self.flipstate_next();
    }
    MyEvent::Termion(Event::Key(Key::Char('F'))) => {
     self.flipstate_prev();
    }
    MyEvent::Termion(Event::Key(Key::Char('s'))) => {
     if let Some(cursor) = self.scroller.get_cursor_in_array() {
      let entries = &self.regex_filtered_cbs_entries;
      let entry = &entries[cursor].cbentry.clone(); // NOTE: the clone can maybe avoided when I put this logic into cbs
                                                    // entry.toggle_selection(&mut cbs);
      cbs.toggle_selection(entry);
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('v'))) => {
     if let Some(cursor) = self.scroller.get_cursor_in_array() {
      let entries = &self.regex_filtered_cbs_entries;
      let entry = &entries[cursor];
      return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenViewPage::new(
       self.config,
       "view entry".to_string(),
       entry.cbentry.text.clone(),
      ))));
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('e'))) => {
     if let Some(cursor) = self.scroller.get_cursor_in_array() {
      let entries = &self.regex_filtered_cbs_entries;
      let entry = &entries[cursor];

      return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenEditorPage::new(
       self.config,
       entry.cbentry.text.clone(),
       cursor,
      ))));
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('w'))) => {
     self.wrapped = !self.wrapped;
    }
    MyEvent::Termion(Event::Key(Key::Char('/'))) => {
     self.regex_edit_mode = Some("".to_string());
    }
    _ => {
     // Pager::handle_event(&mut scroller, &evt);
     Pager::handle_event(&mut self.scroller, evt);
    }
   }
  }
  NextTsp::NoNextTsp
 }
}

pub struct TermionScreenEditorPage {
 config: &'static Config,
 tmpfile: Temp,
 tmpfile_path: PathBuf,
 edited: bool,
 index: usize,
}

impl TermionScreenEditorPage {
 pub fn new(config: &'static Config, text: String, index: usize) -> Self {
  let tmpfile = Temp::new_file().unwrap();
  let tmpfile_path = tmpfile.to_path_buf();
  let mut fs = File::create(&tmpfile).unwrap();
  fs.write_all(text.as_bytes()).unwrap();

  Self {
   config,
   tmpfile,
   tmpfile_path,
   edited: false,
   index,
  }
 }
}

impl TermionScreenPainter for TermionScreenEditorPage {
 fn paint(&mut self, _terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  panic!("not used here");
 }

 fn paint_without_terminal(&mut self, assd: &mut AppStateReceiverData) {
  if !self.edited {
   self.edited = true;

   // suspend_raw_mode();

   edit::edit_file(&self.tmpfile_path).ok();
   // edit::edit_file(&self.tmpfile_path).unwrap();
   // linuxeditor::edit_file( &self.tmpfile_path);
   // restore_raw_mode();

   if let Ok(new_text) = fs::read_to_string(&self.tmpfile_path) {
    let idx = self.index;

    if let Some(entry) = assd.cbs.cbentries.get_mut(idx) {
     // let mut cbentry = (*entry.cbentry).clone();
     // entry.cbentry = Rc::new(cbentry);
     entry.line_count = new_text.lines().count();
     // entry.cbentry).text = new_text.clone();
     entry.cbentry = Rc::new(CBEntry {
      text: new_text.clone(),
      ..(*entry.cbentry).clone()
     });
    }
   }
  }
 }

 fn handle_event(&mut self, _evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
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

pub struct TermionScreenViewPage {
 config: &'static Config,
 main_title: String,
 scroller: Scroller,
 layout: Layout,
 text: String,
 wrapped: bool,
}

impl TermionScreenViewPage {
 fn new(config: &'static Config, main_title: String, text: String) -> Self
 where
  Self: Sized,
 {
  Self {
   config,
   main_title,
   scroller: Scroller::new(),
   layout: Layout::new(),
   text,
   wrapped: false,
  }
 }
}

impl TermionScreenPainter for TermionScreenViewPage {
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let scroller = &mut self.scroller;
  let layout = &mut self.layout;

  let string_lines = self.text.lines().collect::<Vec<_>>();

  let mut rv = RatatuiVariables::new::<PagerLayoutBase>(terminal);

  {
   let (width, height) = termion::terminal_size().unwrap();
   layout.set_width_height(width, height);

   let mut lines = vec![];

   {
    scroller.set_content_length(string_lines.len());
    // scroller.set_windowlength(height + 1 - layout.get_current_line());
    scroller.set_windowlength(rv.pl.get_main_area().inner(Margin::new(0, 1)).height as usize);

    let numbers_width = (string_lines.len() as f64).log10().ceil() as usize;

    for (idx, entry) in string_lines[scroller.get_safe_windowrange()]
     .iter()
     .enumerate()
    {
     let is_cursor = match scroller.get_cursor() {
      None => false,
      Some(value) => idx == value,
     };

     let cursor_star = if is_cursor { ">" } else { " " };

     let s002 = format!(
      "{} {:width$} : {}",
      cursor_star,
      idx + scroller.get_windowposition(), // mqbojcmkot
      entry,
      width = numbers_width,
     );

     lines.push(if self.wrapped { s002.to_string() } else { layout.fixline(&s002) });

     // layout.print_line_cut(&s002);
    }
   }

   let all_lines = lines.join("\n");

   let sw = TwoScreenDefaultWidget {
    helpline: HELP_QX,
    main_title: &self.main_title,
    second_title: "unused",
    rv: &rv,
    // tsfp: &self,
    all_lines: &all_lines,
    all_lines2: "unused",
    wrapped1: self.wrapped,
    wrapped2: false,
    regex_edit_mode: None,
    regex_edit_mode_state: "".to_string(),
    regex_count: 0,
    line_count: string_lines.len(),
    line_count2: None,
    statusline_vector: Rc::clone(&assd.statusline_vector),
   };

   terminal.draw(|frame| frame.render_widget(sw, frame.area()));
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, _assd: &mut AppStateReceiverData) -> NextTsp {
  match evt {
   MyEvent::Termion(Event::Key(Key::Char('w'))) => {
    self.wrapped = !self.wrapped;
   }

   //  MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
   _ => {
    // Pager::handle_event(&mut scroller, &evt);
    Pager::handle_event(&mut self.scroller, evt);
   }
  }
  NextTsp::NoNextTsp
 }
}
