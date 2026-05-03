// #![allow(dead_code)]
// #![allow(unused)]

use std::cell::{RefCell, RefMut};
use std::cmp::Ordering;
use std::collections::VecDeque;

use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use std::rc::Rc;

use crate::clipboards::AppendedCBEntry;
use crate::clipboards::{cbentry::CBEntry, AcbeId, CBType};
use crate::color_theme::{ColorTheme, ThemeColors};
use crate::config::{self, Config};
use crate::constants::{self, HELP_FIRST_PAGE, HELP_WQX};
use crate::event::MyEvent;
use crate::layout::Layout;
use crate::layout_ratatui::{PagerLayout, PagerLayoutBase, PagerLayoutLR, PagerLayoutTB};
// use crate::libmain::SyncStuff;
use crate::libmain::{AppStateReceiverData, StatusLineHeap, StatusSeverity};
use crate::linuxeditor;
use crate::pager::Pager;
use crate::scroller::Scroller;
use crate::tools::{flatline, tabfix};

use enum_iterator::all;
use mktemp::Temp;
use ratatui::layout::{Alignment, Margin, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Paragraph, Widget};
use ratatui::DefaultTerminal;
use termion::event::{Event, Key};
use termion::{self};

use tracing::trace;

use unicode_width::UnicodeWidthChar; // extends char by width, width_cjk
use unicode_width::UnicodeWidthStr; // extends &str by width, width_cjk

use regex::Regex;
use std::fmt::Debug;
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

fn truncate_before_or_at_display_width2(text: &str, width: usize) -> (&str, &str) {
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
 (&text[0..last_idx], &text[last_idx..])
}

pub(crate) fn wrap_text(text: &str, width: usize) -> Vec<&str> {
 let mut ret: Vec<&str> = Vec::new();
 let mut a = text;
 loop {
  let truncated = truncate_before_or_at_display_width2(a, width);

  ret.push(truncated.0);

  if truncated.1.is_empty() {
   break;
  }
  a = truncated.1;
 }

 ret
}

fn render_scroller_lines4<'a, T>(
 scroller: &mut Scroller,
 items: &[T],
 wrapped: bool,
 _layout: &Layout,
 formatter: impl Fn(&str, usize, usize, &T) -> LineStrings<'a>,
) -> Vec<LineStrings<'a>> {
 let numbers_width = (items.len() as f64).log10().ceil() as usize;
 let mut lines = vec![];

 for (idx, item) in items[scroller.get_safe_windowrange()].iter().enumerate() {
  let is_cursor = match scroller.get_cursor_in_window() {
   None => false,
   Some(value) => idx == value,
  };
  let cursor_star = if is_cursor { ">" } else { " " };

  // let line = formatter(cursor_star, idx + scroller.get_windowposition(), numbers_width, item);
  let line = formatter(cursor_star, idx + scroller.get_windowposition(), numbers_width, item);
  lines.push(if wrapped { line } else { line });

  // lines.push(if true { line } else { layout.fixline(&line) });
 }
 lines
}

#[cfg(test)]
mod unicode_tests {
 // use unicode_width::UnicodeWidthChar; // extends char by width, width_cjk
 use unicode_width::UnicodeWidthStr; // extends &str by width, width_cjk

 use super::{truncate_before_or_at_display_width, wrap_text};

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

 #[test]
 fn test_wrap_text() {
  let text = "1234567890";

  assert_eq!(wrap_text(text, 3), ["123", "456", "789", "0"]);
 }
}

#[cfg(test)]
mod termionscreen_tests {
 use super::*;
 use crate::clipboards::{CBType, Clipboards};
 use crate::event::MyEvent;
 use crate::libmain::AppStateReceiverData;
 use std::collections::BinaryHeap;
 use std::rc::Rc;
 use std::sync::mpsc::channel;

 #[test]
 fn test_cb_inserted_sets_needs_refilter() {
  let (sender, _receiver) = channel();
  let config = Box::leak(Box::new(Config::default()));
  let mut assd = AppStateReceiverData::new(config, sender);
  assd
   .cbs
   .insert(&CBType::Clipboard, Some(b"test data".to_vec()));

  let mut screen = TermionScreenFirstPage::new(config);

  let next = screen.handle_event(&MyEvent::CbInserted, &mut assd);
  assert!(screen.needs_refilter);
  assert_eq!(next, NextTsp::NoNextTsp);
 }

 #[test]
 fn test_cb_changed_updates_clipboard_and_refilters() {
  let (sender, _receiver) = channel();
  let config = Box::leak(Box::new(Config::default()));
  let mut assd = AppStateReceiverData::new(config, sender);

  let mut screen = TermionScreenFirstPage::new(config);

  assd
   .cbs
   .insert(&CBType::Clipboard, Some(b"new entry".to_vec()));
  let next = screen.handle_event(&MyEvent::CbInserted, &mut assd);

  assert!(screen.needs_refilter);
  assert_eq!(next, NextTsp::NoNextTsp);
 }

