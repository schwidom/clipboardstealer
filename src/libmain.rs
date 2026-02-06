#![allow(dead_code)]
#![allow(unused)]

// extern crate clipboard;
extern crate clap;
extern crate termion;
extern crate x11_clipboard;

use ratatui;

use termion::{
 cursor::{Hide, Show},
 event::{Event, Key},
 input::TermRead,
 is_tty,
 raw::{IntoRawMode, RawTerminal},
};

use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;

// use x11_clipboard::error::Error as X11Error;
use xcb_1::{
 x::{KeyButMask, QueryPointer},
 Connection,
};

use std::{
 cell::RefCell,
 io::Write,
 os::fd::AsFd,
 rc::Rc,
 sync::mpsc::{self, Receiver, Sender},
 thread::JoinHandle,
};

use std::io::Stdout;
use std::{
 io::{stdin, stdout},
 thread,
};

use crate::{
 clipboards::*,
 config::{sleep_default, Config},
 debug::*,
 event::MyEvent,
 termionscreen::{TermionScreenFirstPage, TermionScreenPainter},
 tools::cb_get_atoms,
};

use nu_ansi_term::AnsiGenericString;

use tracing::{event, info, span, trace, Level};

use clap::Parser;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
 #[arg(long, default_value_t = false, help = "provides debug information")]
 pub(crate) debug: bool,
 #[arg(long, help = "writes debug information into file")]
 pub(crate) debugfile: Option<String>,
}

pub enum MyError {
 PoisonError,
 UnitError,
 // X11Clipboard(X11Error),
}

// impl From<()> for MyError {
//  fn from(value: ()) -> Self {
//   MyError::UnitError
//  }
// }

use x11_clipboard as x11;
// use x11::xcb::Atom;
use x11::Atom;

// use crate::event::*;

/** waits for clipboard events and handles them */
pub struct ClipboardThread {}

impl ClipboardThread {
 fn new() -> Self {
  Self {}
 }

 fn run(&mut self, ass: &'static AppStateSender) -> JoinHandle<Result<(), MyError>> {
  let thread: JoinHandle<_> = thread::spawn(move || -> Result<(), MyError> {
   let crws: Vec<_> = [cb_get_atoms().primary, 2, cb_get_atoms().clipboard]
    .iter()
    .map(|x| ClipboardReaderWriter::new(*x))
    .collect();

   let mut cb_strings: Vec<_> = crws.iter().map(|x| x.read()).collect();

   loop {
    if !ass.is_running() {
     break Ok(());
    }

    let cb_strings2: Vec<_> = crws.iter().map(|x| x.read()).collect();

    for i in 0..cb_strings.len() {
     if cb_strings2[i] != cb_strings[i] {
      ass
       .sender
       .send(MyEvent::CbChanged(crws[i].atom(), cb_strings2[i].clone()))
       .unwrap();
     }
    }

    cb_strings = cb_strings2;

    sleep_default(); // cgyeofnrzk
   }
  });
  thread
 }
}

/// blocking
/** sends termion events to MyEventHandler */
struct TermionLoop {
 stdout_raw: RawTerminal<Stdout>,
}

impl TermionLoop {
 fn new() -> Self {
  let stdout_raw = stdout().into_raw_mode();

  match stdout_raw {
   Ok(stdout_raw) => return Self { stdout_raw },
   Err(err) => panic!("you are not on a terminal : {:?}", err), // TODO : linux tests
  }
 }

 fn run_loop(&mut self, ass: &'static AppStateSender) -> JoinHandle<Result<(), MyError>> {
  let thread = thread::spawn(move || -> Result<(), MyError> {
   let stdin = stdin();
   for e in stdin.events() {
    let u = e.unwrap();

    {
     if !ass.is_running() {
      break;
     }
     ass.sender.send(MyEvent::Termion(u.clone())).unwrap();
    }
    sleep_default(); // cgyeofnrzk
   }
   Ok(())
  });
  thread
 }

 fn suspend_raw_mode(&mut self) {
  self.stdout_raw.suspend_raw_mode().unwrap();
 }
}

impl Drop for TermionLoop {
 fn drop(&mut self) {
  self.suspend_raw_mode();
 }
}

/** sends SIGWINCH, SIGINT events to MyEventHandler*/
struct MySignalsLoop {}

impl MySignalsLoop {
 pub fn new() -> Self {
  Self {}
 }

 fn run_thread(&mut self, ass: &'static AppStateSender) -> JoinHandle<Result<(), MyError>> {
  let mut signals = Signals::new(&[SIGWINCH, SIGINT]).unwrap();
  let thread = thread::spawn(move || -> Result<(), MyError> {
   for signal in &mut signals {
    {
     if !ass.is_running() {
      break;
     }
     ass.sender.send(MyEvent::SignalHook(signal)).unwrap();
    }
    sleep_default(); // cgyeofnrzk
   }
   Ok(())
  });
  thread
 }
}

