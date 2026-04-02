// #![allow(dead_code)]
// #![allow(unused)]

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use std::rc::Rc;

use crate::clipboards::{cbentry::CBEntry, AppendedCBEntry, CBType};
use crate::config::{self, Config};
use crate::constants::{HELP_FIRST_PAGE, HELP_QX};
use crate::event::MyEvent;
use crate::layout::Layout;
use crate::layout_ratatui::{PagerLayout, PagerLayoutBase, PagerLayoutLR, PagerLayoutTB};
// use crate::libmain::SyncStuff;
use crate::libmain::{AppStateReceiverData, StatusMessage, StatusSeverity};
use crate::linuxeditor;
use crate::pager::Pager;
use crate::scroller::Scroller;
use crate::tools::{flatline, tabfix};

use enum_iterator::all;
use mktemp::Temp;
use ratatui::layout::{Alignment, Margin, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, BorderType, Paragraph, Widget, Wrap};
use ratatui::DefaultTerminal;
use termion::event::{Event, Key};
use termion::{self};

use tracing::trace;

use unicode_width::UnicodeWidthChar; // extends char by width, width_cjk
use unicode_width::UnicodeWidthStr; // extends &str by width, width_cjk

use regex::Regex;
use std::io::{Read, Write}; // write_all

// get_max_hoffset, get_max_hoffset_main, get_max_hoffset_second
fn get_max_width(lines: &[String]) -> usize {
 lines.iter().map(|l| l.width()).max().unwrap_or(0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ActiveArea {
 Main,
 Second,
}

// TODO : into tools.rs
fn truncate_before_or_at_display_width(text: &str, width: usize) -> &str {
 let last_idx = text
  .char_indices()
  .map(|(pos, char)| {
   let w1 = pos + char.len_utf8();
   let w2 = text[0..pos].width() + char.width().unwrap_or(0);
   (w1, w2)
  })
  .take_while(|(_, w2)| *w2 <= width)
  .map(|(w1, _w2)| w1)
  .last()
  .unwrap_or(0);
 &text[0..last_idx]
}

// see also apply_hoffset_and_trim
fn render_scroller_lines<T>(
 scroller: &mut Scroller,
 items: &[T],
 wrapped: bool,
 layout: &Layout,
 formatter: impl Fn(&str, usize, usize, &T) -> String,
) -> String {
 let lines = render_scroller_lines2(scroller, items, wrapped, layout, formatter);

 lines.join("\n")
}

fn render_scroller_lines2<T>(
 scroller: &mut Scroller,
 items: &[T],
 wrapped: bool,
 _layout: &Layout,
 formatter: impl Fn(&str, usize, usize, &T) -> String,
) -> Vec<String> {
 let numbers_width = (items.len() as f64).log10().ceil() as usize;
 let mut lines = vec![];

 for (idx, item) in items[scroller.get_safe_windowrange()].iter().enumerate() {
  let is_cursor = match scroller.get_cursor() {
   None => false,
   Some(value) => idx == value,
  };
  let cursor_star = if is_cursor { ">" } else { " " };

  let line = formatter(cursor_star, idx + scroller.get_windowposition(), numbers_width, item);
  // lines.push(if wrapped { line } else { layout.fixline(&line) });
  // TODO : hier bereits nicht printbare Zeichen durch punkte ersetzen und tabfix anwenden
  lines.push(if wrapped {
   line
  } else {
   // layout.fixline(&line)
   // truncate_before_or_at_display_width(&line, layout.width.unwrap_or(80) as usize).to_string()
   // einfach abschneiden ohne berücksichtigung der scroller.get
   // line alleine funktioniert // hack!
   line

   // ist besser, funktioniert aber auch nicht
   // apply_hoffset_and_trim_line(
   //  &line,
   //  Rect::new(0, 0, layout.width.unwrap_or(80), layout.height.unwrap_or(40)),
   //  scroller.get_hoffset(),
   // ).to_string()
  });

  // lines.push(if true { line } else { layout.fixline(&line) });
 }
 lines
}

#[cfg(test)]
mod tests {
 // use unicode_width::UnicodeWidthChar; // extends char by width, width_cjk
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
fn trim_text_to_rect_with(text: &str, rect: Rect) -> String {
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

 // trace!("trim_text_to_rect_with: ret {:?}", ret);
 trimmed.join("\n")
}

#[cfg(test)]
mod test {
 use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

 use super::apply_horizontal_offset;
 use super::apply_horizontal_offset_line;

 #[test]
 fn test_apply_horizontal_offset_basics() {
  assert_eq!(1, '1'.len_utf8());
  assert_eq!(1, '1'.width().unwrap_or(0));
  assert_eq!(None, '\n'.width()); // zoswgrlsbw ?
  assert_eq!(1, "1".width());
  assert_eq!(2, "1\n".width()); // zoswgrlsbw ?
  assert_eq!(3, "1\n2".width()); // zoswgrlsbw ?
 }

 #[test]
 fn test_apply_horizontal_offset() {
  assert_eq!("".to_string(), apply_horizontal_offset("", 0));
  assert_eq!("".to_string(), apply_horizontal_offset("", 1));
  assert_eq!("".to_string(), apply_horizontal_offset("", 100));
  assert_eq!("1".to_string(), apply_horizontal_offset("1", 0));
  assert_eq!("".to_string(), apply_horizontal_offset("1", 1));
  assert_eq!("".to_string(), apply_horizontal_offset("1", 2));
  assert_eq!("".to_string(), apply_horizontal_offset("1", 100));
  assert_eq!("11".to_string(), apply_horizontal_offset("11", 0));
  assert_eq!("1".to_string(), apply_horizontal_offset("11", 1));
  assert_eq!("".to_string(), apply_horizontal_offset("11", 2));
  assert_eq!("".to_string(), apply_horizontal_offset("11", 3));
  assert_eq!("111".to_string(), apply_horizontal_offset("111", 0));
  assert_eq!("11".to_string(), apply_horizontal_offset("111", 1));
  assert_eq!("1".to_string(), apply_horizontal_offset("111", 2));
  assert_eq!("".to_string(), apply_horizontal_offset("111", 3));
  assert_eq!("".to_string(), apply_horizontal_offset("111", 4));

  assert_eq!("111\n2222".to_string(), apply_horizontal_offset("111\n2222", 0));
  assert_eq!("11\n222".to_string(), apply_horizontal_offset("111\n2222", 1));
  assert_eq!("1\n22".to_string(), apply_horizontal_offset("111\n2222", 2));
  assert_eq!("\n2".to_string(), apply_horizontal_offset("111\n2222", 3));
 }

 #[test]
 fn test_apply_horizontal_offset_line() {
  assert_eq!("", apply_horizontal_offset_line("", 0));
  assert_eq!("", apply_horizontal_offset_line("", 1));
  assert_eq!("", apply_horizontal_offset_line("", 100));
  assert_eq!("1", apply_horizontal_offset_line("1", 0));
  assert_eq!("", apply_horizontal_offset_line("1", 1));
  assert_eq!("", apply_horizontal_offset_line("1", 2));
  assert_eq!("", apply_horizontal_offset_line("1", 100));
  assert_eq!("11", apply_horizontal_offset_line("11", 0));
  assert_eq!("1", apply_horizontal_offset_line("11", 1));
  assert_eq!("", apply_horizontal_offset_line("11", 2));
  assert_eq!("", apply_horizontal_offset_line("11", 3));
  assert_eq!("111", apply_horizontal_offset_line("111", 0));
  assert_eq!("11", apply_horizontal_offset_line("111", 1));
  assert_eq!("1", apply_horizontal_offset_line("111", 2));
  assert_eq!("", apply_horizontal_offset_line("111", 3));
  assert_eq!("", apply_horizontal_offset_line("111", 4));
 }

 use super::apply_hoffset_and_trim;
 use super::apply_hoffset_and_trim_line;
 use ratatui::layout::Rect;

 #[test]
 fn test_apply_hoffset_and_trim() {
  assert_eq!("".to_string(), apply_hoffset_and_trim("", Rect::new(0, 0, 1, 1), 0));
  assert_eq!("123".to_string(), apply_hoffset_and_trim("123", Rect::new(0, 0, 4, 1), 0));
  assert_eq!("123".to_string(), apply_hoffset_and_trim("123", Rect::new(0, 0, 3, 1), 0));
  assert_eq!("12".to_string(), apply_hoffset_and_trim("123", Rect::new(0, 0, 2, 1), 0));
  assert_eq!("1".to_string(), apply_hoffset_and_trim("123", Rect::new(0, 0, 1, 1), 0));
  assert_eq!("".to_string(), apply_hoffset_and_trim("123", Rect::new(0, 0, 0, 1), 0));
  assert_eq!("3".to_string(), apply_hoffset_and_trim("12345", Rect::new(0, 0, 1, 1), 2));
 }

 #[test]
 fn test_apply_hoffset_and_trim_line_line() {
  assert_eq!("", apply_hoffset_and_trim_line("", Rect::new(0, 0, 1, 1), 0));
  assert_eq!("123", apply_hoffset_and_trim_line("123", Rect::new(0, 0, 4, 1), 0));
  assert_eq!("123", apply_hoffset_and_trim_line("123", Rect::new(0, 0, 3, 1), 0));
  assert_eq!("12", apply_hoffset_and_trim_line("123", Rect::new(0, 0, 2, 1), 0));
  assert_eq!("1", apply_hoffset_and_trim_line("123", Rect::new(0, 0, 1, 1), 0));
  assert_eq!("", apply_hoffset_and_trim_line("123", Rect::new(0, 0, 0, 1), 0));
  assert_eq!("3", apply_hoffset_and_trim_line("12345", Rect::new(0, 0, 1, 1), 2));
 }
}

fn apply_horizontal_offset_line(line: &str, offset: usize) -> &str {
 {
  let mut display_width = 0;
  let mut byte_offset = 0;
  for (i, c) in line.char_indices() {
   assert_eq!(i, byte_offset);
   let char_width = c.width().unwrap_or(0);
   if display_width + char_width <= offset {
    display_width += char_width;
    byte_offset = i + c.len_utf8();
   } else {
    byte_offset = i;
    break;
   }
  }
  if byte_offset >= line.len() {
   ""
  } else {
   &line[byte_offset..]
  }
 }
}

fn apply_horizontal_offset(text: &str, offset: usize) -> String {
 if offset == 0 {
  // geht auch ohne
  return text.to_string();
 }

 text
  .split('\n')
  .map(|line| apply_horizontal_offset_line(line, offset))
  .collect::<Vec<_>>()
  .join("\n")
}

fn apply_hoffset_and_trim_line(text: &str, rect: Rect, hoffset: usize) -> &str {
 let max_width = rect.width as usize;
 // let max_height = rect.height as usize;
 let offset_text = apply_horizontal_offset_line(text, hoffset);
 truncate_before_or_at_display_width(offset_text, max_width)
}

fn apply_hoffset_and_trim(text: &str, rect: Rect, hoffset: usize) -> String {
 let max_width = rect.width as usize;
 let max_height = rect.height as usize;

 let offset_text = apply_horizontal_offset(text, hoffset);

 offset_text
  .split("\n")
  .take(max_height)
  .map(|line| truncate_before_or_at_display_width(line, max_width))
  .collect::<Vec<_>>()
  .join("\n")
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

/// R for refactor
enum R<'a> {
 Old(&'a str),
 VS(&'a [&'a str]),
 VS2(&'a [String]),
}

/// the TwoScreenDefaultWidget paints in the areas of the
/// rv.pl (RatatuiVariables . PagerLayout)

struct TwoScreenDefaultWidget<'a> {
 helpline: &'a str,
 main_title: &'a str,
 second_title: &'a str,
 rv: &'a RatatuiVariables,
 all_lines: R<'a>,
 all_lines2: R<'a>,
 wrapped1: bool,
 wrapped2: bool,
 regex_edit_mode: Option<String>,
 regex_edit_mode_state: String,
 regex_count: usize,
 line_count: usize,
 line_count2: Option<usize>,
 delete_confirm_mode: Option<usize>,
 statusline_heap: Rc<RefCell<BinaryHeap<StatusMessage>>>,
 paused: bool,
 active_area: ActiveArea,
 hoffset_main: usize,
 hoffset_second: usize,
}

impl<'a> Widget for TwoScreenDefaultWidget<'a> {
 fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
 where
  Self: Sized,
 {
  // assert_eq!(area, buf.area); // this is only in the rootwidget, buf.area is always the root area
  let is_main_active = self.active_area == ActiveArea::Main;

  let regex_count_indicator =
   if 0 != self.regex_count { &format!(" r({})", self.regex_count) } else { "" };
  let title = " ".to_string()
   + if is_main_active { "* " } else { "  " }
   + self.main_title
   + &format!(" l({})", self.line_count)
   + if self.wrapped1 { " (w)" } else { "" }
   + regex_count_indicator
   + " ";

  let top_right_line_text = if self.paused { " PAUSED " } else { "" };
  let bottom_center_line_text = if self.paused { " PAUSED " } else { "" };

  let block = Block::bordered()
   .title(title)
   .title_alignment(Alignment::Left)
   .title(Line::from(top_right_line_text).right_aligned())
   .title_bottom(Line::from(bottom_center_line_text).centered())
   .border_type(BorderType::Rounded);

  // if !is_main_active {
  //  block = block.border_style(Style::new().fg(Color::Gray));
  // }

  // let rect1 = self.rv.pl.get_main_area().inner(Margin::new(0, 0));
  let rect1 = *self.rv.pl.get_main_area();
  let safe_area = rect1.intersection(area); // avoids crash

  let all_lines = match self.all_lines {
   R::Old(al) => {
    let all_lines = tabfix(al);

    if self.wrapped1 {
     all_lines
    } else {
     apply_hoffset_and_trim(&all_lines, safe_area, self.hoffset_main)
    }
   }
   R::VS(vs) => vs
    .iter()
    .map(|x| {
     let x = tabfix(x);
     if self.wrapped1 {
      x
     } else {
      apply_hoffset_and_trim_line(&x, safe_area, self.hoffset_main).to_string()
     }
    })
    .collect::<Vec<_>>()
    .join("\n"),
   R::VS2(vs2) => vs2
    .iter()
    .map(|x| {
     let x = tabfix(x);
     if self.wrapped1 {
      x
     } else {
      apply_hoffset_and_trim_line(&x, safe_area, self.hoffset_main).to_string()
     }
    })
    .collect::<Vec<_>>()
    .join("\n"),
  };

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

  let is_second_active = self.active_area == ActiveArea::Second;

  if let Some(sma) = self.rv.pl.get_second_main_area() {
   let title2 = " ".to_string()
    + if is_second_active { "* " } else { "  " }
    + self.second_title
    + &self
     .line_count2
     .map_or("".to_string(), |x| format!(" l({})", x))
    + if self.wrapped2 { " (w)" } else { "" }
    + " ";

   // &format!(" l({})", self.line_count2);

   let block2 = Block::bordered()
    .title(title2)
    .title_alignment(Alignment::Left)
    .border_type(BorderType::Rounded);

   // if !is_second_active {
   //  block2 = block2.border_style(Style::new().fg(Color::Gray));
   // }

   // let rect2 = sma.inner(Margin::new(0, 1));
   let rect2 = *sma;
   let safe_area2 = rect2.intersection(area); // avoids crash

   let all_lines2 = match self.all_lines2 {
    R::Old(al) => {
     let all_lines2 = tabfix(al);

     if self.wrapped2 {
      all_lines2
     } else {
      apply_hoffset_and_trim(&all_lines2, safe_area2, self.hoffset_second)
     }
    }
    R::VS(vs) => vs
     .iter()
     .map(|x| {
      let x = tabfix(x);
      if self.wrapped2 {
       x
      } else {
       apply_hoffset_and_trim_line(&x, safe_area2, self.hoffset_second).to_string()
      }
     })
     .collect::<Vec<_>>()
     .join("\n"),
    R::VS2(vs2) => vs2
     .iter()
     .map(|x| {
      let x = tabfix(x);
      if self.wrapped2 {
       x
      } else {
       apply_hoffset_and_trim_line(&x, safe_area2, self.hoffset_second).to_string()
      }
     })
     .collect::<Vec<_>>()
     .join("\n"),
   };

   let paragraph2 = Paragraph::new(all_lines2).block(block2).left_aligned();

   // weue806j1y
   let paragraph2 = if !self.wrapped2 { paragraph2 } else { paragraph2.wrap(Wrap { trim: false }) };

   // Clear.render(safe_area2, buf); // doesn't fix the tab problem
   paragraph2.render(safe_area2, buf);
  }
  // Paragraph::new("statusline").render( self.rv.pl.get_status_area().intersection(area), buf);
  let statusline = self.statusline_heap.borrow();
  if let Some(regex_edit_mode) = &self.regex_edit_mode {
   Paragraph::new("/".to_string() + regex_edit_mode + &self.regex_edit_mode_state + " (Esc/Enter)")
    .render(self.rv.pl.get_status_area().intersection(area), buf);
  } else if self.delete_confirm_mode.is_some() {
   Paragraph::new("delete entry? (y/n) (Esc)")
    .render(self.rv.pl.get_status_area().intersection(area), buf);
  } else if let Some(status_msg) = statusline.peek() {
   Paragraph::new(status_msg.text.clone() + &format!(" c({})", statusline.len()) + " (Esc)")
    .render(self.rv.pl.get_status_area().intersection(area), buf);
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

 fn paint_without_terminal(&mut self, _assd: &mut AppStateReceiverData) {
  unreachable!("paint_without_terminal() must be implemented");
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
 fn paint(&mut self, terminal: &mut DefaultTerminal, _assd: &mut AppStateReceiverData) {
  let rv = &RatatuiVariables::new::<PagerLayoutBase>(terminal);

  // if let Some(rc) = &self.tsp_before {
  //  // rc.borrow_mut().handle_event(&MyEvent::Tick, assd);
  //  rc.borrow_mut().paint(terminal, assd);
  // }

  //  writes in the correct area but overwrites the upper part
  terminal
   .draw(|frame| {
    frame.render_widget(
     Paragraph::new(self.question.clone()),
     rv.pl.get_status_area().intersection(frame.area()),
    )
   })
   .unwrap();
 }

 fn handle_event(&mut self, evt: &MyEvent, _assd: &mut AppStateReceiverData) -> NextTsp {
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
 scroller_main: Scroller,
 scroller_second: Scroller,
 layout: Layout,
 flipstate: u8,
 wrapped: bool,
 paused: bool,
 regex_edit_mode: Option<String>,
 regex_edit_mode_state: String,
 regex_edit_mode_last_working: Option<Regex>,
 regex: Vec<Regex>,
 regex_filtered_cbs_entries: VecDeque<FilteredCbsEntries>,
 delete_confirm_mode: Option<usize>,
 active_area: ActiveArea,
 main_width: usize,
 second_width: usize,
 prev_selected_text: Option<Vec<u8>>,
}

enum FilteredCbsEntries {
 ACE(AppendedCBEntry),
 Line,
 Empty,
}

// TODO : mode in the vicinity of first_page() definition (maybe inside)
impl TermionScreenFirstPage {
 pub fn new(config: &'static Config) -> Self {
  Self {
   config,
   scroller_main: Scroller::new(),
   scroller_second: Scroller::new(),
   layout: Layout::new(),
   flipstate: 1,
   wrapped: false,
   paused: false,
   regex_edit_mode: None,
   regex_edit_mode_state: "".to_string(),
   regex_edit_mode_last_working: None,
   regex: vec![],
   regex_filtered_cbs_entries: VecDeque::new(),
   delete_confirm_mode: None,
   active_area: ActiveArea::Main,
   main_width: 80,
   second_width: 80,
   prev_selected_text: None,
  }
 }

 fn flipstate_next(&mut self) {
  self.flipstate = (self.flipstate + 1) % 3;
 }
 fn flipstate_prev(&mut self) {
  self.flipstate = (self.flipstate + 2) % 3;
 }

 fn get_max_hoffset_main(&self) -> usize {
  let entries = &self.regex_filtered_cbs_entries;
  let max_line_width = entries
   .iter()
   .map(|e| match e {
    FilteredCbsEntries::ACE(a) => a.cbentry.borrow().as_string().width(),
    _ => 0,
   })
   .max()
   .unwrap_or(0);
  // max_line_width.saturating_sub(self.main_width / 2)
  max_line_width
 }

 fn get_max_hoffset_second(&self) -> usize {
  let entries = &self.regex_filtered_cbs_entries;
  let max_line_width = entries
   .iter()
   .filter_map(|e| match e {
    FilteredCbsEntries::ACE(a) => {
     let text = a.cbentry.borrow().as_string().into_owned();
     if text.contains('\n') {
      Some(text.lines().map(|l| l.width()).max().unwrap_or(0))
     } else {
      Some(text.width())
     }
    }
    _ => None,
   })
   .max()
   .unwrap_or(0);
  // max_line_width.saturating_sub(self.second_width / 2)
  max_line_width
 }

 fn toggle_active_area(&mut self) {
  self.active_area = match self.active_area {
   ActiveArea::Main => ActiveArea::Second,
   ActiveArea::Second => ActiveArea::Main,
  };
 }

 fn get_active_scroller(&mut self) -> &mut Scroller {
  match self.active_area {
   ActiveArea::Main => &mut self.scroller_main,
   ActiveArea::Second => &mut self.scroller_second,
  }
 }
}

impl TermionScreenPainter for TermionScreenFirstPage {
 /// the paint method opens a TwoScreenDefaultWidget which is later painted
 /// by the terminal.draw method
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let layout = &mut self.layout;

  let cbs = &mut assd.cbs;

  let rv = if self.flipstate == 0 {
   &RatatuiVariables::new::<PagerLayoutBase>(terminal)
  } else if self.flipstate == 1 {
   &RatatuiVariables::new::<PagerLayoutTB>(terminal)
  } else {
   &RatatuiVariables::new::<PagerLayoutLR>(terminal)
  };

  self.main_width = rv.pl.get_main_area().width as usize;
  self.second_width = rv
   .pl
   .get_second_main_area()
   .map_or(self.main_width, |r| r.width as usize);

  {
   let inner_main_rect = rv.pl.get_main_area().inner(Margin::new(1, 1));
   layout.set_width_height_from_rect(inner_main_rect);
   let inner_second_rect = rv
    .pl
    .get_second_main_area()
    .map(|x| x.inner(Margin::new(1, 1)));

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
       if !r.is_match(&line.cbentry.borrow().as_string()) {
        res = false;
        break;
       }
      }
      res
     })
     .collect::<VecDeque<_>>();

    self.regex_filtered_cbs_entries = entries
     .iter()
     .map(|x| FilteredCbsEntries::ACE((**x).clone()))
     .collect::<VecDeque<FilteredCbsEntries>>();

    drop(entries);

    {
     let cbtype_enum_vector: Vec<CBType> = all::<CBType>().collect::<Vec<_>>();
     let mut last_entries = cbtype_enum_vector
      .iter()
      .map(|x| cbs.last_entries.get(x))
      .collect::<Vec<_>>();

     last_entries.sort_by(|a, b| match (a, b) {
      (None, None) => Ordering::Equal,
      (None, Some(_)) => Ordering::Less,
      (Some(_), None) => Ordering::Greater,
      (Some(c), Some(d)) => c
       .cbentry
       .borrow()
       .get_timestamp()
       .cmp(&d.cbentry.borrow().get_timestamp()),
     });

     self
      .regex_filtered_cbs_entries
      .push_front(FilteredCbsEntries::Line);
     last_entries
      .iter()
      .map(|x| match x {
       Some(v) => FilteredCbsEntries::ACE((*v).clone()),
       None => FilteredCbsEntries::Empty,
      })
      .for_each(|x| self.regex_filtered_cbs_entries.push_front(x));
    }

    let entries = &self.regex_filtered_cbs_entries;

    // let mut selected_string = Vec::<u8>::new();
    // let mut selected_lines = &Vec::<String>::new();
    let mut selected_cbentry: Option<Rc<RefCell<CBEntry>>> = None;
    let mut line_count2 = None;

    if self.config.debug {
     trace!("scroller.set_content_length(entries.len()) : {}", entries.len());
    }
    self.scroller_main.set_hwindowlength(self.main_width);
    self.scroller_second.set_hwindowlength(self.second_width);
    self.scroller_main.set_content_length(entries.len());

    // scroller.set_windowlength(height + 1 - layout.get_current_line());
    self
     .scroller_main
     .set_windowlength(inner_main_rect.height as usize);

    let second_area_height = inner_second_rect.map_or(0, |x| x.height as usize);
    self.scroller_second.set_windowlength(second_area_height);

    let numbers_width = (entries.len() as f64).log10().ceil() as usize;

    if self.config.debug {
     trace!("scroller.get_safe_windowrange() : {:?}", self.scroller_main.get_safe_windowrange());
    }

    // iwcqjc9i11 Example for the line selection

    for (idx, entry) in entries
     .range(self.scroller_main.get_safe_windowrange())
     .enumerate()
    {
     match entry {
      FilteredCbsEntries::ACE(appended_cbentry) => {
       let cbentry = &appended_cbentry.cbentry;
       let is_cursor = match self.scroller_main.get_cursor() {
        None => false,
        Some(value) => idx == value,
       };

       let cursor_star = if is_cursor { ">" } else { " " };

       // let is_selected = entry.is_selected(cbs);
       let is_selected = cbs.is_fixated(cbentry);

       let selection_star = if is_selected { "*" } else { " " };

       let cbentry_borrowed = cbentry.borrow_mut();

       if is_cursor {
        if self.prev_selected_text.as_ref() != Some(cbentry_borrowed.get_data()) {
         self.scroller_second.reset_hoffset();
         self.prev_selected_text = Some(cbentry_borrowed.get_data().clone());
        }
        // selected_string = cbentry.data.clone();
        // selected_lines = cbentry_borrowed.get_text();
        selected_cbentry = Some(Rc::clone(cbentry));
        let _ = line_count2.insert(cbentry_borrowed.get_text().len());
       }

       {
        let s002 = format!(
         "{} {} {:width$} {} {} : {}",
         cursor_star,
         selection_star,
         idx + self.scroller_main.get_windowposition(), // mqbojcmkot
         cbentry_borrowed.get_cbtype().get_info(),
         cbentry_borrowed.get_date_time(),
         cbentry_borrowed.as_string(),
         width = numbers_width,
        );
        // lines.push(layout.fixline(&s002));
        lines.push(flatline(&s002));
       }
      }
      FilteredCbsEntries::Line => {
       lines.push(layout.centerline("----- ↑ active ↑ ----- ↓ incoming ↓ -----"));
      }
      FilteredCbsEntries::Empty => {
       lines.push("".into());
      }
     }
    }

    // let all_lines = lines.join("\n");
    let all_lines = lines;

    let all_lines2 = {
     let string_lines = match selected_cbentry {
      Some(rc) => rc.borrow().get_text().clone(),
      None => vec![],
     };
     self.scroller_second.set_content_length(string_lines.len());

     render_scroller_lines2(
      &mut self.scroller_second,
      &string_lines,
      self.wrapped,
      layout,
      |cursor_star, idx, numbers_width, entry| {
       format!("{} {:width$} : {}", cursor_star, idx, entry, width = numbers_width,)
      },
     )
    };

    let sw = TwoScreenDefaultWidget {
     helpline: HELP_FIRST_PAGE,
     main_title: "entry list",
     second_title: "selected content",
     rv,
     // tsfp: &self,
     all_lines: R::VS2(&all_lines),
     all_lines2: R::VS2(&all_lines2),
     wrapped1: false,
     wrapped2: self.wrapped,
     regex_edit_mode: self.regex_edit_mode.clone(),
     regex_edit_mode_state: self.regex_edit_mode_state.clone(),
     regex_count: self.regex.len() + self.regex_edit_mode.is_some() as usize,
     line_count: entries.len(),
     // line_count2: selected_string.lines().count(),
     line_count2,
     delete_confirm_mode: self.delete_confirm_mode,
     statusline_heap: Rc::clone(&assd.statusline_heap),
     paused: self.paused,
     active_area: self.active_area,
     hoffset_main: self.scroller_main.get_hoffset(),
     hoffset_second: self.scroller_second.get_hoffset(),
    };

    terminal
     .draw(|frame| frame.render_widget(sw, frame.area()))
     .unwrap();
   }
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
  let cbs = &mut assd.cbs;

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
     let _ = self.regex_edit_mode.insert(regex_edit_mode);
    }
    MyEvent::Termion(Event::Key(Key::Char(char))) => {
     regex_edit_mode.push(*char);
     let _ = self.regex_edit_mode.insert(regex_edit_mode);
    }
    _ => {}
   }
   return NextTsp::IgnoreBasicEvents;
  } else if self.delete_confirm_mode.is_some() {
   match evt {
    MyEvent::Termion(Event::Key(Key::Esc)) => {
     self.delete_confirm_mode = None;
    }
    MyEvent::Termion(Event::Key(Key::Char('y'))) => {
     if let Some(seq) = self.delete_confirm_mode {
      cbs.remove_by_seq(seq);
      self.delete_confirm_mode = None;
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('n'))) => {
     self.delete_confirm_mode = None;
    }
    _ => {}
   }
   return NextTsp::IgnoreBasicEvents;
  } else {
   match evt {
    MyEvent::Termion(Event::Key(Key::Char('r'))) => {
     self.regex.pop();
    }
    //  MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
    MyEvent::Termion(Event::Key(Key::Char('h'))) => {
     return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenViewPage::new(
      self.config,
      "help".to_string(),
      CBEntry::new(config::USAGE.to_string().as_bytes()),
     ))));
    }
    MyEvent::Termion(Event::Key(Key::Char('f'))) => {
     self.flipstate_next();
    }
    MyEvent::Termion(Event::Key(Key::Char('F'))) => {
     self.flipstate_prev();
    }
    MyEvent::Termion(Event::Key(Key::Char('s'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_array() {
      let entries = &self.regex_filtered_cbs_entries;
      if let FilteredCbsEntries::ACE(appended_cbentry) = &entries[cursor] {
       // let entry = &appended_cbentry.cbentry.clone();
       // let entry = &entries[cursor].cbentry.clone(); // NOTE: the clone can maybe avoided when I put this logic into cbs
       // entry.toggle_selection(&mut cbs);
       cbs.toggle_fixation(appended_cbentry);
      }
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('t'))) => {
     cbs.toggle_clipboards();
    }
    MyEvent::Termion(Event::Key(Key::Char('v'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_array() {
      let entries = &self.regex_filtered_cbs_entries;
      if let FilteredCbsEntries::ACE(appended_cbentry) = &entries[cursor] {
       return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenViewPage::new(
        self.config,
        "view entry".to_string(),
        appended_cbentry.cbentry.borrow().clone(),
       ))));
      };
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('e'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_array() {
      let entries = &self.regex_filtered_cbs_entries;
      // let entry = &entries[cursor];

      if let FilteredCbsEntries::ACE(appended_cbentry) = &entries[cursor] {
       match TermionScreenEditorPage::new(
        self.config,
        appended_cbentry
         .cbentry
         .borrow_mut()
         .as_string()
         .into_owned(),
        cursor,
       ) {
        Ok(page) => return NextTsp::Stack(Rc::new(RefCell::new(page))),
        Err(e) => {
         eprintln!("Failed to create editor page: {}", e);
         match assd.statusline_heap.try_borrow_mut() {
          Ok(mut v) => v.push(StatusMessage {
           severity: crate::libmain::StatusSeverity::Warning,
           text: format!("Failed to create editor page: {}", e),
          }),
          Err(err) => {
           trace!("Failed to create editor page: {}", e);
           trace!("Failed to create editor page: {}", err);
           trace!("Failed to open statusline heap: ");
          }
         }
         return NextTsp::NoNextTsp;
        }
       }
      }
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('w'))) => {
     self.wrapped = !self.wrapped;
    }
    MyEvent::Termion(Event::Key(Key::Char('\t'))) => {
     self.toggle_active_area();
    }
    MyEvent::Termion(Event::Key(Key::Char('d'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_array() {
      let entries = &self.regex_filtered_cbs_entries;
      if entries.is_empty() {
      } else if let FilteredCbsEntries::ACE(appended_cbentry) = &entries[cursor] {
       self.delete_confirm_mode = Some(appended_cbentry.seq);
      }
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('p'))) => {
     assd.sender.send(MyEvent::TogglePause).unwrap();
    }
    MyEvent::TogglePauseResult(paused) => {
     self.paused = *paused;
    }
    MyEvent::Termion(Event::Key(Key::Char('/'))) => {
     self.regex_edit_mode = Some("".to_string());
    }
    _ => {
     // TODO : optimize
     let max_offset = match self.active_area {
      ActiveArea::Main => self.get_max_hoffset_main(),
      ActiveArea::Second => self.get_max_hoffset_second(),
     };

     let scroller = self.get_active_scroller();

     scroller.set_max_hoffset(max_offset);
     Pager::handle_event(scroller, evt);
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
 pub fn new(config: &'static Config, text: String, index: usize) -> Result<Self, String> {
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
   index,
  })
 }
}

impl TermionScreenPainter for TermionScreenEditorPage {
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
       let idx = self.index;
       if let Some(entry) = assd.cbs.cbentries.get_mut(idx) {
        entry.cbentry.borrow_mut().set_data(&buf);
       }
      }
      Err(err) => assd.statusline_heap.borrow_mut().push(StatusMessage {
       severity: StatusSeverity::Error,
       text: err.to_string(),
      }),
     };
    }
    Err(err) => assd.statusline_heap.borrow_mut().push(StatusMessage {
     severity: StatusSeverity::Error,
     text: err.to_string(),
    }),
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

pub struct TermionScreenViewPage {
 config: &'static Config,
 main_title: String,
 scroller: Scroller,
 layout: Layout,
 text: CBEntry,
 wrapped: bool,
}

impl TermionScreenViewPage {
 fn new(config: &'static Config, main_title: String, text: CBEntry) -> Self
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

 // TODO : ?
 fn get_max_hoffset(&self) -> usize {
  let max_line_width = self
   .text
   .get_text()
   .iter()
   .map(|l| l.width())
   .max()
   .unwrap_or(0);
  // let window_width = 80;
  // max_line_width.saturating_sub(window_width / 2)
  max_line_width
 }
}

impl TermionScreenPainter for TermionScreenViewPage {
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let scroller = &mut self.scroller;
  let layout = &mut self.layout;

  let string_lines: Vec<String> = self
   .text
   .get_text()
   .iter()
   .map(|x| x.to_string())
   .collect::<Vec<_>>();

  let rv = RatatuiVariables::new::<PagerLayoutBase>(terminal);

  {
   let inner_main_rect = rv.pl.get_main_area().inner(Margin::new(1, 1));
   layout.set_width_height_from_rect(inner_main_rect);

   scroller.set_content_length(string_lines.len());
   // scroller.set_windowlength(height + 1 - layout.get_current_line());
   // scroller.set_windowlength(rv.pl.get_main_area().inner(Margin::new(0, 1)).height as usize);
   scroller.set_windowlength(inner_main_rect.height as usize);

   // TODO : render_scroller_lines2
   let all_lines = render_scroller_lines2(
    scroller,
    &string_lines,
    self.wrapped,
    layout,
    |cursor_star, idx, numbers_width, entry| {
     format!("{} {:width$} : {}", cursor_star, idx, entry, width = numbers_width,)
    },
   );
   // for R::VS
   // let all_lines = all_lines.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
   // for R::Old
   // let all_lines = all_lines.join( "\n");

   let sw = TwoScreenDefaultWidget {
    helpline: HELP_QX,
    main_title: &self.main_title,
    second_title: "unused",
    rv: &rv,
    // all_lines: R::Old(&all_lines),
    all_lines: R::VS2(all_lines.as_ref()),
    all_lines2: R::Old("unused"),
    wrapped1: self.wrapped,
    wrapped2: false,
    regex_edit_mode: None,
    regex_edit_mode_state: "".to_string(),
    regex_count: 0,
    line_count: string_lines.len(),
    line_count2: None,
    delete_confirm_mode: None,
    statusline_heap: Rc::clone(&assd.statusline_heap),
    paused: false,
    active_area: ActiveArea::Main,
    hoffset_main: self.scroller.get_hoffset(),
    hoffset_second: 0,
   };

   terminal
    .draw(|frame| frame.render_widget(sw, frame.area()))
    .unwrap();
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
    // TODO : optimize
    self.scroller.set_max_hoffset(self.get_max_hoffset());
    Pager::handle_event(&mut self.scroller, evt);
   }
  }
  NextTsp::NoNextTsp
 }
}
