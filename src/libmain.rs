#![allow(dead_code)]
#![allow(unused)]

// extern crate clipboard;
extern crate clap;
extern crate termion;
extern crate x11_clipboard;

use termion::{
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
 borrow::Borrow,
 error::Error,
 fs::{File, OpenOptions},
 io::Write,
 os::fd::AsFd,
 path::PathBuf,
 str::FromStr,
 sync::{LazyLock, MutexGuard, PoisonError, RwLock, TryLockError},
 thread::JoinHandle,
};

use std::{
 io::Stdout,
 sync::{Arc, Mutex},
};
use std::{
 io::{stdin, stdout},
 thread,
};

use crate::{
 config::{sleep_default, Config},
 debug::*,
 event::{MyEvent, MyEventHandler},
};

use crate::clipboards::*;

use crate::termionscreen::TermionScreen;

use nu_ansi_term::AnsiGenericString;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
 #[arg(long, default_value_t = false)]
 pub(crate) debug: bool,
}

pub enum MyError {
 PoisonError,
 UnitError,
 // X11Clipboard(X11Error),
}

// From<PoisonError<MutexGuard<'_, MyEventHandler>>>`
// impl From<PoisonError<_>> for MyError {
//  fn from(value: PoisonError<_>) -> Self {
//   MyError::PoisonError
//  }
// }

impl From<PoisonError<MutexGuard<'_, MyEventHandler>>> for MyError {
 fn from(value: PoisonError<MutexGuard<'_, MyEventHandler>>) -> Self {
  MyError::PoisonError
 }
}

impl From<()> for MyError {
 fn from(value: ()) -> Self {
  MyError::UnitError
 }
}

// use crate::event::*;

/** waits for clipboard events and handles them */
pub struct ClipboardThread {
 cbs: Arc<Mutex<Clipboards>>,
}

impl ClipboardThread {
 fn new() -> Self {
  Self {
   cbs: Arc::new(Mutex::new(Clipboards::new())),
  }
 }

 fn run(&mut self, ss: SyncStuff) -> JoinHandle<Result<(), MyError>> {
  let cbs = self.cbs.clone();

  let thread: JoinHandle<_> = thread::spawn(move || -> Result<(), MyError> {
   let meh = ss.meh;
   // TODO : ggf. in verschiedene threads zerlegen mit verschiedenen timeouts
   ss.loop_start.read();
   loop {
    // dt0gtu9sxm, ic4q5snjyp t 6 alt, fddt4zu0y5 t 6 // ClipboardThread.run
    match meh.lock() {
     Err(poison_error) => {
      // eprintln!("{:?}", poison_error); // TODO : logfile
      break Err(MyError::PoisonError);
     }
     Ok(meh) => {
      if meh.get_stop_threads() {
       break Err(MyError::PoisonError);
      }

      // dbaphuses4, a0vbfusiba // ClipboardThread.run
      // sleep_default();

      if meh.get_mouse_button_1_is_pressed() {
       continue;
      }

      if meh.get_shift_is_pressed() {
       continue;
      }
     }
    }

    let (inserted_primary, inserted_secondary, inserted_clipboard) = match cbs.lock() {
     Err(poison_error) => {
      break Err(MyError::PoisonError);
     }
     Ok(cbs) => {
      let inserted_primary = match cbs.primary.lock() {
       Err(poison_error) => {
        break Err(MyError::PoisonError);
       }
       Ok(mut cbs) => cbs.process_clipboard_contents(),
      };

      let inserted_secondary = match cbs.secondary.lock() {
       Err(poison_error) => {
        break Err(MyError::PoisonError);
       }
       Ok(mut cbs) => cbs.process_clipboard_contents(),
      };

      let inserted_clipboard = match cbs.clipboard.lock() {
       Err(poison_error) => {
        break Err(MyError::PoisonError);
       }
       Ok(mut cbs) => cbs.process_clipboard_contents(),
      };

      (inserted_primary, inserted_secondary, inserted_clipboard)
     }
    };

    if inserted_primary.0 {
     meh.lock()?.push_event(&MyEvent::CbInsertedPrimary)?;
    }

    sleep_default(); // cgyeofnrzk // avoids deadlock

    // NOTE : maybe it is better, to have a mutex here which is blocked as long as meh is used
    // on the receiver side (or a Barrier)
    // the Barrier had to be after the meh lock on the receiver side
    // and should be here after the push_event and after meh is released here
    // thread::yield_now(); // don't avoid deadlock

    if inserted_secondary.0 {
     meh.lock()?.push_event(&MyEvent::CbInsertedSecondary)?;
    }

    sleep_default(); // cgyeofnrzk // avoids deadlock

    if inserted_clipboard.0 {
     // ic4q5snjyp t 6
     meh.lock()?.push_event(&MyEvent::CbInsertedClipboard)?;
    }

    sleep_default(); // cgyeofnrzk // avoids deadlock
   }
  });
  thread
 }
}