struct MouseThread<'a> {
 config: &'a Config,
}

impl<'a> MouseThread<'a> {
 fn new(config: &'a Config) -> Self {
  Self { config }
 }
 fn run(&self, ass: &'static AppStateSender) -> JoinHandle<Result<(), MyError>> {
  let debug = self.config.debug;
  let thread = thread::spawn(move || -> Result<(), MyError> {
   // TODO : clean up the unwrap
   let displayname: String = std::env::var_os("DISPLAY")
    .unwrap()
    .to_string_lossy()
    .into();
   let (connection, preferred_screen) = Connection::connect(Some(&displayname)).unwrap();
   if debug {
    trace!("MouseThread goes into loop state");
   }

   let setup = connection.get_setup();
   let screen = setup.roots().nth(preferred_screen as usize).unwrap();
   let rootwindow = screen.root();

   let mut mousebutton1pressed = false;
   let mut shift_pressed = false;

   // ss.loop_start.read();
   loop {
    // if ss.meh.lock()?.get_stop_threads() {
    //  break;
    // }
    if !ass.is_running() {
     break;
    }
    let cookie = connection.send_request(&QueryPointer { window: rootwindow });
    let event = connection.wait_for_reply(cookie);

    let event_mask = event.unwrap().mask();

    let x = event_mask.contains(KeyButMask::BUTTON1);
    if x && !mousebutton1pressed {
     // ss.meh.lock()?.push_event(&MyEvent::MouseButton1(true))?;
     ass.sender.send(MyEvent::MouseButton1(true)).unwrap();
     sleep_default(); // cgyeofnrzk
     mousebutton1pressed = x
    }
    if !x && mousebutton1pressed {
     // ss.meh.lock()?.push_event(&MyEvent::MouseButton1(false))?;
     ass.sender.send(MyEvent::MouseButton1(false)).unwrap();
     sleep_default(); // cgyeofnrzk
     mousebutton1pressed = x
    }

    let y = event_mask.contains(KeyButMask::SHIFT);
    if y && !shift_pressed {
     // ss.meh.lock()?.push_event(&MyEvent::Shift(true))?;
     ass.sender.send(MyEvent::Shift(true)).unwrap();
     sleep_default(); // cgyeofnrzk
     shift_pressed = y
    }

    if !y && shift_pressed {
     // ss.meh.lock()?.push_event(&MyEvent::Shift(false))?;
     ass.sender.send(MyEvent::Shift(false)).unwrap();
     sleep_default(); // cgyeofnrzk
     shift_pressed = y
    }
    sleep_default();
   }
   Ok(())
  });
  thread
 }
}

#[derive(Debug)]
struct AppStateSender<'a> {
 running: &'a AtomicBool,
 sender: Sender<MyEvent>,
}

impl<'a> AppStateSender<'a> {
 fn new(running: &'a AtomicBool, sender: Sender<MyEvent>) -> Self {
  Self { running, sender }
 }

 fn is_running(&self) -> bool {
  self.running.load(Ordering::Relaxed)
 }
}

// WEITERBEI, TODO : define the data, see g6lyj3epcb
pub struct AppStateReceiverData {
 pub cbs: Clipboards,
}

impl AppStateReceiverData {
 pub fn new() -> Self {
  Self {
   cbs: Clipboards::new(),
  }
 }
}

pub struct AppStateReceiver<'a> {
 running: &'a AtomicBool,
 receiver: Receiver<MyEvent>,
 config: &'static Config,
 data: AppStateReceiverData,
}

impl<'a> AppStateReceiver<'a> {
 fn new(config: &'static Config, running: &'a AtomicBool, receiver: Receiver<MyEvent>) -> Self {
  Self {
   running,
   receiver,
   config,
   data: AppStateReceiverData::new(),
  }
 }

 fn is_running(&self) -> bool {
  self.running.load(Ordering::Relaxed)
 }

