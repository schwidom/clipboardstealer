#![allow(dead_code)]
#![allow(unused)]

use std::cmp::min;
use std::io::{stdout, Stdout, Write};
use std::ops::Deref;
use std::sync::mpsc::Receiver;
use std::sync::TryLockError::{Poisoned, WouldBlock};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::clipboards::Clipboards;

use crate::config::{self, Config};
use crate::constants::{self, HELP_FIRST_PAGE, HELP_QX};
use crate::entries::Entries;
use crate::event::{MyEvent, MyEventHandler};
use crate::layout::Layout;
use crate::layout_ratatui::{FrameNew, PagerLayout};
use crate::libmain::SyncStuff;
use crate::pager::Pager;
use crate::scroller::{CursorRepetitions, Scroller};
use crate::tools::MyTime; // TODO

use ratatui::layout::Margin;
use ratatui::prelude::CrosstermBackend;
use ratatui::text::Text;
use ratatui::Terminal;
use signal_hook::consts::SIGWINCH;
use termion::event::{Event, Key};
use termion::screen::IntoAlternateScreen;
use termion::{self, scroll};

// use num::Integer;

struct TermionScreens<'a> {
 config: &'a Config,
 cbs: Arc<Mutex<Clipboards>>,
 ss: SyncStuff,
 stdout: Stdout,
 receiver: Arc<Mutex<Receiver<MyEvent>>>,
}

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

impl<'a> TermionScreens<'a> {
 fn new(config: &'a Config, cbs: Arc<Mutex<Clipboards>>, ss: SyncStuff) -> Self {
  let receiver = ss.meh.clone().lock().unwrap().get_receiver().clone();
  let stdout = stdout();
  Self {
   config,
   cbs,
   ss,
   stdout,
   receiver,
  }
 }

 fn cls(&mut self) {
  write!(self.stdout, "{}", termion::cursor::Goto(1, 1));
  write!(self.stdout, "{}", termion::clear::All).unwrap();
 }

 // NOTE : übernimmt Layout, kann ggf. raus
 fn gotoline(&mut self, line1: u16) {
  write!(self.stdout, "{}", termion::cursor::Goto(1, line1));
 }

 fn flush(&mut self) {
  self.stdout.flush().unwrap();
 }

 fn event_page(&mut self) {
  self.cls();

  print!("Event Page, q ends the screen");

  loop {
   let (_width, _height) = termion::terminal_size().unwrap();

   self.flush();
   // println!("flushed");

   let evt = match self.receiver.lock() {
    // blocks
    Ok(rcv) => match rcv.recv() {
     Ok(value) => value,
     Err(_) => break,
    },
    Err(_) => break,
   };

   // // if meh is nedded longer after recv, use this one
   // if self.ss.meh.lock().unwrap().get_stop_threads() {
   //  break;
   // }

   // if meh is nedded longer after recv, create a new thread
   if self.get_stop_threads() {
    break;
   }

   // println!(" got : {:?}", evt);

   match evt {
    MyEvent::Termion(Event::Key(Key::Char('q'))) => break, // gbrxzcymlj
    _ => {
     print!("{:?}", evt);
    }
   }
  }
  self.flush();
 }