/// blocking
/** sends termion events to MyEventHandler */
struct TermionLoop {
 stdout_raw: Arc<Mutex<RawTerminal<Stdout>>>,
}

impl TermionLoop {
 fn new() -> Self {
  let stdout_raw = stdout().into_raw_mode();

  match stdout_raw {
   Ok(stdout_raw) => {
    return Self {
     stdout_raw: Arc::new(Mutex::new(stdout_raw)),
    }
   }
   Err(err) => panic!("you are not on a terminal : {:?}", err), // TODO : linux tests
  }

  // Self {
  //  stdout_raw: Arc::new(Mutex::new(stdout().into_raw_mode().unwrap())),
  // }
 }

 fn run_loop(&mut self, ss: SyncStuff) -> JoinHandle<Result<(), MyError>> {
  // let mut tla = self.tla.clone();
  // let mut stdout_raw = self.stdout_raw.lock().unwrap();
  // let mut stdout_raw = self.stdout_raw.clone();
  let thread = thread::spawn(move || -> Result<(), MyError> {
   let meh = ss.meh;
   // let mut tla = tla.lock().unwrap();
   let stdin = stdin();
   // ic4q5snjyp t 8
   ss.loop_start.read();
   for e in stdin.events() {
    // let mut stdout = stdout_raw.lock().unwrap();
    let u = e.unwrap();

    // a0vbfusiba, x9kwvw3yj0, dt0gtu9sxm, ic4q5snjyp t 8 alt // TermionLoop.run_loop
    {
     let mut meh = meh.lock()?;
     if meh.get_stop_threads() {
      break;
     }
     // fddt4zu0y5 t 8 // TermionLoop.run_loop
     meh.push_event(&MyEvent::Termion(u.clone()))?;
    }
    sleep_default(); // cgyeofnrzk
   }
   Ok(())
  });
  thread
 }
}

impl Drop for TermionLoop {
 fn drop(&mut self) {
  self.stdout_raw.lock().unwrap().suspend_raw_mode();
 }
}

/** sends SIGWINCH, SIGINT events to MyEventHandler*/
struct MySignalsLoop {}

impl MySignalsLoop {
 pub fn new() -> Self {
  Self {}
 }

 fn run_thread(&mut self, ss: SyncStuff) -> JoinHandle<Result<(), MyError>> {
  let mut signals = Signals::new(&[SIGWINCH, SIGINT]).unwrap();
  // let handle = signals.handle();

  let thread = thread::spawn(move || -> Result<(), MyError> {
   let meh = ss.meh;
   // a0vbfusiba, x9kwvw3yj0, fddt4zu0y5 t 7 // MySignalsLoop.run_thread
   ss.loop_start.read();
   for signal in &mut signals {
    // dbaphuses4, ic4q5snjyp t 7
    {
     let mut meh = meh.lock()?;
     if meh.get_stop_threads() {
      break;
     }
     // println!("signal : {:?}", signal);
     // ic4q5snjyp t 7 alt
     meh.push_event(&MyEvent::SignalHook(signal))?;
    }
    sleep_default(); // cgyeofnrzk
   }
   Ok(())
  });
  thread
 }
}

/** blocks until a quit code is sent and sets the stop threads information in MyEventHandler */
struct WaitForEnd {}

impl WaitForEnd {
 fn new() -> Self {
  Self {}
 }

 fn run_blocking(self, meh: Arc<Mutex<MyEventHandler>>) {
  loop {
   // TODO : semaphore? or mpsc?
   sleep_default();
   // dbaphuses4, a0vbfusiba, x9kwvw3yj0, dt0gtu9sxm // WaitForEnd.run_blocking
   if meh.lock().unwrap().get_stop_threads() {
    break;
   }
  }
 }
}

struct MouseThread<'a> {
 config: &'a Config,
}

