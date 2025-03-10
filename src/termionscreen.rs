#![allow(dead_code)]
#![allow(unused)]

use std::cmp::min;
use std::io::{stdout, Stdout, Write};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::clipboards::Clipboards;

use crate::config::Config;
use crate::constants::{HELP_FIRST_PAGE, HELP_QX};
use crate::entries::Entries;
use crate::event::{MyEvent, MyEventHandler};
use crate::layout::Layout;
use crate::pager::Pager;
use crate::scroller::{CursorRepetitions, Scroller};
use crate::tools::MyTime; // TODO

use termion::event::{Event, Key};
use termion::screen::IntoAlternateScreen;
use termion::{self, scroll};

// use num::Integer;

struct TermionScreens<'a> {
 config: &'a Config,
 cbs: Arc<Mutex<Clipboards>>,
 meh: Arc<Mutex<MyEventHandler>>,
 stdout: Stdout,
}

impl<'a> TermionScreens<'a> {
 fn new(config: &'a Config, cbs: Arc<Mutex<Clipboards>>, meh: Arc<Mutex<MyEventHandler>>) -> Self {
  let stdout = stdout();
  Self {
   config,
   cbs,
   meh,
   stdout,
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

  let receiver = self.meh.lock().unwrap().get_receiver();

  loop {
   let (_width, _height) = termion::terminal_size().unwrap();

   self.flush();
   // println!("flushed");

   let evt = match receiver.lock() {
    // blocks
    Ok(rcv) => match rcv.recv() {
     Ok(value) => value,
     Err(_) => break,
    },
    Err(_) => break,
   };

   // if meh is nedded longer after recv, use this one
   if self.meh.lock().unwrap().get_stop_threads() {
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

  self.cls(); // in loop possible but flickers

  let receiver = self.meh.lock().unwrap().get_receiver();

  loop {
   // x9kwvw3yj0, ic4q5snjyp t 9
   // if self.meh.lock().unwrap().get_stop_threads() {
   //  break;
   // }

   let (width, height) = termion::terminal_size().unwrap();
   layout.set_width_height(width, height);
   layout.reset_current_line();

   layout.print_line_cut(HELP_FIRST_PAGE);

   self.flush();
   // println!("flushed");

   let selected_entry = {
    // screen listing
    let cbsclone = self.cbs.clone();
    let cbs = cbsclone.lock().unwrap();

    let mut entries = Entries::from_csl("p", &cbs.primary);
    entries.append(&mut Entries::from_csl("s", &cbs.clipboard));

    entries.sort_by(|x, y| y.timestamp.cmp(&x.timestamp));

    scroller.set_content_length(Some(entries.len()));
    scroller.set_windowlength(height + 1 - layout.get_current_line());

    if self.config.debug {
     let s001 =
      format!("w {}, h {}, cl {}, {:?}", width, height, layout.get_current_line(), scroller);
     layout.print_line_cut(&s001);
    } else {
     layout.print_line_cut("");
    }

    // let leading_zeroes = log( 10); // TODO : later

    for (idx, entry) in entries[scroller.get_windowrange()].iter().enumerate() {
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

     let selection_star = {
      let csl = entry.csl.lock().unwrap();
      if Some(entry.csl_idx) == csl.current_selection {
       "*"
      } else {
       " "
      }
     };

     let s002 = format!(
      "{} {} {} {} {} : {}",
      cursor_star,
      selection_star,
      idx + scroller.get_windowposition(), // mqbojcmkot
      entry.info,
      entry.get_date_time(),
      entry.string
     );
     layout.print_line_cut(&s002);
    }

    match scroller.get_cursor_in_array() {
     None => None,
     // Some(value) => entries.get(value).clone(),
     Some(value) => Some(entries[value].clone()),
    }
   };

   print!("{}", termion::clear::AfterCursor);

   // println gets printed, print or write needs flush
   self.flush();

   // a0vbfusiba // TermionScreens.first_page
   let evt = match receiver.lock() {
    // blocks
    Ok(rcv) => match rcv.recv() {
     Ok(value) => value,
     Err(_) => break,
    },
    Err(_) => break,
   };

   // if meh is nedded longer after recv, use this one
   if self.meh.lock().unwrap().get_stop_threads() {
    break;
   }

   // println!(" got : {:?}", evt);

   match evt {
    MyEvent::Termion(Event::Key(Key::Char('q'))) => break, // gbrxzcymlj
    MyEvent::Termion(Event::Key(Key::Char('f'))) => {
     if self.config.debug {
      // fill with testdata
      let mut cbs = self.cbs.lock().unwrap();
      for i in 10..20 {
       cbs
        .primary
        .lock()
        .unwrap()
        .captured_from_clipboard
        .push((MyTime::now(), i.to_string()));
       cbs
        .clipboard
        .lock()
        .unwrap()
        .captured_from_clipboard
        .push((MyTime::now(), i.to_string()));
      }
      println!("done f");
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('h'))) => {
     // println!("got a");
     self.help_page();
    }
    MyEvent::Termion(Event::Key(Key::Char('e'))) => {
     if self.config.debug {
      // println!("got a");
      self.event_page();
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('s'))) => match selected_entry {
     None => {}
     Some(se) => se.select(),
    },
    MyEvent::Termion(Event::Key(Key::Char('v'))) => match selected_entry {
     None => {}
     Some(se) => self.view_page(&se.string),
    },
    _ => {
     Pager::handle_event(&mut scroller, &evt);
    }
   }
  }
  self.flush();
 }

 // kgi2gntsqo
 fn view_page(&mut self, string_to_view: &str) {
  let mut scroller = Scroller::new();
  let mut layout = Layout::new();

  let string_lines = string_to_view.split("\n").collect::<Vec<_>>();

  self.cls(); // in loop possible but flickers

  let receiver = self.meh.lock().unwrap().get_receiver();

  loop {
   let (width, height) = termion::terminal_size().unwrap();
   layout.set_width_height(width, height);
   layout.reset_current_line();

   layout.print_line_cut(HELP_QX);

   self.flush();
   // println!("flushed");

   let selected_entry = {
    // screen listing
    scroller.set_content_length(Some(string_lines.len()));
    scroller.set_windowlength(height + 1 - layout.get_current_line());

    if self.config.debug {
     let s001 =
      format!("w {}, h {}, cl {}, {:?}", width, height, layout.get_current_line(), scroller);
     layout.print_line_cut(&s001);
    } else {
     layout.print_line_cut("");
    }

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
     layout.print_line_cut(&s002);
    }

    match scroller.get_cursor_in_array() {
     None => None,
     // Some(value) => entries.get(value).clone(),
     Some(value) => Some(string_lines[value].clone()),
    }
   };

   print!("{}", termion::clear::AfterCursor);

   // println gets printed, print or write needs flush
   self.flush();

   let evt = match receiver.lock() {
    // blocks
    Ok(rcv) => match rcv.recv() {
     Ok(value) => value,
     Err(_) => break,
    },
    Err(_) => break,
   };

   // if meh is nedded longer after recv, use this one
   if self.meh.lock().unwrap().get_stop_threads() {
    break;
   }

   // println!(" got : {:?}", evt);

   match evt {
    MyEvent::Termion(Event::Key(Key::Char('q'))) => break, // gbrxzcymlj
    _ => {
     Pager::handle_event(&mut scroller, &evt);
    }
   }
  }
  self.flush();
 }

 fn help_page(&mut self) {
  let text = r"
  
  clipboardstealer [--debug]

  - runs in a terminal window, 
  - captures the primary and secondary X11 clipboard
  - allows selection of primary and secondary X11 clipboard
  - enforces the user choice

  - Keys: 

   orientation: Cursor Up, Cursor Down, PgUp, PgDown, Home, End
   orientation: Cursor Left, Cursor Right (not implemented yet)

   regex search ... / (not implemented yet)

   (h)elp ... this screen 
   (v)iew ... shows the selected entry
   (s)elect ... selects the chosen entry and 
                enforces it for the specific 
                (p)rimary or (s)econdary clipboard

   (q)uit ... exits a screen
   e(x)it ... exits the program
   Ctrl-C ... exits the program
  
  Copyright : Frank Schwidom 2025
  This software is licensed under the terms of the Apache-2.0 license. ";

  self.view_page(&text);
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

 pub fn run_loop(&mut self, meh: Arc<Mutex<MyEventHandler>>) -> JoinHandle<()> {
  let cbs = self.cbs.clone();
  let config = (*self.config).clone();
  let thread = thread::spawn(move || {
   // saves the previous console picture
   let mut stdout = stdout().into_alternate_screen().unwrap();
   write!(stdout, "{}", termion::cursor::Goto(1, 1));
   write!(stdout, "{}", termion::clear::All).unwrap();
   stdout.flush().unwrap();

   if true {
    let mut tss = TermionScreens::new(&config, cbs.clone(), meh.clone());
    // a0vbfusiba, x9kwvw3yj0, ic4q5snjyp t 9 // TermionScreen.run_loop
    tss.first_page();
    meh.lock().unwrap().set_stop_threads();
   }
   stdout.flush().unwrap();
  });
  thread
 }
}
