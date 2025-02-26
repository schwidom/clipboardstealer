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

use xcb_1::{
 x::{KeyButMask, QueryPointer},
 Connection,
};

use std::{
 borrow::Borrow,
 fs::{File, OpenOptions},
 io::Write,
 os::fd::AsFd,
 path::PathBuf,
 str::FromStr,
 sync::LazyLock,
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

 fn run(&mut self, meh: Arc<Mutex<MyEventHandler>>) -> JoinHandle<()> {
  let cbs = self.cbs.clone();

  let thread: JoinHandle<_> = thread::spawn(move || {
   // TODO : ggf. in verschiedene threads zerlegen mit verschiedenen timeouts
   loop {
    if meh.lock().unwrap().get_stop_threads() {
     break;
    }

    sleep_default();

    if meh.lock().unwrap().get_mouse_button_1_is_pressed() {
     continue;
    }

    if meh.lock().unwrap().get_shift_is_pressed() {
     continue;
    }

    let inserted_primary = cbs
     .lock()
     .unwrap()
     .primary
     .lock()
     .unwrap()
     .process_clipboard_contents();
    let inserted_clipboard = cbs
     .lock()
     .unwrap()
     .clipboard
     .lock()
     .unwrap()
     .process_clipboard_contents();

    if inserted_primary.0 {
     meh.lock().unwrap().push_event(&MyEvent::CbInsertedPrimary);
    }
    if inserted_clipboard.0 {
     meh
      .lock()
      .unwrap()
      .push_event(&MyEvent::CbInsertedClipboard);
    }
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

 fn run_loop(&mut self, meh: Arc<Mutex<MyEventHandler>>) -> JoinHandle<()> {
  // let mut tla = self.tla.clone();
  // let mut stdout_raw = self.stdout_raw.lock().unwrap();
  // let mut stdout_raw = self.stdout_raw.clone();
  let thread = thread::spawn(move || {
   // let mut tla = tla.lock().unwrap();
   let stdin = stdin();
   for e in stdin.events() {
    if meh.lock().unwrap().get_stop_threads() {
     break;
    }
    // let mut stdout = stdout_raw.lock().unwrap();
    let u = e.unwrap();
    meh.lock().unwrap().push_event(&MyEvent::Termion(u.clone()));
   }
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

 fn run_thread(&mut self, meh: Arc<Mutex<MyEventHandler>>) -> JoinHandle<()> {
  let mut signals = Signals::new(&[SIGWINCH, SIGINT]).unwrap();
  // let handle = signals.handle();

  let thread = thread::spawn(move || {
   for signal in &mut signals {
    if meh.lock().unwrap().get_stop_threads() {
     break;
    }
    // println!("signal : {:?}", signal);
    meh.lock().unwrap().push_event(&MyEvent::SignalHook(signal));
    // match signal {
    //  SIGWINCH => {
    //   // println!("winch");
    //  }
    //  SIGINT => {
    //   // println!("int");
    //   // panic!(); // tritt nicht ein, kommt als MyEvent::Termion an ( Ctrl('c') )
    //  }
    //  _ => unreachable!(),
    // }
   }
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
   // TODO : semaphore?
   sleep_default();
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
 fn run(&self, meh: Arc<Mutex<MyEventHandler>>) -> JoinHandle<()> {
  let debug = self.config.debug;
  let thread = thread::spawn(move || {
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

   loop {
    if meh.lock().unwrap().get_stop_threads() {
     break;
    }
    sleep_default();
    let cookie = connection.send_request(&QueryPointer { window: rootwindow });
    let event = connection.wait_for_reply(cookie);
    // println!("QueryPointer 0x1de {:?}", event);
    if (false) {
     let p = PathBuf::from_str("/tmp/tmp.ceEb8WHeI9/log.txt").unwrap();
     // NOTE: without 'create' : blocks if the file dont exist
     let mut file = OpenOptions::new()
      .create(true)
      .write(true)
      .append(true)
      .open(p)
      .unwrap();
     // file.write("test\n".as_bytes());
     writeln!(&file, "QueryPointer 0x1de {:?}", event);
     file.flush();
    }
    // thats it
    // QueryPointer 0x1de Ok(QueryPointerReply { response_type: 1, same_screen: true, sequence: 26791, length: 0, root: Window { res_id: 478 }, child: Window { res_id: 16787724 }, root_x: 822, root_y: 560, win_x: 822, win_y: 560, mask: CONTROL, pad: 2 })
    // QueryPointer 0x1de Ok(QueryPointerReply { response_type: 1, same_screen: true, sequence: 53770, length: 0, root: Window { res_id: 478 }, child: Window { res_id: 16787724 }, root_x: 1094, root_y: 623, win_x: 1094, win_y: 623, mask: CONTROL | BUTTON1, pad: 2 })

    let event_mask = event.unwrap().mask();

    let x = event_mask.contains(KeyButMask::BUTTON1);
    if x && !mousebutton1pressed {
     // println!("press");
     meh.lock().unwrap().push_event(&MyEvent::MouseButton1(true));
     mousebutton1pressed = x
    }
    if !x && mousebutton1pressed {
     // println!("release");
     meh
      .lock()
      .unwrap()
      .push_event(&MyEvent::MouseButton1(false));
     mousebutton1pressed = x
    }

    let y = event_mask.contains(KeyButMask::SHIFT);
    if y && !shift_pressed {
     meh.lock().unwrap().push_event(&MyEvent::Shift(true));
     shift_pressed = y
    }

    if !y && shift_pressed {
     meh.lock().unwrap().push_event(&MyEvent::Shift(false));
     shift_pressed = y
    }
   }
  });
  thread
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
 let meh = Arc::new(Mutex::new(MyEventHandler::new()));

 // let mut _stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap()); // creates mouse events
 // let mut _stdout2 = RawTerminal::from(stdout().into_raw_mode().unwrap()); // creates  ???

 if config.debug {
  monitor();
 }

 let mt = MouseThread::new(&config);
 let mtjh = mt.run(meh.clone());

 if config.debug {
  monitor2("ct");
 }

 let mut ct = ClipboardThread::new();
 let ctjh = ct.run(meh.clone());

 if config.debug {
  monitor2("ms");
 }

 let mut ms = MySignalsLoop::new();
 let _msjh = ms.run_thread(meh.clone());

 if config.debug {
  monitor2("tl");
 }

 let mut tl = TermionLoop::new();
 let _tljh = tl.run_loop(meh.clone());

 if config.debug {
  monitor2("ts");
 }

 let mut ts = TermionScreen::new(&config, ct.cbs.clone());
 let tsjh = ts.run_loop(meh.clone());

 if config.debug {
  println!("WaitForEnd start");
  monitor2("wfe");
 }
 WaitForEnd::new().run_blocking(meh.clone());
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