 fn first_page(&mut self) {
  let mut scroller = Scroller::new();
  let mut layout = Layout::new();

  // self.cls(); // in loop possible but flickers

  let cbsclone = self.cbs.clone();

  let mut terminal_reinitialize = false;
  let mut rv = RatatuiVariables::<PagerLayout>::new();

  let mut is_folded = false;

  loop {
   if terminal_reinitialize {
    rv = RatatuiVariables::new();
    terminal_reinitialize = false;
   }

   // x9kwvw3yj0, ic4q5snjyp t 9
   // if self.meh.lock().unwrap().get_stop_threads() {
   //  break;
   // }

   // frame.render_widget(Text::raw("123"), pl.title_area); // don't work

   if false {
    // works
    rv.terminal.draw(|mut frame| {
     frame.render_widget(Text::raw("123"), rv.pl.status_area);
    });
    thread::sleep(Duration::from_secs(1));
   }

   let (width, height) = termion::terminal_size().unwrap();
   layout.set_width_height(width, height);
   // layout.reset_current_line();

   // layout.print_line_cut(HELP_FIRST_PAGE);

   // self.flush();
   // println!("flushed");

   let mut lines = vec![];

   let selected_entry = {
    // screen listing
    let cbs = cbsclone.lock().unwrap();

    let mut entries = vec![];
    for (name, cb) in &cbs.hm {
     entries.append(&mut Entries::from_csl(&name, cb));
    }

    entries.sort_by(|x, y| y.timestamp.cmp(&x.timestamp));

    scroller.set_content_length(Some(entries.len()));
    // scroller.set_windowlength(height + 1 - layout.get_current_line());
    scroller.set_windowlength(rv.pl.main_area.inner(Margin::new(0, 1)).height);

    // TODO : restore
    // if self.config.debug {
    //  let s001 =
    //   format!("w {}, h {}, cl {}, {:?}", width, height, layout.get_current_line(), scroller);
    //  layout.print_line_cut(&s001);
    // } else {
    //  layout.print_line_cut("");
    // }

    // let leading_zeroes = log( 10); // TODO : later

    for (idx, entry) in entries[scroller.get_windowrange()].iter().enumerate() {
     let is_cursor = match scroller.get_cursor() {
      None => false,
      Some(value) => {
       if idx as u16 == value {
        true
       } else {
        false
       }
      }
     };

     let cursor_star = if is_cursor { ">" } else { " " };

     let is_selected = {
      let csl = entry.csl.lock().unwrap();
      if Some(entry.csl_idx) == csl.current_selection {
       true
      } else {
       false
      }
     };

     let selection_star = if is_selected { "*" } else { " " };

     // layout.print_line_cut(&s002);
     if is_cursor && !is_folded {
      let s002 = format!(
       "{} {} {} {} {} : \n {}",
       cursor_star,
       selection_star,
       idx + scroller.get_windowposition(), // mqbojcmkot
       entry.info,
       entry.get_date_time(),
       entry.string
      );
      lines.push(s002);
     } else {
      let s002 = format!(
       "{} {} {} {} {} : {}",
       cursor_star,
       selection_star,
       idx + scroller.get_windowposition(), // mqbojcmkot
       entry.info,
       entry.get_date_time(),
       entry.string
      );
      lines.push(layout.fixline(&s002));
     }
    }

    match scroller.get_cursor_in_array() {
     None => None,
     // Some(value) => entries.get(value).clone(),
     Some(value) => Some(entries[value].clone()),
    }
   };

   let all_lines = lines.join("\n");

   rv.terminal.draw(|mut frame| {
    frame.render_widget(Text::raw(constants::HELP_FIRST_PAGE), rv.pl.title_area);
    frame.render_widget(Text::raw(all_lines), rv.pl.main_area.inner(Margin::new(0, 1)));
   });

   // print!("{}", termion::clear::AfterCursor);

   // println gets printed, print or write needs flush
   // self.flush();

   // a0vbfusiba // TermionScreens.first_page
   let evt = match self.receiver.lock() {
    // blocks
    Ok(rcv) => match rcv.recv() {
     Ok(value) => value,
     Err(_) => break,
    },
    Err(_) => break,
   };

   // // fddt4zu0y5 t 9
   // // br83mnnp4d t 10
   // // if meh is nedded longer after recv, use this one
   // if self.ss.meh.lock().unwrap().get_stop_threads() {
   //  break;
   // }

   // if meh is nedded longer after recv, create a new thread
   if self.get_stop_threads() {
    break;
   }

   // println!(" got : {:?}", evt);

   match evt {
    MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
    MyEvent::Termion(Event::Key(Key::Char('q'))) => break, // gbrxzcymlj
    MyEvent::Termion(Event::Key(Key::Char('t'))) => {
     if self.config.debug {
      // fill with testdata
      let mut cbs = self.cbs.lock().unwrap();
      for i in 10..20 {
       for (name, cb) in &cbs.hm {
        cb
         .lock()
         .unwrap()
         .captured_from_clipboard
         .push((MyTime::now(), i.to_string()));
       }
      }
      println!("done f");
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('h'))) => {
     // println!("got a");
     self.help_page();
     terminal_reinitialize = true;
    }
    MyEvent::Termion(Event::Key(Key::Char('e'))) => {
     if self.config.debug {
      // println!("got a");
      self.event_page();
      terminal_reinitialize = true;
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('f'))) => {
     is_folded = !is_folded;
    }
    MyEvent::Termion(Event::Key(Key::Char('s'))) => match selected_entry {
     None => {}
     Some(se) => se.select(),
    },
    MyEvent::Termion(Event::Key(Key::Char('v'))) => match selected_entry {
     None => {}
     Some(se) => {
      self.view_page(&se.string);
      terminal_reinitialize = true;
     }
    },
    _ => {
     Pager::handle_event(&mut scroller, &evt);
    }
   }
  }
  // self.flush();
 }