 #[test]
 fn test_termion_event_key_handles_normally() {
  let (sender, _receiver) = channel();
  let config = Box::leak(Box::new(Config::default()));
  let mut assd = AppStateReceiverData::new(config, sender);

  let mut screen = TermionScreenFirstPage::new(config);
  screen.needs_refilter = false;

  let next = screen.handle_event(&MyEvent::Termion(Event::Key(Key::Char('a'))), &mut assd);

  assert!(!screen.needs_refilter);
  assert_eq!(next, NextTsp::NoNextTsp);
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
mod unicode_with_test {
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

fn apply_horizontal_offset_line2<'a>(line1: &'a str, line2: &'a str, offset: usize) -> String {
 {
  // let line = line1.to_string() + line2;
  let offset = offset.saturating_sub(line1.width());
  let mut display_width = 0;
  let mut byte_offset = 0;
  for (i, c) in line2.char_indices() {
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
  if byte_offset >= line2.len() {
   line1.to_string()
  } else {
   line1.to_string() + &line2[byte_offset..]
  }
 }
}

fn apply_horizontal_offset_line3<'a>(
 line1: &'a str,
 line2: &'a str,
 offset: usize,
) -> (String, String) {
 {
  // let line = line1.to_string() + line2;
  let offset = offset.saturating_sub(line1.width());
  let mut display_width = 0;
  let mut byte_offset = 0;
  for (i, c) in line2.char_indices() {
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
  if byte_offset >= line2.len() {
   (line1.to_string(), "".into())
  } else {
   (line1.to_string(), line2[byte_offset..].to_string())
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
fn apply_hoffset_and_trim_line2<'a>(
 text1: &'a str,
 text2: &'a str,
 rect: Rect,
 hoffset: usize,
) -> String {
 let max_width = rect.width as usize;
 // let max_height = rect.height as usize;
 // let offset_text = apply_horizontal_offset_line(text, hoffset);
 let offset_text = apply_horizontal_offset_line2(text1, text2, hoffset);
 truncate_before_or_at_display_width(&offset_text, max_width).to_string()
}

fn apply_hoffset_and_trim_line3<'a>(
 text1: &'a str,
 text2: &'a str,
 rect: Rect,
 hoffset: usize,
) -> (String, String) {
 let max_width = rect.width as usize;
 // let max_height = rect.height as usize;
 // let offset_text = apply_horizontal_offset_line(text, hoffset);
 let (offset_text1, offset_text2) = apply_horizontal_offset_line3(text1, text2, hoffset);
 let w = offset_text1.width();
 let max_width = max_width.saturating_sub(w);
 (
  offset_text1.to_string(),
  truncate_before_or_at_display_width(&offset_text2, max_width).to_string(),
 )
}

fn apply_hoffset_and_trim_line3_array<'a>(
 text1: &'a str,
 text2: &'a str,
 rect: Rect,
 hoffset: usize,
) -> (String, Vec<String>) {
 let max_width = rect.width as usize;
 // let max_height = rect.height as usize;
 // let offset_text = apply_horizontal_offset_line(text, hoffset);
 let (offset_text1, offset_text2) = apply_horizontal_offset_line3(text1, text2, hoffset);
 let w = offset_text1.width();
 let max_width = max_width.saturating_sub(w);
 (
  offset_text1.to_string(),
  wrap_text(&offset_text2, max_width)
   .iter()
   .map(|x| x.to_string())
   .collect(),
 )
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

#[derive(Debug, PartialEq, Clone)]
enum LineStringsType<'a> {
 S(String),
 L(Vec<Line<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
struct LineStrings<'a> {
 wrapped: bool,
 cursor: String,
 line_number: String,
 text: LineStringsType<'a>,
}

#[derive(Debug, PartialEq, Clone)]
enum LineStringsWrappedType<'a> {
 S(Vec<String>),
 L(Vec<Line<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
struct LineStringsWrapped<'a> {
 wrapped: bool,
 cursor: String,
 line_number: String,
 text: LineStringsWrappedType<'a>,
}

/// manages the visible parts of a line in the pagers
impl<'a> LineStrings<'a> {
 fn tabfix(&self, hoffset: usize, safe_area: Rect) -> LineStringsWrapped {
  let newtext2 = match &self.text {
   LineStringsType::S(text) => {
    let text = tabfix(text);

    let newtext = match self.wrapped {
     true =>
     // vec![text.clone(), text], // gqhdbjurhn, TODO : wordwrap
     {
      apply_hoffset_and_trim_line3_array(
       // TODO : the "    " hack is not really good but works for the first part
       &(String::from("    ") + &self.cursor + &self.line_number),
       &text,
       safe_area,
       hoffset,
      )
      .1
     }
     false => vec![text],
    };
    LineStringsWrappedType::S(newtext)
   }

   LineStringsType::L(lines) => LineStringsWrappedType::L(lines.iter().cloned().collect()),
  };

  LineStringsWrapped {
   wrapped: self.wrapped,
   cursor: tabfix(&self.cursor),
   line_number: tabfix(&self.line_number),
   // text: tabfix(&self.text),
   text: newtext2,
  }
 }
}

/// manages the visible parts of the text in the pagers
#[derive(Debug, Default)]
struct LineStringsConfig<'a> {
 line_strings: &'a [LineStrings<'a>],
 wrapped: bool,
 title: &'a str,
 line_count: Option<usize>,
 hoffset: usize,
 theme_colors: ThemeColors,
 cursor_color: Option<ratatui::style::Color>,
 // scroller: Option<RefMut<'a, Scroller>>,
 // scroller: Option<&'a mut Scroller>,
}

impl<'a> LineStringsConfig<'a> {
 fn prepare2print(&self, safe_area: Rect) -> Vec<Vec<Line<'_>>>
// fn prepare2print(&self, safe_area: Rect) -> Vec<Line<'_>>
 {
  let cursor_style = if let Some(color) = self.cursor_color {
   Style::new().fg(color)
  } else if let Some(color) = self.theme_colors.cursor {
   Style::new().fg(color)
  } else {
   Style::new()
  };

  let line_number_style = if let Some(color) = self.theme_colors.line_number {
   Style::new().fg(color)
  } else {
   Style::new()
  };
  let text_style =
   if let Some(color) = self.theme_colors.text { Style::new().fg(color) } else { Style::new() };

  self
   .line_strings
   .iter()
   .map(|ls| {
    // LineStrings
    let lsw = ls.tabfix(self.hoffset, safe_area); // LineStringsWrapped

    match &lsw.text {
     LineStringsWrappedType::S(items) => {
      let res = items
       .iter()
       .map(|x| {
        apply_hoffset_and_trim_line3(
         &(String::new() + &lsw.cursor + &lsw.line_number),
         x,
         safe_area,
         self.hoffset,
        )
       })
       .collect::<Vec<_>>();

      res
       .iter()
       .map(|res| {
        let lsw = lsw.clone();
        Line::from(vec![
         Span::styled(lsw.cursor, cursor_style),
         Span::styled(lsw.line_number, line_number_style),
         Span::styled(res.1.clone(), text_style),
        ])
       })
       .collect::<Vec<_>>()
     }
     LineStringsWrappedType::L(lines) => {
      // lines.clone()
      // assert_eq!( 1, lines.len());
      lines
       .iter()
       .map(|x| {
        let lsw = lsw.clone();
        // Line::from( vec![ Span::styled(lsw.cursor, cursor_style), Span::styled(lsw.line_number, line_number_style)])]
        let mut vec_of_spans = vec![
         Span::styled(lsw.cursor, cursor_style),
         Span::styled(lsw.line_number, line_number_style),
        ];
        x.iter().for_each(|y| vec_of_spans.push(y.clone()));
        Line::default().spans(vec_of_spans)
       })
       .collect::<Vec<_>>()
      // vec![Line::from( vec![ Span::styled(lsw.cursor, cursor_style), Span::styled(lsw.line_number, line_number_style)])]
     }
    }
   })
   .collect::<Vec<_>>()
 }

 fn get_title(&self, is_active: bool, rest: &[&str]) -> String {
  String::from(" ")
   + if is_active { "* " } else { "  " }
   + self.title
   + &self
    .line_count
    .map_or("".to_string(), |x| format!(" l({})", x))
   + if self.wrapped { " (w)" } else { "" }
   + &(if self.hoffset == 0 { "".to_string() } else { format!(" o({})", self.hoffset) })
   + &rest.join(" ")
   + " "
 }

 fn get_block(&self, is_active: bool) -> Block<'_> {
  let border_color =
   if is_active { self.theme_colors.border } else { self.theme_colors.border_inactive };
  let border_style =
   if let Some(color) = border_color { Style::new().fg(color) } else { Style::new() };

  let bg_style = if let Some(color) = self.theme_colors.window_bg {
   Style::new().bg(color)
  } else {
   Style::new()
  };

  let block = Block::bordered()
   .title_alignment(Alignment::Left)
   .border_type(BorderType::Rounded)
   .border_style(border_style)
   .style(bg_style);
  block
 }
}

/// the TwoScreenDefaultWidget paints in the areas of the
/// rv.pl (RatatuiVariables . PagerLayout)

struct TwoScreenDefaultWidget<'a> {
 helpline: &'a str,
 rv: &'a RatatuiVariables,
 all_lines: LineStringsConfig<'a>,
 all_lines2: LineStringsConfig<'a>,
 regex_edit_mode: Option<String>,
 regex_edit_mode_state: String,
 regex_count: usize,
 delete_confirm_mode: Option<AcbeId>,
 statusline_heap: StatusLineHeap,
 paused: bool,
 active_area: ActiveArea,
 theme_colors: ThemeColors,
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
  let title = self
   .all_lines
   .get_title(is_main_active, &[regex_count_indicator]);