 fn run_loop(&mut self) {
  // self.running.store(true, Ordering::Relaxed);
  assert!(self.is_running());
  let mut tsp_default: Rc<RefCell<dyn TermionScreenPainter>> =
   Rc::new(RefCell::new(TermionScreenFirstPage::new(self.config)));
  let mut tsp_stack: Vec<Rc<RefCell<dyn TermionScreenPainter>>> = vec![];

  #[derive(Default)]
  struct EventState {
   mouse_button_1_is_pressed: bool,
   shift_is_pressed: bool,
   cb_changed: Option<(Atom, Option<String>)>,
  }

  let mut event_state = EventState::default();

  impl EventState {
   fn update_clipboard(&mut self, cbs: &mut Clipboards) {
    if !self.mouse_button_1_is_pressed && !self.shift_is_pressed {
     if let Some((atom, string)) = &self.cb_changed {
      cbs.insert(*atom, string.clone());
      self.cb_changed = None;
     }
    }
   }
  }

  let mut so = stdout();

  loop {
   let mut current_painter = tsp_stack.last().unwrap_or(&tsp_default).borrow_mut();
   current_painter.paint(&mut self.data);
   print!("{}", Hide);
   so.flush().unwrap();
   let ev = self.receiver.recv().unwrap(); // TODO : match
   if self.config.debug {
    trace!("ev: {:?}", ev);
   }
   let next_tsp = current_painter.handle_event(&ev, &mut self.data);
   drop(current_painter);
   match next_tsp {
    crate::termionscreen::NextTsp::NoNextTsp => {}
    crate::termionscreen::NextTsp::Replace(rc) => {
     if tsp_stack.is_empty() {
      tsp_default = rc;
     } else {
      *(tsp_stack.last_mut().unwrap()) = rc;
     }
    }
    crate::termionscreen::NextTsp::Stack(rc) => tsp_stack.push(rc),
   }
   if ev.is_stop_event() {
    self.running.store(false, Ordering::Relaxed);
    break;
   } else {
    match ev {
     MyEvent::Termion(Event::Key(Key::Char('q'))) => {
      if tsp_stack.is_empty() {
       self.running.store(false, Ordering::Relaxed);
       break;
      } else {
       tsp_stack.pop();
      }
     }
     // MyEvent::Termion(event) => todo!(),
     // MyEvent::SignalHook(_) => todo!(),
     MyEvent::MouseButton1(pressed) => {
      event_state.mouse_button_1_is_pressed = pressed;
      event_state.update_clipboard(&mut self.data.cbs);
     }

     MyEvent::Shift(pressed) => {
      event_state.shift_is_pressed = pressed;
      event_state.update_clipboard(&mut self.data.cbs);
     }

     // MyEvent::CbInserted => todo!(),
     // MyEvent::Unused => todo!(),
     MyEvent::CbChanged(atom, string) => {
      event_state.cb_changed = Some((atom, string));
      event_state.update_clipboard(&mut self.data.cbs);
     }
     _ => {}
    }
   }
  }
  print!("{}", Show);
  so.flush().unwrap();
 }
}

struct AppState<'a> {
 ass: AppStateSender<'a>,
 asr: AppStateReceiver<'a>,
 running: &'a AtomicBool,
 config: &'static Config,
}

impl<'a> AppState<'a> {
 fn new(config: &'static Config) -> Self {
  let (sender, receiver) = mpsc::channel();
  let running: &mut AtomicBool = Box::leak(Box::new(AtomicBool::new(true)));
  Self {
   ass: AppStateSender::new(running, sender),
   asr: AppStateReceiver::new(config, running, receiver),
   running,
   config,
  }
 }
}

pub fn main() {
 let args = Args::parse();

 let config = Box::leak(Box::new(Config::from_args(&args)));

 // let appstate = AppState::new();
 let appstate = Box::leak(Box::new(AppState::new(config)));

 match (is_tty(&stdin().as_fd()), is_tty(&stdout().as_fd())) {
  (true, true) => {}
  (true, false) => {
   println!("stdin is not a tty");
   return;
  }
  (false, true) => {
   println!("stdout is not a tty");
   return;
  }
  (false, false) => {
   println!("stdin and stdout are not ttys");
   return;
  }
 }

 println!("{}", AnsiGenericString::title("Clipboardstealer"));

 if config.debug {
  monitor();
 }

 // let mut _stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap()); // creates mouse events
 // let mut _stdout2 = RawTerminal::from(stdout().into_raw_mode().unwrap()); // creates  ???

 if config.debug {
  monitor();
 }

 let mt = MouseThread::new(&config);
 let mtjh = mt.run(&appstate.ass);

 if config.debug {
  monitor2("ct");
 }

 let mut ct = ClipboardThread::new();
 let ctjh = ct.run(&appstate.ass);

 if config.debug {
  monitor2("ms");
 }

 let mut ms = MySignalsLoop::new();
 let _msjh = ms.run_thread(&appstate.ass);

 if config.debug {
  monitor2("tl");
 }

 let mut tl = TermionLoop::new();
 let _tljh = tl.run_loop(&appstate.ass);

 if config.debug {
  monitor2("ts");
 }

 if config.debug {
  monitor2("start appstate.asr.run_loop");
 }

 appstate.asr.run_loop();

 if config.debug {
  monitor2("ctjh");
 }

 let _ = ctjh.join(); // needed

 if config.debug {
  monitor2("mtjh");
 }

 let _ = mtjh.join(); // currently not needed, but possible, for the sake of tidyness

 if config.debug {
  monitor2("end");
 }

 ratatui::restore();
 println!("{}", AnsiGenericString::title("Clipboardstealer ended"));

 tl.suspend_raw_mode();
 // tljh.join(); // never!, that would block here, we don't want that
 // msjh.join(); // never!, that would block here, we don't want that
}