 // kgi2gntsqo
 fn view_page(&mut self, string_to_view: &str) {
  let mut scroller = Scroller::new();
  let mut layout = Layout::new();

  let string_lines = string_to_view.split("\n").collect::<Vec<_>>();

  // self.cls(); // in loop possible but flickers

  let mut terminal_reinitialize = false;
  let mut rv = RatatuiVariables::<PagerLayout>::new();

  loop {
   if terminal_reinitialize {
    rv = RatatuiVariables::new();
    terminal_reinitialize = false;
   }

   let (width, height) = termion::terminal_size().unwrap();
   layout.set_width_height(width, height);
   // layout.reset_current_line();

   // layout.print_line_cut(HELP_QX);

   // self.flush();
   // println!("flushed");

   let mut lines = vec![];

   let selected_entry = {
    // screen listing
    scroller.set_content_length(Some(string_lines.len()));
    // scroller.set_windowlength(height + 1 - layout.get_current_line());
    scroller.set_windowlength(rv.pl.main_area.inner(Margin::new(0, 1)).height);

    // if self.config.debug {
    //  let s001 =
    //   format!("w {}, h {}, cl {}, {:?}", width, height, layout.get_current_line(), scroller);
    //  layout.print_line_cut(&s001);
    // } else {
    //  layout.print_line_cut("");
    // }

    // let leading_zeroes = log( 10); // TODO : later

    for (idx, entry) in string_lines[scroller.get_windowrange()].iter().enumerate() {
     let cursor_star = match scroller.get_cursor() {
      None => " ",
      Some(value) => {
       if idx as u16 == value {
        ">"
       } else {
        " "
       }
      }
     };

     let s002 = format!(
      "{} {} : {}",
      cursor_star,
      idx + scroller.get_windowposition(), // mqbojcmkot
      entry,
     );
     // layout.print_line_cut(&s002);
     lines.push(layout.fixline(&s002));
    }

    match scroller.get_cursor_in_array() {
     None => None,
     // Some(value) => entries.get(value).clone(),
     Some(value) => Some(string_lines[value]),
    }
   };

   let all_lines = lines.join("\n");

   rv.terminal.draw(|mut frame| {
    frame.render_widget(Text::raw(constants::HELP_QX), rv.pl.title_area);
    // wdlxnboitz
    frame.render_widget(Text::raw(all_lines), rv.pl.main_area.inner(Margin::new(0, 1)));
   });

   // print!("{}", termion::clear::AfterCursor);

   // println gets printed, print or write needs flush
   // self.flush();

   let evt = match self.receiver.lock() {
    // blocks
    Ok(rcv) => match rcv.recv() {
     Ok(value) => value,
     Err(_) => break,
    },
    Err(_) => break,
   };

   // // if meh is nedded longer after recv, use this one
   // if self.ss.meh.lock().unwrap().get_stop_threads() {
   //  break;
   // }

   // if meh is nedded longer after recv, create a new thread
   if self.get_stop_threads() {
    break;
   }

   // println!(" got : {:?}", evt);

   match evt {
    MyEvent::Termion(Event::Key(Key::Char('q'))) => break, // gbrxzcymlj
    MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
    _ => {
     Pager::handle_event(&mut scroller, &evt);
    }
   }
  }
  // self.flush();
 }

 fn help_page(&mut self) {
  let text = config::USAGE;

  self.view_page(&text);
 }

 // TODO : error handling
 fn get_stop_threads(&self) -> bool {
  match self.ss.meh.try_lock() {
   Err(err) => match err {
    Poisoned(poison_error) => {
     // TODO
     eprintln!("poison_error : {:?}", poison_error);
     true
    }
    WouldBlock => false,
   },
   Ok(meh) => meh.get_stop_threads(),
  }
 }
}

pub struct TermionScreen<'a> {
 config: &'a Config,
 cbs: Arc<Mutex<Clipboards>>,
}

impl<'a> TermionScreen<'a> {
 pub fn new(config: &'a Config, cbs: Arc<Mutex<Clipboards>>) -> Self {
  Self { config, cbs }
 }

 pub fn run_loop(&mut self, ss: SyncStuff) -> JoinHandle<()> {
  let cbs = self.cbs.clone();
  let config = (*self.config).clone();
  let thread = thread::spawn(move || {
   // saves the previous console picture
   let mut stdout = stdout().into_alternate_screen().unwrap();
   write!(stdout, "{}", termion::cursor::Goto(1, 1));
   write!(stdout, "{}", termion::clear::All).unwrap();
   stdout.flush().unwrap();

   if true {
    let mut tss = TermionScreens::new(&config, cbs.clone(), ss.clone());
    ss.loop_start.read();
    // a0vbfusiba, x9kwvw3yj0, ic4q5snjyp t 9, fddt4zu0y5 t 9  // TermionScreen.run_loop
    // br83mnnp4d t 10
    tss.first_page();
    ss.meh.lock().unwrap().set_stop_threads();
   }
   stdout.flush().unwrap();
  });
  thread
 }
}