  let top_right_line_text = if self.paused { " PAUSED " } else { "" };
  let bottom_center_line_text = if self.paused { " PAUSED " } else { "" };

  let block = self
   .all_lines
   .get_block(is_main_active)
   .title(title)
   .title(Line::from(top_right_line_text).right_aligned())
   .title_bottom(Line::from(bottom_center_line_text).centered());

  // let rect1 = self.rv.pl.get_main_area().inner(Margin::new(0, 0));
  let rect1 = *self.rv.pl.get_main_area();
  let safe_area = rect1.intersection(area); // avoids crash

  let all_lines = self.all_lines.prepare2print(safe_area);

  // trace!( "TwoScreenDefaultWidget all_lines : {}", all_lines);

  // let paragraph = Paragraph::new(all_lines).block(block).left_aligned();
  // let paragraph : Vec<Paragraph> = all_lines
  //  .iter()
  //  .map(|all_lines| Paragraph::new(all_lines.clone()).block(block).left_aligned())
  //  .collect();

  let all_lines_flattened: Vec<Line<'_>> = all_lines.iter().flatten().cloned().collect();
  // let all_lines_flattened = all_lines.iter().flatten().collect::<Vec<Line<'_>>>();
  let paragraph = Paragraph::new(all_lines_flattened)
   .block(block)
   .left_aligned();

  // weue806j1y
  // let paragraph =
  //  if !self.all_lines.wrapped { paragraph } else { paragraph.wrap(Wrap { trim: false }) };

  let menu_style =
   if let Some(color) = self.theme_colors.menu { Style::new().fg(color) } else { Style::new() };
  Text::styled(self.helpline, menu_style).render(*self.rv.pl.get_title_area(), buf);
  paragraph.render(safe_area, buf);

  let is_second_active = self.active_area == ActiveArea::Second;