impl<'a> MouseThread<'a> {
 fn new(config: &'a Config) -> Self {
  Self { config }
 }
 fn run(&self, ss: SyncStuff) -> JoinHandle<Result<(), MyError>> {
  let debug = self.config.debug;
  let thread = thread::spawn(move || -> Result<(), MyError> {
   // println!("EventMask::all() {:x}", EventMask::all());
   // TODO : clean up the unwrap
   let displayname: String = std::env::var_os("DISPLAY")
    .unwrap()
    .to_string_lossy()
    .into();
   // println!("display : {displayname}");
   let (connection, preferred_screen) = Connection::connect(Some(&displayname)).unwrap();
   // println!("preferred_screen : {preferred_screen}");
   // println!("connection : {:?}", connection);

   if debug {
    println!("MouseThread goes into loop state");
   }

   let setup = connection.get_setup();
   let screen = setup.roots().nth(preferred_screen as usize).unwrap();
   let rootwindow = screen.root();
   // laut "xwininfo -tree -root | less" ist 0x1de das root window, zeigt sich hier an den Ergebnissen
   // println!("rootwindow : {:x}", rootwindow.resource_id());

   let mut mousebutton1pressed = false;
   let mut shift_pressed = false;

   ss.loop_start.read();
   loop {
    // dbaphuses4, x9kwvw3yj0 2x, dt0gtu9sxm, ic4q5snjyp t 2, fddt4zu0y5 t 2
    if ss.meh.lock()?.get_stop_threads() {
     break;
    }
    let cookie = connection.send_request(&QueryPointer { window: rootwindow });
    let event = connection.wait_for_reply(cookie);

    let event_mask = event.unwrap().mask();

    let x = event_mask.contains(KeyButMask::BUTTON1);
    if x && !mousebutton1pressed {
     // println!("press");
     ss.meh.lock()?.push_event(&MyEvent::MouseButton1(true))?;
     sleep_default(); // cgyeofnrzk
     mousebutton1pressed = x
    }
    if !x && mousebutton1pressed {
     // println!("release");
     // a0vbfusiba
     ss.meh.lock()?.push_event(&MyEvent::MouseButton1(false))?;
     sleep_default(); // cgyeofnrzk
     mousebutton1pressed = x
    }

    let y = event_mask.contains(KeyButMask::SHIFT);
    if y && !shift_pressed {
     ss.meh.lock()?.push_event(&MyEvent::Shift(true))?;
     sleep_default(); // cgyeofnrzk
     shift_pressed = y
    }

    if !y && shift_pressed {
     ss.meh.lock()?.push_event(&MyEvent::Shift(false))?;
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

#[derive(Clone)]
pub struct SyncStuff {
 pub meh: Arc<Mutex<MyEventHandler>>,
 pub loop_start: Arc<RwLock<()>>,
}

impl SyncStuff {
 pub fn new() -> Self {
  Self {
   meh: Arc::new(Mutex::new(MyEventHandler::new())),
   loop_start: Arc::new(RwLock::new(())),
  }
 }
}

pub fn main() {
 let args = Args::parse();

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

 let config = Config::from_args(&args);

 if config.debug {
  monitor();
 }

 // let meh = MyEventHandler::new();
 // let meh = Arc::new(Mutex::new(MyEventHandler::new()));
 let ss = SyncStuff::new();
 let loop_start_block = ss.loop_start.write();

 // let mut _stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap()); // creates mouse events
 // let mut _stdout2 = RawTerminal::from(stdout().into_raw_mode().unwrap()); // creates  ???

 if config.debug {
  monitor();
 }

 let mt = MouseThread::new(&config);
 let mtjh = mt.run(ss.clone());

 if config.debug {
  monitor2("ct");
 }

 let mut ct = ClipboardThread::new();
 let ctjh = ct.run(ss.clone());

 if config.debug {
  monitor2("ms");
 }

 let mut ms = MySignalsLoop::new();
 let _msjh = ms.run_thread(ss.clone());

 if config.debug {
  monitor2("tl");
 }

 let mut tl = TermionLoop::new();
 let _tljh = tl.run_loop(ss.clone());

 if config.debug {
  monitor2("ts");
 }

 let mut ts = TermionScreen::new(&config, ct.cbs.clone());
 let tsjh = ts.run_loop(ss.clone());

 if config.debug {
  println!("WaitForEnd start");
  monitor2("wfe");
 }

 sleep_default();
 drop(loop_start_block);

 // dbaphuses4, a0vbfusiba, x9kwvw3yj0, dt0gtu9sxm, ic4q5snjyp t 1, fddt4zu0y5 t 1 // main
 // blockt hier meh noch nicht
 WaitForEnd::new().run_blocking(ss.meh.clone());
 if config.debug {
  println!("WaitForEnd end");
  monitor2("tsjh");
 }

 tsjh.join(); // needed
 if config.debug {
  monitor2("ctjh");
 }
 ctjh.join(); // needed
 if config.debug {
  monitor2("mtjh");
 }
 mtjh.join(); // currently not needed, but possible, for the sake of tidyness
 if config.debug {
  monitor2("end");
 }

 println!("{}", AnsiGenericString::title("Clipboardstealer ended"));

 // tljh.join(); // never!, that would block here, we don't want that
 // msjh.join(); // never!, that would block here, we don't want that

 // println!("meh get_stop_threads : {}", meh.clone().lock().unwrap().get_stop_threads());
}
