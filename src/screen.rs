// #![allow(dead_code)]
// #![allow(unused)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::constants::{self};
use crate::event::MyEvent;
use crate::layout::Layout;
use crate::layout_ratatui::PagerLayout;
use crate::scroller::Scroller;
use crate::tools::tabfix;
use crate::{
 clipboards::AcbeId,
 color_theme::ThemeColors,
 libmain::{AppStateReceiverData, StatusLineHeap},
};

use menu::ScreenMenu;
use ratatui::layout::{Alignment, Rect};
use ratatui::{
 style::Style,
 text::{Line, Span, Text},
 widgets::{Block, BorderType, Paragraph, Widget},
 DefaultTerminal,
};

use unicode_width::UnicodeWidthChar; // extends char by width, width_cjk
use unicode_width::UnicodeWidthStr; // extends &str by width, width_cjk

use std::fmt::Debug;
// write_all

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
 use first_page::ScreenFirstPage;
 use termion::event::{Event, Key};

 use super::*;
 use crate::clipboards::CBType;
 use crate::config::Config;
use crate::event::MyEvent;
 use crate::libmain::AppStateReceiverData;
 use std::sync::mpsc::channel;

 #[test]
 fn test_cb_inserted_sets_needs_refilter() {
  let (sender, _receiver) = channel();
  let config = Box::leak(Box::new(Config::default()));
  let mut assd = AppStateReceiverData::new(config, sender);
  assd
   .cbs
   .insert(&CBType::Clipboard, Some(b"test data".to_vec()));

  let mut screen = ScreenFirstPage::new(config);

  let next = screen.handle_event(&MyEvent::CbInserted, &mut assd);
  assert!(screen.needs_refilter);
  assert_eq!(next, NextTsp::NoNextTsp);
 }

 #[test]
 fn test_cb_changed_updates_clipboard_and_refilters() {
  let (sender, _receiver) = channel();
  let config = Box::leak(Box::new(Config::default()));
  let mut assd = AppStateReceiverData::new(config, sender);

  let mut screen = ScreenFirstPage::new(config);

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

  let mut screen = ScreenFirstPage::new(config);
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
 fn tabfix(&self, hoffset: usize, safe_area: Rect) -> LineStringsWrapped<'_> {
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

   LineStringsType::L(lines) => LineStringsWrappedType::L(lines.to_vec()),
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

pub(crate) enum NextTsp {
 NoNextTsp,
 Replace(Rc<RefCell<dyn ScreenPainter>>),
 Stack(Rc<RefCell<dyn ScreenPainter>>),
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

pub(crate) trait ScreenPainter {
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

pub(crate) use self::statusbardialog_y_n::ScreenStatusBarDialogYN;

mod statusbardialog_y_n;

mod menu;

mod color_theme_chooser;

pub(crate) use self::first_page::ScreenFirstPage;

mod first_page;

pub(crate) use self::editor_page::ScreenEditorPage;

mod editor_page;

pub(crate) use self::view_page::ScreenViewPage;

mod view_page;