  if let Some(sma) = self.rv.pl.get_second_main_area() {
   let title2 = self.all_lines2.get_title(is_second_active, &[]);

   let block2 = self.all_lines2.get_block(is_second_active).title(title2);

   // let rect2 = sma.inner(Margin::new(0, 1));
   let rect2 = *sma;
   let safe_area2 = rect2.intersection(area); // avoids crash

   let all_lines2 = self.all_lines2.prepare2print(safe_area);

   // let paragraph2 = Paragraph::new(all_lines2).block(block2).left_aligned();
   let all_lines_flattened2: Vec<Line<'_>> = all_lines2.iter().flatten().cloned().collect();
   let paragraph2 = Paragraph::new(all_lines_flattened2)
    .block(block2)
    .left_aligned();

   // weue806j1y
   // let paragraph2 =
   //  if !self.all_lines2.wrapped { paragraph2 } else { paragraph2.wrap(Wrap { trim: false }) };

   // Clear.render(safe_area2, buf); // doesn't fix the tab problem
   paragraph2.render(safe_area2, buf);
  }
  // Paragraph::new("statusline").render( self.rv.pl.get_status_area().intersection(area), buf);
  let statusline = &self.statusline_heap;
  let status_style =
   if let Some(color) = self.theme_colors.menu { Style::new().fg(color) } else { Style::new() };
  if let Some(regex_edit_mode) = &self.regex_edit_mode {
   Paragraph::new("/".to_string() + regex_edit_mode + &self.regex_edit_mode_state + " (Esc/Enter)")
    .style(status_style)
    .render(self.rv.pl.get_status_area().intersection(area), buf);
  } else if self.delete_confirm_mode.is_some() {
   Paragraph::new("delete entry? (y/n) (Esc)")
    .style(status_style)
    .render(self.rv.pl.get_status_area().intersection(area), buf);
  } else if let Some(status_msg) = statusline.peek() {
   Paragraph::new(status_msg.text.clone() + &format!(" c({})", statusline.len()) + " (Esc)")
    .style(status_style)
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

impl PartialEq for NextTsp {
 fn eq(&self, other: &Self) -> bool {
  core::mem::discriminant(self) == core::mem::discriminant(other)
 }
}

impl Debug for NextTsp {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
  match self {
   NextTsp::NoNextTsp => write!(f, "NoNextTsp"),
   NextTsp::Replace(_) => write!(f, "Replace(...)"),
   NextTsp::Stack(_) => write!(f, "Stack(...)"),
   NextTsp::Quit => write!(f, "Quit"),
   NextTsp::PopThis => write!(f, "PopThis"),
   NextTsp::IgnoreBasicEvents => write!(f, "IgnoreBasicEvents"),
  }
 }
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

pub struct TermionScreenMenu {
 config: &'static Config,
 scroller: Scroller,
 items: Vec<&'static str>,
}

impl TermionScreenMenu {
 pub fn new(config: &'static Config) -> Self {
  Self {
   config,
   scroller: Scroller::new(),
   items: vec!["Color Theme"],
  }
 }
}

impl TermionScreenPainter for TermionScreenMenu {
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let scroller = &mut self.scroller;

  let rv = RatatuiVariables::new::<PagerLayoutBase>(terminal);

  {
   let inner_main_rect = rv.pl.get_main_area().inner(Margin::new(1, 1));

   scroller.set_content_length(self.items.len());
   scroller.set_windowlength(inner_main_rect.height as usize);

   let all_lines = render_scroller_lines4(
    scroller,
    &self.items,
    false,
    &Layout::new(),
    |cursor_star, _idx, _numbers_width, entry| LineStrings {
     wrapped: false,
     cursor: cursor_star.to_string(),
     line_number: " ".to_string(),
     text: LineStringsType::S(entry.to_string()),
    },
   );

   let theme_colors = self
    .config
    .color_theme
    .read()
    .unwrap()
    .get_colors_with_override(self.config.custom_theme_colors.read().unwrap().as_ref());

   let all_lines = LineStringsConfig {
    line_strings: all_lines.as_ref(),
    wrapped: false,
    title: "Menu",
    line_count: Some(self.items.len()),
    hoffset: 0,
    theme_colors: theme_colors.clone(),
    cursor_color: None,
   };

   {
    let window_wraps = all_lines
     .prepare2print(*rv.pl.get_main_area())
     .iter()
     .map(|x| x.len())
     .collect::<Vec<_>>();

    self.scroller.set_wrapped_window_length(&window_wraps);
   }

   let sw = TwoScreenDefaultWidget {
    helpline: constants::HELP_QXE,
    rv: &rv,
    all_lines,
    all_lines2: LineStringsConfig::default(),
    regex_edit_mode: None,
    regex_edit_mode_state: "".to_string(),
    regex_count: 0,
    delete_confirm_mode: None,
    statusline_heap: assd.statusline_heap.clone(),
    paused: false,
    active_area: ActiveArea::Main,
    theme_colors: theme_colors.clone(),
   };

   terminal
    .draw(|frame| frame.render_widget(sw, frame.area()))
    .unwrap();
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, _assd: &mut AppStateReceiverData) -> NextTsp {
  match evt {
   MyEvent::Termion(Event::Key(Key::Char('\n'))) => {
    if let Some(cursor) = self.scroller.get_cursor_in_content_array() {
     if cursor < self.items.len() && self.items[cursor] == "Color Theme" {
      return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenColorThemeChooser::new(
       self.config,
      ))));
     }
    }
   }
   _ => {
    Pager::handle_event(&mut self.scroller, evt);
   }
  }
  NextTsp::NoNextTsp
 }
}

pub struct TermionScreenColorThemeChooser {
 config: &'static Config,
 scroller: Scroller,
 themes: Vec<(String, ColorTheme)>,
 has_custom_theme: bool,
}

impl TermionScreenColorThemeChooser {
 pub fn new(config: &'static Config) -> Self {
  let themes: Vec<(String, ColorTheme)> = ColorTheme::all_themes()
   .iter()
   .map(|(name, theme)| (name.to_string(), *theme))
   .collect();
  let has_custom_theme = config.custom_theme_colors.read().unwrap().is_some();
  Self {
   config,
   scroller: Scroller::new(),
   themes,
   has_custom_theme,
  }
 }

 fn total_entries(&self) -> usize {
  if self.has_custom_theme {
   self.themes.len() + 1
  } else {
   self.themes.len()
  }
 }
}

impl TermionScreenPainter for TermionScreenColorThemeChooser {
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let rv = RatatuiVariables::new::<PagerLayoutBase>(terminal);

