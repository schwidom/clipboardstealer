#![allow(dead_code)]
#![allow(unused)]

use std::cell::RefCell;
use std::cmp::min;
use std::io::Stdout;
use std::rc::Rc;

use crate::config::{self, Config};
use crate::constants::{HELP_FIRST_PAGE, HELP_QX};
use crate::event::MyEvent;
use crate::layout::Layout;
use crate::layout_ratatui::{PagerLayout, PagerLayoutBase, PagerLayoutLR, PagerLayoutTB};
// use crate::libmain::SyncStuff;
use crate::libmain::AppStateReceiverData;
use crate::pager::Pager;
use crate::scroller::Scroller;
use crate::tools::tabfix;

use ratatui::layout::{Alignment, Margin};
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

struct TwoScreenDefaultWidget<'a> {
 helpline: &'a str,
 main_title: &'a str,
 second_title: &'a str,
 rv: &'a RatatuiVariables,
 all_lines: &'a str,
 all_lines2: &'a str,
}

impl<'a> Widget for TwoScreenDefaultWidget<'a> {
 fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
 where
  Self: Sized,
 {
  let block = Block::bordered()
   .title(self.main_title)
   .title_alignment(Alignment::Left)
   // .style(Style::new().fg(Color::Blue))
   .border_type(BorderType::Rounded);

  // let rect1 = self.rv.pl.get_main_area().inner(Margin::new(0, 0));
  let rect1 = *self.rv.pl.get_main_area();
  let safe_area = rect1.intersection(buf.area); // avoids crash

  let all_lines = trim_text_to_rect_with(self.all_lines, safe_area);
  let all_lines = tabfix(&all_lines);

  let paragraph = Paragraph::new(all_lines)
   .block(block)
   // .fg(Color::Cyan)
   // .bg(Color::Black)
   .left_aligned();

  Text::raw(self.helpline).render(*self.rv.pl.get_title_area(), buf);
  // Text::raw(self.all_lines).render(self.rv.pl.main_area.inner(Margin::new(0, 1)), buf);
  // block.render(self.rv.pl.main_area.inner(Margin::new(0, 1)), buf);
  // paragraph.render(rect1, buf);
  // Clear.render(safe_area, buf); // doesn't fix the tab problem
  paragraph.render(safe_area, buf);

  if let Some(sma) = self.rv.pl.get_second_main_area() {
   let block2 = Block::bordered()
    .title(self.second_title)
    .title_alignment(Alignment::Left)
    .border_type(BorderType::Rounded);

   // let rect2 = sma.inner(Margin::new(0, 1));
   let rect2 = *sma;
   let safe_area2 = rect2.intersection(buf.area); // avoids crash
   let all_lines2 = trim_text_to_rect_with(self.all_lines2, safe_area2);
   let all_lines2 = tabfix(&all_lines2);

   let paragraph2 = Paragraph::new(all_lines2)
    .block(block2)
    // .fg(Color::Cyan)
    // .bg(Color::Black)
    .left_aligned();

   // Clear.render(safe_area2, buf); // doesn't fix the tab problem
   paragraph2.render(safe_area2, buf);
  }
 }
}

pub struct TermionScreenFirstPage {
 config: &'static Config,
 scroller: Scroller,
 layout: Layout,
 flipstate: u8,
}

pub enum NextTsp {
 NoNextTsp,
 Replace(Rc<RefCell<dyn TermionScreenPainter>>),
 Stack(Rc<RefCell<dyn TermionScreenPainter>>),
}

pub trait TermionScreenPainter {
 // fn new(config: &'static Config) -> Self
 // where
 //  Self: Sized;
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData);
 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp;
}

