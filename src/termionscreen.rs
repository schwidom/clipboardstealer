#![allow(dead_code)]
#![allow(unused)]

use std::cell::RefCell;
use std::io::Stdout;
use std::rc::Rc;

use crate::config::{self, Config};
use crate::constants::{HELP_FIRST_PAGE, HELP_QX};
use crate::event::MyEvent;
use crate::layout::Layout;
use crate::layout_ratatui::{FrameNew, PagerLayout};
// use crate::libmain::SyncStuff;
use crate::libmain::AppStateReceiverData;
use crate::pager::Pager;
use crate::scroller::Scroller;

use ratatui::layout::Margin;
use ratatui::prelude::CrosstermBackend;
use ratatui::text::Text;
use ratatui::Terminal;
use termion::event::{Event, Key};
use termion::{self, scroll};

use tracing::{event, info, span, trace, Instrument, Level};

// use num::Integer;

struct RatatuiVariables<T> {
 terminal: Terminal<CrosstermBackend<Stdout>>,
 pl: T,
}

impl<T: FrameNew> RatatuiVariables<T> {
 fn new() -> Self {
  let mut terminal = ratatui::init();
  let pl = T::new(&terminal.get_frame());
  Self { terminal, pl }
 }
}

pub struct TermionScreenFirstPage {
 config: &'static Config,
 scroller: Scroller,
 layout: Layout,
 is_folded: bool,
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
 fn paint(&mut self, assd: &mut AppStateReceiverData);
 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp;
}

// TODO : mode in the vicinity of first_page() definition (maybe inside)
impl TermionScreenFirstPage {
 pub fn new(config: &'static Config) -> Self {
  Self {
   config,
   scroller: Scroller::new(),
   layout: Layout::new(),
   is_folded: false,
  }
 }
}

impl TermionScreenPainter for TermionScreenFirstPage {
 fn paint(&mut self, assd: &mut AppStateReceiverData) {
  let scroller = &mut self.scroller;
  let layout = &mut self.layout;

  let cbs = &mut assd.cbs;

  let mut rv = RatatuiVariables::<PagerLayout>::new();

  {
   let (width, height) = termion::terminal_size().unwrap();
   layout.set_width_height(width, height);

   let mut lines = vec![];

   {
    let entries = cbs.get_entries();

    scroller.set_content_length(Some(entries.len()));
    // scroller.set_windowlength(height + 1 - layout.get_current_line());
    scroller.set_windowlength(rv.pl.main_area.inner(Margin::new(0, 1)).height);

    let numbers_width = (entries.len() as f64).log10().ceil() as usize;

    for (idx, entry) in entries[scroller.get_windowrange()].iter().enumerate() {
     let is_cursor = match scroller.get_cursor() {
      None => false,
      Some(value) => idx as u16 == value,
     };

     let cursor_star = if is_cursor { ">" } else { " " };

     let is_selected = entry.is_selected(cbs);

     let selection_star = if is_selected { "*" } else { " " };

     // layout.print_line_cut(&s002);
     if is_cursor && !self.is_folded {
      let s002 = format!(
       "{} {} {:width$} {} {} : \n{}",
       cursor_star,
       selection_star,
       idx + scroller.get_windowposition(), // mqbojcmkot
       entry.info,
       entry.get_date_time(),
       entry.string,
       width = numbers_width,
      );
      lines.push(s002);
     } else {
      let s002 = format!(
       "{} {} {:width$} {} {} : {}",
       cursor_star,
       selection_star,
       idx + scroller.get_windowposition(), // mqbojcmkot
       entry.info,
       entry.get_date_time(),
       entry.string,
       width = numbers_width,
      );
      lines.push(layout.fixline(&s002));
     }
    }
   }

   let all_lines = lines.join("\n");

   rv
    .terminal
    .draw(|frame| {
     frame.render_widget(Text::raw(HELP_FIRST_PAGE), rv.pl.title_area);
     frame.render_widget(Text::raw(all_lines), rv.pl.main_area.inner(Margin::new(0, 1)));
    })
    .unwrap();
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
  let mut cbs = &mut assd.cbs;

  match evt {
   //  MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
   MyEvent::Termion(Event::Key(Key::Char('h'))) => {
    return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenViewPage::new(
     self.config,
     config::USAGE.to_string(),
    ))));
   }
   MyEvent::Termion(Event::Key(Key::Char('f'))) => {
    self.is_folded = !self.is_folded;
   }
   MyEvent::Termion(Event::Key(Key::Char('s'))) => {
    if let Some(cursor) = self.scroller.get_cursor_in_array() {
     let entries = cbs.get_entries();
     let entry = &entries[cursor];
     entry.toggle_selection(&mut cbs);
    }
   }
   MyEvent::Termion(Event::Key(Key::Char('v'))) => {
    if let Some(cursor) = self.scroller.get_cursor_in_array() {
     let entries = cbs.get_entries();
     let entry = &entries[cursor];
     return NextTsp::Stack(Rc::new(RefCell::new(TermionScreenViewPage::new(
      self.config,
      entry.string.clone(),
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
 scroller: Scroller,
 layout: Layout,
 text: String,
}

impl TermionScreenViewPage {
 fn new(config: &'static Config, text: String) -> Self
 where
  Self: Sized,
 {
  Self {
   config,
   scroller: Scroller::new(),
   layout: Layout::new(),
   text,
  }
 }
}

impl TermionScreenPainter for TermionScreenViewPage {
 fn paint(&mut self, _assd: &mut AppStateReceiverData) {
  let scroller = &mut self.scroller;
  let layout = &mut self.layout;

  let string_lines = self.text.split("\n").collect::<Vec<_>>();

  let mut rv = RatatuiVariables::<PagerLayout>::new();

  {
   let (width, height) = termion::terminal_size().unwrap();
   layout.set_width_height(width, height);

   let mut lines = vec![];

   {
    scroller.set_content_length(Some(string_lines.len()));
    // scroller.set_windowlength(height + 1 - layout.get_current_line());
    scroller.set_windowlength(rv.pl.main_area.inner(Margin::new(0, 1)).height);

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

   rv
    .terminal
    .draw(|frame| {
     frame.render_widget(Text::raw(HELP_QX), rv.pl.title_area);
     frame.render_widget(Text::raw(all_lines), rv.pl.main_area.inner(Margin::new(0, 1)));
    })
    .unwrap();
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