  {
   let inner_main_rect = rv.pl.get_main_area().inner(Margin::new(1, 1));

   let total = self.total_entries();
   self.scroller.set_content_length(total);
   self
    .scroller
    .set_windowlength(inner_main_rect.height as usize);

   let theme_colors = self
    .config
    .color_theme
    .read()
    .unwrap()
    .get_colors_with_override(self.config.custom_theme_colors.read().unwrap().as_ref());

   let cursor_in_window = self.scroller.get_cursor_in_window();
   let window_position = self.scroller.get_windowposition();

   let themes_count = self.themes.len();
   let has_custom = self.has_custom_theme;

   let mut lines: Vec<LineStrings> = Vec::new();

   for idx in 0..total {
    let is_cursor = match cursor_in_window {
     None => false,
     Some(value) => idx == window_position + value,
    };
    let cursor_star = if is_cursor { ">" } else { " " };

    let name = if has_custom && idx == themes_count {
     "custom".to_string()
    } else if idx < themes_count {
     self.themes[idx].0.clone()
    } else {
     continue;
    };

    let tc = if has_custom && idx == themes_count {
     self
      .config
      .custom_theme_colors
      .read()
      .unwrap()
      .clone()
      .unwrap_or_default()
    } else if idx < themes_count {
     self.themes[idx].1.get_colors()
    } else {
     ThemeColors::default()
    };
    let swatch = |c: Option<Color>| -> Span {
     match c {
      Some(c) => Span {
       style: Style::new().bg(c).fg(c),
       content: "██".into(),
      }, // "██" .bg(c).fg(c),
      None => Span {
       style: Style::new(),
       content: "░░".into(),
      },
     }
    };
    let swatches = vec![
     Span {
      style: Style::new(),
      content: format!(" {}", name).into(),
     },
     swatch(tc.window_bg),
     swatch(tc.window_fg),
     swatch(tc.cursor),
     swatch(tc.border),
     swatch(tc.menu),
    ];

    lines.push(LineStrings {
     wrapped: false,
     cursor: cursor_star.to_string(),
     line_number: "".to_string(),
     //  text: format!("{}   {}", name, swatches),
     text: LineStringsType::L(vec![Line::default().spans(swatches)]),
    });
   }

   let all_lines = LineStringsConfig {
    line_strings: &lines,
    wrapped: false,
    title: "Color Theme",
    line_count: Some(total),
    hoffset: 0,
    theme_colors: theme_colors.clone(),
    cursor_color: None,
   };

   {
    let window_wraps = all_lines
     .prepare2print(*rv.pl.get_main_area())
     .iter()
     .map(|x| x.len())
     .collect::<Vec<_>>();

    self.scroller.set_wrapped_window_length(&window_wraps);
   }

   let sw = TwoScreenDefaultWidget {
    helpline: constants::HELP_QXE,
    rv: &rv,
    all_lines,
    all_lines2: LineStringsConfig::default(),
    regex_edit_mode: None,
    regex_edit_mode_state: "".to_string(),
    regex_count: 0,
    delete_confirm_mode: None,
    statusline_heap: assd.statusline_heap.clone(),
    paused: false,
    active_area: ActiveArea::Main,
    theme_colors: theme_colors.clone(),
   };

   terminal
    .draw(|frame| frame.render_widget(sw, frame.area()))
    .unwrap();
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
  match evt {
   MyEvent::Termion(Event::Key(Key::Char('\n'))) => {
    if let Some(cursor) = self.scroller.get_cursor_in_content_array() {
     if cursor < self.themes.len() {
      let theme_name = self.themes[cursor].0.clone();
      let theme = self.themes[cursor].1;
      let mut color_theme = self.config.color_theme.write().unwrap();
      *color_theme = theme;
      drop(color_theme);
      let mut custom = self.config.custom_theme_colors.write().unwrap();
      *custom = None;
      drop(custom);
      assd
       .statusline_heap
       .push(StatusSeverity::Info, format!("Theme changed to {}", theme_name));
     } else if self.has_custom_theme && cursor == self.themes.len() {
      let mut color_theme = self.config.color_theme.write().unwrap();
      *color_theme = ColorTheme::Default;
      drop(color_theme);
      assd
       .statusline_heap
       .push(StatusSeverity::Info, "Theme changed to custom".to_string());
     }
     return NextTsp::NoNextTsp;
    }
   }
   _ => {
    Pager::handle_event(&mut self.scroller, evt);
   }
  }
  NextTsp::NoNextTsp
 }
}

pub struct TermionScreenFirstPage {
 config: &'static Config,
 // scroller_main: WrapScroller,
 scroller_main: Scroller,
 // scroller_second: Scroller,
 layout: Layout,
 flipstate: u8,
 wrapped: bool,
 paused: bool,
 regex_edit_mode: Option<String>,
 regex_edit_mode_state: String,
 regex_edit_mode_last_working: Option<Regex>,
 regex: Vec<Regex>,
 regex_filtered_cbs_entries: VecDeque<FilteredCbsEntries>,
 delete_confirm_mode: Option<AcbeId>,
 active_area: ActiveArea,
 main_width: usize,
 second_width: usize,
 prev_selected_text: Option<Vec<u8>>,
 needs_refilter: bool,
 last_entry_count: usize,
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
   // scroller_main: WrapScroller::default(),
   scroller_main: Scroller::default(),
   // scroller_second: Scroller::new(),
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
   needs_refilter: true,
   last_entry_count: 0,
  }
 }

 fn flipstate_next(&mut self) {
  self.flipstate = (self.flipstate + 1) % 3;
 }
 fn flipstate_prev(&mut self) {
  self.flipstate = (self.flipstate + 2) % 3;
 }

 fn update_filtered_entries(&mut self, cbs: &mut crate::clipboards::Clipboards) {
  if !self.needs_refilter {
   return;
  }
  trace!("update_filtered_entries");
  let entries = cbs.get_cbentries();

  // gtewxxi8oh
  self.regex_filtered_cbs_entries = entries
   .values()
   .rev()
   .filter_map(|line| {
    let mut res = true;
    let mut r = self.regex.clone();
    r.extend(self.regex_edit_mode_last_working.iter().cloned());
    for r in r {
     if !r.is_match(&line.cbentry.borrow().as_string()) {
      res = false;
      break;
     }
    }
    match res {
     true => Some(FilteredCbsEntries::ACE(line.clone())),
     false => None,
    }
   })
   .collect::<VecDeque<_>>();

  {
   let cbtype_enum_vector: Vec<CBType> = all::<CBType>().collect::<Vec<_>>();
   let mut last_entries = cbtype_enum_vector
    .iter()
    .map(|x| cbs.get_last_entries().get(x))
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

  self.last_entry_count = cbs.get_cbentries().len();
  self.needs_refilter = false;
 }

 fn get_max_hoffset_main(&self, _cbs: &crate::clipboards::Clipboards) -> usize {
  if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
   let entries = &self.regex_filtered_cbs_entries;
   if cursor < entries.len() {
    if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
     return tabfix(&flatline(&acbe.cbentry.borrow().as_string())).width();
    }
   }
  }
  0
 }

 fn get_max_hoffset_second(&self, _cbs: &crate::clipboards::Clipboards) -> usize {
  if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
   let entries = &self.regex_filtered_cbs_entries;
   if cursor < entries.len() {
    if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
     let cbentry_borrowed = acbe.cbentry.borrow();
     let scroller_second = cbentry_borrowed.get_scroller();
     if let Some(cursor_second) = scroller_second.get_cursor_in_content_array() {
      let lines = cbentry_borrowed.get_text();
      // return lines.iter().map(|l| tabfix(l).width()).max().unwrap_or(0);
      if cursor_second < lines.len() {
       return tabfix(&lines[cursor_second]).width();
      }
     }
    }
   }
  }
  0
 }

 fn toggle_active_area(&mut self) {
  self.active_area = match self.active_area {
   ActiveArea::Main => ActiveArea::Second,
   ActiveArea::Second => ActiveArea::Main,
  };
 }

 fn get_current_entry(&self) -> Option<RefMut<'_, CBEntry>> {
  if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
   let entries = &self.regex_filtered_cbs_entries;
   if cursor < entries.len() {
    if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
     let cbentry_borrowed = acbe.cbentry.borrow_mut();
     return Some(cbentry_borrowed);
    }
   }
  }
  None
 }

 // fn get_active_scroller<'a>(
 //  &'a mut self,
 //  cbe: Option<&'a mut CBEntry>,
 // ) -> Option<&'a mut Scroller> {
 //  match self.active_area {
 //   ActiveArea::Main => Some(&mut self.scroller_main),
 //   ActiveArea::Second => cbe.map(|x| x.get_scroller_mut()),
 //  }
 // }
}