// TODO : mode in the vicinity of first_page() definition (maybe inside)
impl TermionScreenFirstPage {
 pub fn new(config: &'static Config) -> Self {
  Self {
   config,
   scroller: Scroller::new(),
   layout: Layout::new(),
   flipstate: 1,
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

    let mut selected_string = &String::default();

    scroller.set_content_length(Some(entries.len()));
    // scroller.set_windowlength(height + 1 - layout.get_current_line());
    scroller.set_windowlength(rv.pl.get_main_area().inner(Margin::new(0, 1)).height);

    let numbers_width = (entries.len() as f64).log10().ceil() as usize;

    for (idx, entry) in entries.range(scroller.get_windowrange()).enumerate() {
     let is_cursor = match scroller.get_cursor() {
      None => false,
      Some(value) => idx as u16 == value,
     };

     let cursor_star = if is_cursor { ">" } else { " " };

     // let is_selected = entry.is_selected(cbs);
     let is_selected = cbs.is_fixated(entry);

     let selection_star = if is_selected { "*" } else { " " };

     if is_cursor {
      selected_string = &entry.text;
     }

     {
      let s002 = format!(
       "{} {} {:width$} {} {} : {}",
       cursor_star,
       selection_star,
       idx + scroller.get_windowposition(), // mqbojcmkot
       entry.cbtype.get_info(),
       entry.get_date_time(),
       entry.text,
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
    };

    terminal.draw(|frame| frame.render_widget(sw, frame.area()));
   }
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
  let mut cbs = &mut assd.cbs;

  match evt {
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
     let entries = cbs.get_entries();
     let entry = &entries[cursor].clone(); // NOTE: the clone can maybe avoided when I put this logic into cbs
                                           // entry.toggle_selection(&mut cbs);
     cbs.toggle_selection(entry);
    }
   }
   MyEvent::Termion(Event::Key(Key::Char('v'))) => {
    if let Some(cursor) = self.scroller.get_cursor_in_array() {
     let entries = cbs.get_entries();
     let entry = &entries[cursor];
     return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenViewPage::new(
      self.config,
      "view entry".to_string(),
      entry.text.clone(),
     ))));
    }
   }
   _ => {
    // Pager::handle_event(&mut scroller, &evt);
    Pager::handle_event(&mut self.scroller, evt);
   }
  }
  NextTsp::NoNextTsp
 }
}

pub struct TermionScreenViewPage {
 config: &'static Config,
 main_title: String,
 scroller: Scroller,
 layout: Layout,
 text: String,
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
  }
 }
}

impl TermionScreenPainter for TermionScreenViewPage {
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let scroller = &mut self.scroller;
  let layout = &mut self.layout;

  let string_lines = self.text.split("\n").collect::<Vec<_>>();

  let mut rv = RatatuiVariables::new::<PagerLayoutBase>(terminal);

  {
   let (width, height) = termion::terminal_size().unwrap();
   layout.set_width_height(width, height);

   let mut lines = vec![];

   {
    scroller.set_content_length(Some(string_lines.len()));
    // scroller.set_windowlength(height + 1 - layout.get_current_line());
    scroller.set_windowlength(rv.pl.get_main_area().inner(Margin::new(0, 1)).height);

    let numbers_width = (string_lines.len() as f64).log10().ceil() as usize;

    for (idx, entry) in string_lines[scroller.get_windowrange()].iter().enumerate() {
     let is_cursor = match scroller.get_cursor() {
      None => false,
      Some(value) => idx as u16 == value,
     };

     let cursor_star = if is_cursor { ">" } else { " " };

     let s002 = format!(
      "{} {:width$} : {}",
      cursor_star,
      idx + scroller.get_windowposition(), // mqbojcmkot
      entry,
      width = numbers_width,
     );

     lines.push(layout.fixline(&s002));

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
   };

   terminal.draw(|frame| frame.render_widget(sw, frame.area()));
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, _assd: &mut AppStateReceiverData) -> NextTsp {
  match evt {
   //  MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
   _ => {
    // Pager::handle_event(&mut scroller, &evt);
    Pager::handle_event(&mut self.scroller, evt);
   }
  }
  NextTsp::NoNextTsp
 }
}