impl TermionScreenPainter for TermionScreenFirstPage {
 /// the paint method opens a TwoScreenDefaultWidget which is later painted
 /// by the terminal.draw method
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let cbs = &mut assd.cbs;

  if cbs.get_cbentries().len() != self.last_entry_count {
   self.needs_refilter = true;
  }
  trace!("needs_refilter {}", self.needs_refilter);
  // self.needs_refilter = false;
  self.update_filtered_entries(cbs);
  // return;

  let layout = &mut self.layout;

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

   // cawxd8rc8j 0%
   {
    let entries = &self.regex_filtered_cbs_entries;

    // let mut selected_string = Vec::<u8>::new();
    // let mut selected_lines = &Vec::<String>::new();
    let mut selected_cbentry: Option<Rc<RefCell<CBEntry>>> = None;
    let mut line_count2 = None;

    if self.config.debug {
     trace!("scroller.set_content_length(entries.len()) : {}", entries.len());
    }
    self.scroller_main.set_hwindowlength(self.main_width);

    // etzwepgkfl
    // self.scroller_second.set_hwindowlength(self.second_width);

    self.scroller_main.set_content_length(entries.len());

    // scroller.set_windowlength(height + 1 - layout.get_current_line());

    self
     .scroller_main
     .set_windowlength(inner_main_rect.height as usize);

    let second_area_height = inner_second_rect.map_or(0, |x| x.height as usize);

    // etzwepgkfl
    // self.scroller_second.set_windowlength(second_area_height);

    let numbers_width = (entries.len() as f64).log10().ceil() as usize;

    if self.config.debug {
     trace!("scroller.get_safe_windowrange() : {:?}", self.scroller_main.get_safe_windowrange());
    }

    // iwcqjc9i11 Example for the line selection
    // cawxd8rc8j 40%

    let mut lines = vec![];

    for (idx, entry) in entries
     .range(self.scroller_main.get_safe_windowrange())
     .enumerate()
    {
     if let FilteredCbsEntries::ACE(appended_cbentry) = entry {
      let mut bm = appended_cbentry.cbentry.borrow_mut();
      let scroller_mut = bm.get_scroller_mut();
      // etzwepgkfl
      scroller_mut.set_hwindowlength(self.second_width);
      // etzwepgkfl
      scroller_mut.set_windowlength(second_area_height);
     }

     // if &FilteredCbsEntries::ACE( entry) = entry {
     //  let bm = entry.borrow_mut();
     //  bm.
     // }

     let is_cursor = match self.scroller_main.get_cursor_in_window() {
      None => false,
      Some(value) => idx == value,
     };

     let cursor_star = if is_cursor { ">" } else { " " };

     match entry {
      FilteredCbsEntries::ACE(acbe) => {
       let cbentry = &acbe.cbentry;
       // let is_selected = entry.is_selected(cbs);
       let is_selected = cbs.is_fixated(cbentry);

       let selection_star = if is_selected { "*" } else { " " };

       let cbentry_borrowed = cbentry.borrow_mut();

       if is_cursor {
        if self.prev_selected_text.as_ref() != Some(cbentry_borrowed.get_data()) {
         // etzwepgkfl

         // self.scroller_second.reset_hoffset();
         // cbentry_borrowed.get_scroller_mut().reset_hoffset();
         self.prev_selected_text = Some(cbentry_borrowed.get_data().clone());
        }
        // selected_string = cbentry.data.clone();
        // selected_lines = cbentry_borrowed.get_text();
        selected_cbentry = Some(Rc::clone(cbentry));
        let _ = line_count2.insert(cbentry_borrowed.get_text().len());
       }

       {
        // let s002 = format!(
        //  "{} {} {:width$} {} {} : {}",
        //  cursor_star,
        //  selection_star,
        //  idx + self.scroller_main.get_windowposition(), // mqbojcmkot
        //  cbentry_borrowed.get_cbtype().get_info(),
        //  cbentry_borrowed.get_date_time(),
        //  // cbentry_borrowed.as_string(),
        //  "",
        // width = numbers_width,
        // );
        // // lines.push(layout.fixline(&s002));
        // // lines.push(flatline(&s002));
        // lines.push((flatline(&s002), flatline(&cbentry_borrowed.as_string().into_owned())));

        lines.push(LineStrings {
         wrapped: false,
         cursor: cursor_star.to_string(),
         line_number: format!(
          " {} {:width$} {} {} : ",
          selection_star,
          idx + self.scroller_main.get_windowposition(), // mqbojcmkot
          cbentry_borrowed.get_cbtype().get_info(),
          cbentry_borrowed.get_date_time(),
          width = numbers_width,
         ),
         text: LineStringsType::S(flatline(&cbentry_borrowed.as_string())),
        });
       }
      }
      FilteredCbsEntries::Line => {
       //  lines.push((layout.centerline("----- ↑ active ↑ ----- ↓ incoming ↓ -----"), "".to_string()));
       lines.push(LineStrings {
        wrapped: false,
        cursor: cursor_star.to_string(),
        line_number: layout
         .centerline("----- ↑ active ↑ ----- ↓ incoming ↓ -----")
         .to_string(),
        text: LineStringsType::S("".to_string()),
       });
      }
      FilteredCbsEntries::Empty => {
       //  lines.push(("".into(), "".into()));
       lines.push(LineStrings {
        wrapped: false,
        cursor: cursor_star.to_string(),
        line_number: "".to_string(),
        text: LineStringsType::S("".to_string()),
       });
      }
     }
    }

    // let all_lines = lines.join("\n");
    let all_lines = lines;

    // etzwepgkfl
    let mut hoffset_second: usize = 0;
    // cawxd8rc8j 40%
    let all_lines2 = {
     let string_lines = match &selected_cbentry {
      Some(rc) => rc.borrow().get_text().clone(),
      None => vec![],
     };
     // etzwepgkfl
     // self.scroller_second.set_content_length(string_lines.len());

     let mut bm;
     let scroller_second = if let Some(selected_cbentry) = selected_cbentry.as_ref() {
      bm = selected_cbentry.borrow_mut();
      let sm = bm.get_scroller_mut();
      sm.set_content_length(string_lines.len());
      hoffset_second = sm.get_hoffset();
      sm
     } else {
      &mut Scroller::default()
     };

     render_scroller_lines4(
      // &mut self.scroller_second,
      scroller_second,
      &string_lines,
      self.wrapped,
      layout,
      |cursor_star, idx, numbers_width, entry| {
       //  (format!("{} {:width$} : ", cursor_star, idx, width = numbers_width,), entry.to_string())
       LineStrings {
        wrapped: self.wrapped,
        cursor: cursor_star.to_string(),
        line_number: format!(" {:width$} : ", idx, width = numbers_width,),
        text: LineStringsType::S(entry.to_string()),
       }
      },
     )
    };
    // cawxd8rc8j 50%

    // wrap simulation gqhdbjurhn :
    // let all_lines = all_lines
    //  .iter()
    //  .flat_map(|x| vec![(*x).clone(), (*x).clone()])
    //  .collect::<Vec<LineStrings>>();

    let theme_colors = self
     .config
     .color_theme
     .read()
     .unwrap()
     .get_colors_with_override(self.config.custom_theme_colors.read().unwrap().as_ref());

    let all_lines = LineStringsConfig {
     line_strings: &all_lines,
     wrapped: false,
     title: "entry list",
     line_count: Some(entries.len()),
     hoffset: self.scroller_main.get_hoffset(),
     theme_colors: theme_colors.clone(),
     cursor_color: if self.active_area == ActiveArea::Second {
      theme_colors.cursor_inactive
     } else {
      None
     },
    };

    {
     let window_wraps = all_lines
      .prepare2print(*rv.pl.get_main_area())
      .iter()
      .map(|x| x.len())
      .collect::<Vec<_>>();

     self.scroller_main.set_wrapped_window_length(&window_wraps);
    }

    let all_lines2 = LineStringsConfig {
     line_strings: &all_lines2,
     wrapped: self.wrapped,
     title: "selected content",
     line_count: line_count2,
     // etzwepgkfl
     // hoffset: self.scroller_second.get_hoffset(),
     hoffset: hoffset_second,
     theme_colors: theme_colors.clone(),
     cursor_color: if self.active_area == ActiveArea::Main {
      theme_colors.cursor_inactive
     } else {
      None
     },
    };

    if let Some(second_main_area) = rv.pl.get_second_main_area() {
     let window_wraps = all_lines2
      .prepare2print(*second_main_area)
      .iter()
      .map(|x| x.len())
      .collect::<Vec<_>>();

     let mut bm;
     let scroller_second = if let Some(selected_cbentry) = selected_cbentry.as_ref() {
      bm = selected_cbentry.borrow_mut();
      let sm = bm.get_scroller_mut();
      Some(sm)
     } else {
      None
     };
     // let mut bm = selected_cbentry.map( |x| x.borrow_mut());
     // self.scroller_main.set_wrapped_window_length(&window_wraps);
     if let Some(x) = scroller_second {
      x.set_wrapped_window_length(&window_wraps)
     }
    }

    let sw = TwoScreenDefaultWidget {
     helpline: HELP_FIRST_PAGE,
     rv,
     // tsfp: &self,
     all_lines,
     all_lines2,
     regex_edit_mode: self.regex_edit_mode.clone(),
     regex_edit_mode_state: self.regex_edit_mode_state.clone(),
     regex_count: self.regex.len() + self.regex_edit_mode.is_some() as usize,
     delete_confirm_mode: self.delete_confirm_mode,
     statusline_heap: assd.statusline_heap.clone(),
     paused: self.config.paused.is_paused(),
     active_area: self.active_area,
     theme_colors: theme_colors.clone(),
    };

    terminal
     .draw(|frame| frame.render_widget(sw, frame.area()))
     .unwrap();
   }
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
  let cbs = &mut assd.cbs;

  if evt == &MyEvent::CbInserted {
   self.needs_refilter = true;
   return NextTsp::NoNextTsp;
  }

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
   self.needs_refilter = true;
   return NextTsp::IgnoreBasicEvents;
  } else if self.delete_confirm_mode.is_some() {
   match evt {
    MyEvent::Termion(Event::Key(Key::Esc)) => {
     self.delete_confirm_mode = None;
    }
    MyEvent::Termion(Event::Key(Key::Char('y'))) => {
     if let Some(id) = self.delete_confirm_mode {
      cbs.remove_by_seq(id);
      self.delete_confirm_mode = None;
      self.needs_refilter = true;
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
     self.needs_refilter = true;
    }
    //  MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
    MyEvent::Termion(Event::Key(Key::Char('h'))) => {
     return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenViewPage::new(
      self.config,
      "help".to_string(),
      CBEntry::new(config::USAGE.to_string().as_bytes()),
     ))));
    }
    MyEvent::Termion(Event::Key(Key::Char('m'))) => {
     return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenMenu::new(self.config))));
    }
    MyEvent::Termion(Event::Key(Key::Char('f'))) => {
     self.flipstate_next();
    }
    MyEvent::Termion(Event::Key(Key::Char('F'))) => {
     self.flipstate_prev();
    }
    MyEvent::Termion(Event::Key(Key::Char('s'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
      let entries = &self.regex_filtered_cbs_entries;
      if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
       cbs.toggle_fixation(&(*acbe).clone());
      }
      self.needs_refilter = true;
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('t'))) => {
     cbs.toggle_clipboards();
    }
    MyEvent::Termion(Event::Key(Key::Char('v'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
      let entries = &self.regex_filtered_cbs_entries;
      if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
       return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenViewPage::new(
        self.config,
        "view entry".to_string(),
        acbe.cbentry.borrow().clone(),
       ))));
      };
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('e'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
      let entries = &self.regex_filtered_cbs_entries;
      // let entry = &entries[cursor];

      if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
       match TermionScreenEditorPage::new(
        self.config,
        acbe.cbentry.borrow().as_string().into_owned(),
        acbe.id,
       ) {
        Ok(page) => return NextTsp::Stack(Rc::new(RefCell::new(page))),
        Err(e) => {
         eprintln!("Failed to create editor page: {}", e);
         assd
          .statusline_heap
          .push(StatusSeverity::Warning, format!("Failed to create editor page: {}", e));
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
     if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
      let entries = &self.regex_filtered_cbs_entries;
      if entries.is_empty() {
      } else if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
       match assd.cbs.get_cbentries().get(&acbe.id) {
        Some(_) => self.delete_confirm_mode = Some(acbe.id),
        None => assd
         .statusline_heap
         .push(StatusSeverity::Info, "not deletable".to_string()),
       }
      }
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('p'))) => {
     assd.sender.send(MyEvent::TogglePause).unwrap();
    }
    MyEvent::Termion(Event::Key(Key::Char('/'))) => {
     self.regex_edit_mode = Some("".to_string());
     self.needs_refilter = true;
    }
    _ => {
     // TODO : optimize
     let max_offset = match self.active_area {
      ActiveArea::Main => self.get_max_hoffset_main(cbs),
      ActiveArea::Second => self.get_max_hoffset_second(cbs),
     };

     match self.active_area {
      ActiveArea::Main => {
       self.scroller_main.set_max_hoffset(max_offset);
       Pager::handle_event(&mut self.scroller_main, evt);
      }

      ActiveArea::Second => {
       let mut current_entry = self.get_current_entry();
       let current_entry = current_entry.as_deref_mut().map(|x| x.get_scroller_mut());
       current_entry.map(|x| {
        x.set_max_hoffset(max_offset);
        Pager::handle_event(x, evt);
       });
      }
     };
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
 entry_id: AcbeId,
}

impl TermionScreenEditorPage {
 pub fn new(config: &'static Config, text: String, entry_id: AcbeId) -> Result<Self, String> {
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
   let all_lines = render_scroller_lines4(
    scroller,
    &string_lines,
    self.wrapped,
    layout,
    |cursor_star, idx, numbers_width, entry| {
     // format!("{} {:width$} : {}", cursor_star, idx, entry, width = numbers_width,)
     //  (format!("{} {:width$} : ", cursor_star, idx, width = numbers_width,), entry.to_string())
     LineStrings {
      wrapped: self.wrapped,
      cursor: cursor_star.to_string(),
      line_number: format!(" {:width$} : ", idx, width = numbers_width,),
      text: LineStringsType::S(entry.to_string()),
     }
    },
   );
   // for R::VS
   // let all_lines = all_lines.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
   // for R::Old
   // let all_lines = all_lines.join( "\n");

   let theme_colors = self
    .config
    .color_theme
    .read()
    .unwrap()
    .get_colors_with_override(self.config.custom_theme_colors.read().unwrap().as_ref());

   let all_lines = LineStringsConfig {
    line_strings: all_lines.as_ref(),
    wrapped: self.wrapped,
    title: &self.main_title,
    line_count: Some(string_lines.len()),
    hoffset: self.scroller.get_hoffset(),
    theme_colors: theme_colors.clone(),
    cursor_color: None,
   };

   {
    let window_wraps = all_lines
     .prepare2print(*rv.pl.get_main_area())
     .iter()
     .map(|x| x.len())
     .collect::<Vec<_>>();

    self.scroller.set_wrapped_window_length(&window_wraps);
   }

   let sw = TwoScreenDefaultWidget {
    helpline: HELP_WQX,
    rv: &rv,
    // all_lines: R::Old(&all_lines),
    // all_lines: LineStringsConfig::New2(all_lines.as_ref())
    all_lines,
    // all_lines: LineStringsConfig {
    //  line_strings: all_lines.as_ref(),
    //  wrapped: self.wrapped,
    //  title: &self.main_title,
    //  line_count: Some(string_lines.len()),
    //  hoffset: self.scroller.get_hoffset(),
    //  theme_colors: theme_colors.clone(),
    //  cursor_color: None,
    // },
    all_lines2: LineStringsConfig::default(),
    regex_edit_mode: None,
    regex_edit_mode_state: "".to_string(),
    regex_count: 0,
    delete_confirm_mode: None,
    statusline_heap: assd.statusline_heap.clone(),
    paused: false,
    active_area: ActiveArea::Main,
    theme_colors: theme_colors.clone(),
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
