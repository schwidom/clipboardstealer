// #![allow(dead_code)]
// #![allow(unused)]

// extern crate clipboard;
extern crate clap;
extern crate termion;
extern crate x11_clipboard;

use ratatui::{self, DefaultTerminal};

use termion::{
 cursor::{Hide, Show},
 event::{Event, Key},
 input::TermRead,
 is_tty,
};

use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;

use xcb_1::{
 x::{KeyButMask, QueryPointer},
 ConnError, Connection,
};

use std::{
 cell::RefCell,
 cmp::min,
 collections::{BinaryHeap, HashMap, HashSet},
 env::var_os,
 ffi::OsString,
 fs::read_to_string,
 io::Write,
 os::fd::AsFd,
 path::Path,
 rc::Rc,
 sync::{
  mpsc::{self, Receiver, Sender},
  RwLockWriteGuard,
 },
 thread::JoinHandle,
 time::Duration,
};

use std::{
 io::{stdin, stdout},
 thread,
};

use crate::{
 clipboards::*,
 config::{sleep_default, Config},
 constants::{DISPLAY, EDITOR},
 debug::*,
 event::MyEvent,
 termionscreen::{TermionScreenFirstPage, TermionScreenPainter},
};

use nu_ansi_term::AnsiGenericString;

use tracing::trace;

use clap::Parser;

use std::sync::{
 atomic::{AtomicBool, Ordering},
 Arc, Mutex,
};

use crate::clipboards::{CBType, ClipboardFixation, ClipboardReaderWriter};
use crate::clipboards::cbentry::CBEntry;

/// Drain any pending input from stdin to clear escape sequences from external editors
// fn drain_stdin() { // works randomly
//  use std::os::unix::io::AsRawFd;
//  use std::os::unix::io::RawFd;

//  let stdin_fd: RawFd = stdin().as_raw_fd();
//  let flags;
//  let nonblocking_flags;
//  unsafe {
//   flags = libc::fcntl(stdin_fd, libc::F_GETFL);
//   nonblocking_flags = flags | libc::O_NONBLOCK;
//   libc::fcntl(stdin_fd, libc::F_SETFL, nonblocking_flags);
//  }

//  let mut buf = [0u8; 256];
//  loop {
//   let n;
//   unsafe {
//    n = libc::read(stdin_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len());
//   }
//   if n <= 0 {
//    break;
//   }
//  }

//  unsafe {
//   libc::fcntl(stdin_fd, libc::F_SETFL, flags);
//  }
// }

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
 #[arg(long, help = "appends clipboard information to file")]
 pub(crate) append_ndjson: Option<String>,
 #[arg(long, help = "reads clipboard information from file")]
 pub(crate) load_ndjson: Vec<String>,
 #[arg(long, help = "loads clipboard information from file and appends to it")]
 pub(crate) load_and_append_ndjson: Option<String>,
 #[arg(
  long,
  help = "interprets the EDITOR environment variable always as editor"
 )]
 pub(crate) editor: bool,
 #[arg(long, default_value_t = false, help = "provides debug information")]
 pub(crate) debug: bool,
 #[arg(long, help = "writes debug information into file")]
 pub(crate) debugfile: Option<String>,
}

#[derive(Debug)]
pub(crate) enum CbsError {
 PoisonError,
 UnitError,
 // X11Clipboard(X11Error),
 DISPLAY(x11_clipboard::error::Error),
 ConnError(ConnError),
}

// impl From<()> for MyError {
//  fn from(value: ()) -> Self {
//   MyError::UnitError
//  }
// }
impl From<x11_clipboard::error::Error> for CbsError {
 fn from(value: x11_clipboard::error::Error) -> Self {
  Self::DISPLAY(value)
 }
}

impl From<ConnError> for CbsError {
 fn from(value: ConnError) -> Self {
  Self::ConnError(value)
 }
}

pub struct TicksThread {}

impl TicksThread {
 fn new() -> Self {
  Self {}
 }

 fn run(&mut self, ass: &'static AppStateSender) -> JoinHandle<Result<(), CbsError>> {
  let thread: JoinHandle<_> = thread::spawn(move || -> Result<(), CbsError> {
   loop {
    ass.config.wait_for_external_program();
    if !ass.is_running() {
     break Ok(());
    }

    ass.sender.send(MyEvent::Tick).unwrap();

    thread::sleep(Duration::from_millis(300));
   }
  });
  thread
 }
}

/** waits for clipboard events and handles them */
pub struct ClipboardThread {}

impl ClipboardThread {
 fn new() -> Self {
  Self {}
 }

 fn run(
  &mut self,
  ass: &'static AppStateSender,
  cfmap: &HashMap<CBType, ClipboardFixation>,
 ) -> JoinHandle<Result<(), CbsError>> {
  let echofree_vec: Vec<(CBType, Arc<Mutex<HashSet<Vec<u8>>>>)> = cfmap
   .iter()
   .map(|(cbtype, cf)| (cbtype.clone(), cf.crw.echofree()))
   .collect();

  let thread: JoinHandle<_> = thread::spawn(move || -> Result<(), CbsError> {
   let crws: Vec<ClipboardReaderWriter> = echofree_vec
    .iter()
    .filter_map(|(cbtype, echofree): &(CBType, Arc<Mutex<HashSet<Vec<u8>>>>)| {
     ClipboardReaderWriter::from_cbtype_with_echofree(cbtype, echofree.clone()).ok()
    })
    .collect();

   if !crws.is_empty() {
    // let mut cb_strings: Vec<_> = crws.iter().map(|x| x.read()).collect();
    let mut cb_strings: Vec<_> = crws.iter().map(|_| None).collect();

    loop {
     ass.config.wait_for_external_program();
     if !ass.is_running() {
      break Ok(());
     }

     let cb_strings2: Vec<_> = crws
      .iter()
      .map(|x: &ClipboardReaderWriter| x.crw_read())
      .collect();

     for i in 0..cb_strings.len() {
      if cb_strings2[i] != cb_strings[i] && !ass.is_paused() && cb_strings2[i].is_some() {
       ass
        .sender
        .send(MyEvent::CbChanged(crws[i].cbtype(), cb_strings2[i].clone()))
        .unwrap();
       cb_strings[i] = cb_strings2[i].clone();
      }
     }

     // cb_strings = cb_strings2;

     sleep_default(); // cgyeofnrzk
    }
   } else {
    Ok(())
   }
  });
  thread
 }
}

/// blocking
struct TermionLoop {
 // stdout_raw: Option<RawTerminal<Stdout>>,
 terminal: Option<DefaultTerminal>, // DefaultTerminal = Terminal<CrosstermBackend<Stdout>>
}

impl TermionLoop {
 fn create_raw_mode(&mut self) {
  // // restores terminal state after a zsh exit
  let magic = "\x1b[?1l\x1b>"; // DECCKM off (normal arrows) + DECKPNM (normal keypad)
  println!("{}", magic);
  // let stdout_raw = stdout().into_raw_mode();

  // self.stdout_raw.insert(stdout_raw.unwrap());
  let _ = self.terminal.insert(ratatui::init());
  // self.terminal.as_mut().unwrap().flush();
 }

 fn new() -> Self {
  let magic = "\x1b[?1l\x1b>"; // DECCKM off (normal arrows) + DECKPNM (normal keypad)
  println!("{}", magic);
  Self {
   terminal: Some(ratatui::init()),
  }
 }

 fn run_loop(&mut self, ass: &'static AppStateSender) -> JoinHandle<Result<(), CbsError>> {
  thread::spawn(move || -> Result<(), CbsError> {
   loop {
    if ass.config.is_blocked_for_external_program() {
     sleep_default();
     continue;
    }
    {
     let stdin = stdin();
     let stdin_guard = stdin.lock();

     for e in stdin_guard.events() {
      let u = e.unwrap();
      ass.sender.send(MyEvent::Termion(u.clone())).unwrap();
      // this timeout ensures that we don't block stdin in case it is an external editor
      if u == Event::Key(Key::Char('e')) {
       // when the editor is entered
       thread::sleep(Duration::from_millis(100)); // the most expensive line so far in coding time
      }
      if ass.config.is_blocked_for_external_program() {
       break;
      }
      // ass.config.wait_for_external_program(); // poacutopn4
      if !ass.is_running() {
       break;
      }
      sleep_default(); // cgyeofnrzk
     }
    }

    // ass.config.wait_for_external_program(); // poacutopn4
    if !ass.is_running() {
     break;
    }
   }

   Ok(())
  })
 }

 fn suspend_raw_mode(&mut self) {
  self.terminal = None;
  ratatui::restore();
 }
}

/// sends SIGWINCH, SIGINT events
struct MySignalsLoop {}

impl MySignalsLoop {
 pub fn new() -> Self {
  Self {}
 }

 fn run_thread(&mut self, ass: &'static AppStateSender) -> JoinHandle<Result<(), CbsError>> {
  let mut signals = Signals::new([SIGWINCH, SIGINT]).unwrap();

  thread::spawn(move || -> Result<(), CbsError> {
   for signal in &mut signals {
    {
     ass.config.wait_for_external_program();
     if !ass.is_running() {
      break;
     }
     // trace!( "MySignalsLoop: {:?}", signal);
     ass.sender.send(MyEvent::SignalHook(signal)).unwrap();
    }
    sleep_default(); // cgyeofnrzk
   }
   Ok(())
  })
 }
}

struct MouseThread<'a> {
 config: &'a Config,
}

impl<'a> MouseThread<'a> {
 fn new(config: &'a Config) -> Self {
  Self { config }
 }
 fn run(&self, ass: &'static AppStateSender) -> JoinHandle<Result<(), CbsError>> {
  let debug = self.config.debug;
  let thread = thread::spawn(move || -> Result<(), CbsError> {
   // TODO : clean up the unwrap
   let displayname: String = var_os(DISPLAY)
    .unwrap_or(OsString::from(""))
    .to_string_lossy()
    .into();
   let (connection, preferred_screen) = Connection::connect(Some(&displayname))?;
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
    ass.config.wait_for_external_program();
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
 config: &'static Config,
 running: &'a AtomicBool,
 paused: &'a AtomicBool,
 sender: Sender<MyEvent>,
}

impl<'a> AppStateSender<'a> {
 fn new(
  config: &'static Config,
  running: &'a AtomicBool,
  paused: &'a AtomicBool,
  sender: Sender<MyEvent>,
 ) -> Self {
  Self {
   config,
   running,
   paused,
   sender,
  }
 }

 fn is_running(&self) -> bool {
  self.running.load(Ordering::Relaxed)
 }

 fn is_paused(&self) -> bool {
  self.paused.load(Ordering::Relaxed)
 }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusSeverity {
 Info = 0,
 Warning = 1,
 Error = 2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusMessage {
 pub severity: StatusSeverity,
 pub text: String,
}

impl Ord for StatusMessage {
 fn cmp(&self, other: &Self) -> std::cmp::Ordering {
  self.severity.cmp(&other.severity)
 }
}

impl PartialOrd for StatusMessage {
 fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
  Some(self.cmp(other))
 }
}

pub struct AppStateReceiverData {
 pub cbs: Clipboards,
 pub statusline_heap: Rc<RefCell<BinaryHeap<StatusMessage>>>,
 pub sender: Sender<MyEvent>,
}

impl AppStateReceiverData {
 pub fn new(config: &'static Config, sender: Sender<MyEvent>) -> Self {
  let mut cbs = Clipboards::new();
  let mut statusline_heap = BinaryHeap::new();
  for load_ndjson in &config.load_ndjson {
   let p_load_ndjson = Path::new(load_ndjson);
   // no error message if the file don't already exist but is intended to get created
   if !p_load_ndjson.is_file() && Some(load_ndjson) == config.append_ndjson.as_ref() {
    continue;
   }
   let content = read_to_string(p_load_ndjson);
   let content = match content {
    Ok(content) => content,
    Err(err) => {
     let err_msg = format!("Failed to open load file: {:?} - {}", p_load_ndjson, err);
     eprintln!("{}", err_msg);
     statusline_heap.push(StatusMessage {
      severity: StatusSeverity::Warning,
      text: err_msg,
     });
     continue;
    }
   };
   let deserializer = serde_json::Deserializer::from_str(&content);
   let mut svec: Vec<CBEntry> = deserializer
    .into_iter::<CBEntry>()
    .map(|x| x.unwrap())
    .collect::<Vec<_>>();

   svec.reverse();

   for cbentry in svec {
    cbs.cbentries.push_back(AppendedCBEntry {
     appended: true,
     cbentry: Rc::new(RefCell::new(cbentry)),
     seq: cbs.seq_counter,
    });
    cbs.seq_counter += 1;
   }
  }
  Self {
   cbs,
   statusline_heap: Rc::new(RefCell::new(statusline_heap)),
   sender,
  }
 }
}

pub struct AppStateReceiver<'a> {
 running: &'a AtomicBool,
 paused: &'a AtomicBool,
 receiver: Receiver<MyEvent>,
 config: &'static Config,
 data: AppStateReceiverData,
 tl: TermionLoop,
}

impl<'a> AppStateReceiver<'a> {
 fn new(
  config: &'static Config,
  running: &'a AtomicBool,
  paused: &'a AtomicBool,
  receiver: Receiver<MyEvent>,
  sender: Sender<MyEvent>,
 ) -> Self {
  let data = AppStateReceiverData::new(config, sender);
  Self {
   running,
   paused,
   receiver,
   config,
   data,
   tl: TermionLoop::new(),
  }
 }

 fn is_running(&self) -> bool {
  self.running.load(Ordering::Relaxed)
 }

 fn set_stopping(&self) {
  self.running.store(false, Ordering::Relaxed);
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
   cb_changed: Option<(CBType, Option<Vec<u8>>)>,
  }

  let mut event_state = EventState::default();

  impl EventState {
   fn update_clipboard(&mut self, cbs: &mut Clipboards) {
    if !self.mouse_button_1_is_pressed && !self.shift_is_pressed {
     if let Some((atom, string)) = &self.cb_changed {
      cbs.insert(atom, string.clone());
      self.cb_changed = None;
     }
    }
   }
  }

  let mut so = stdout();

  // NOTE: geht nicht in assd (self.data), da vermutlich das terminal gedroppt sein muss am Ende, was aber nicht passiert, wenn es im geleakten 'static appstate ist.
  // Nach Ende der loop hier wird es aber regulär gedroppt
  // Meine Vermutung ist korrekt
  // let mut terminal = ratatui::init();
  // let terminal = &mut self.data.tl.terminal.as_mut().unwrap();

  let mut is_external_program: Option<RwLockWriteGuard<'_, ()>> = None;

  loop {
   let mut current_painter = tsp_stack.last().unwrap_or(&tsp_default).borrow_mut(); // pfna784hof

   // if current_painter.is_external_program() { panic!();} // kommt

   if current_painter.is_external_program() {
    // panic!(); // kommt bei edit
    // poacutopn4
    let _ = is_external_program.insert(self.config.block_threads_for_external_program()); // blocks threads
    print!("{}", Show); // doesn't get restored by ratatui::restore()
                        // self.data.tl.suspend_raw_mode();
                        // stdout().flush().unwrap();
                        // ratatui::restore();
    self.tl.suspend_raw_mode();
   }

   match &mut self.tl.terminal.as_mut() {
    Some(terminal) => {
     current_painter.paint(terminal, &mut self.data);
    }
    None => {
     current_painter.paint_without_terminal(&mut self.data);
    }
   }

   if current_painter.is_external_program() {
    // panic!(); // kommt nach beenden des Editors
    // poacutopn4
    drop(current_painter);
    // terminal = ratatui::init();

    // full reset sequence
    // print!("{}", termion::cursor::Show);
    // print!("{}", termion::clear::All);
    // print!("{}", termion::cursor::Goto(1,1));
    // print!("{}", termion::style::Reset);
    // stdout().flush().unwrap();

    self.tl.create_raw_mode();
    print!("{}", Hide);

    // continues polling
    is_external_program = None;

    tsp_stack.pop();
    continue;
   }

   // if current_painter.is_sticky_dialog() { panic!();}

   print!("{}", Hide);
   so.flush().unwrap();
   let ev = match self.receiver.recv() {
    Ok(ev) => ev,
    Err(_) => {
     break; // abifadosqa
    }
   };
   if self.config.debug {
    trace!("ev: {:?}", ev);
   }
   let is_sticky = current_painter.is_sticky_dialog();
   let next_tsp = current_painter.handle_event(&ev, &mut self.data);
   drop(current_painter);
   if let MyEvent::Termion(Event::Key(Key::Esc)) = ev {
    if !is_sticky && !matches!(next_tsp, crate::termionscreen::NextTsp::IgnoreBasicEvents) {
     let mut statusline = self.data.statusline_heap.borrow_mut();
     statusline.pop();
    }
   }
   let mut ignore_basic_events = false;
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
    crate::termionscreen::NextTsp::Quit => {
     break; // abifadosqa
    }
    crate::termionscreen::NextTsp::PopThis => {
     tsp_stack.pop();
    }
    crate::termionscreen::NextTsp::IgnoreBasicEvents => {
     ignore_basic_events = true;
    }
   }
   if true {
    if ev.is_stop_event() && !ignore_basic_events {
     let tsp_before = Rc::clone(tsp_stack.last().unwrap_or(&tsp_default)); // pfna784hof

     if !tsp_before.borrow().is_sticky_dialog() {
      tsp_stack.push(Rc::new(RefCell::new(
       crate::termionscreen::TermionScreenStatusBarDialogYN::new(
        self.config,
        tsp_before,
        "exit? y/n".to_string(),
       ),
      )));
     }
    } else {
     match ev {
      MyEvent::Termion(Event::Key(Key::Char('q'))) if !ignore_basic_events => {
       let tsp_before = Rc::clone(tsp_stack.last().unwrap_or(&tsp_default)); // pfna784hof
       if !tsp_before.borrow().is_sticky_dialog() {
        if tsp_stack.is_empty() {
         tsp_stack.push(Rc::new(RefCell::new(
          crate::termionscreen::TermionScreenStatusBarDialogYN::new(
           self.config,
           tsp_before,
           "exit? y/n".to_string(),
          ),
         )));
        } else {
         tsp_stack.pop();
        }
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
      MyEvent::CbChanged(cbtype, string) => {
       event_state.cb_changed = Some((cbtype, string));
       event_state.update_clipboard(&mut self.data.cbs);
      }

      MyEvent::Tick => {
       if let Some(append_ndjson_filename) = &self.config.append_ndjson {
        if let Err(err_msg) = self.data.cbs.append_ndjson(append_ndjson_filename) {
         self.data.statusline_heap.borrow_mut().push(StatusMessage {
          severity: StatusSeverity::Error,
          text: err_msg,
         });
        }
       }
      }

      MyEvent::TogglePause => {
       self.toggle_paused();
       self
        .data
        .sender
        .send(MyEvent::TogglePauseResult(self.is_paused()))
        .unwrap();
      }
      _ => {}
     }
    }
   }
  }
  self.set_stopping(); // just in case someone breaks abifadosqa

  so.flush().unwrap();
 }

 fn is_paused(&self) -> bool {
  self.paused.load(Ordering::Relaxed)
 }

 fn toggle_paused(&self) -> bool {
  let current = self.is_paused();
  self.paused.store(!current, Ordering::Relaxed);
  // current <=> "if ! paused", wenn die pause aufgehoben ist X11 clipboard fixations neu schreiben
  if current {
   self.data.cbs.refresh_fixation();
  }
  !current
 }
}

struct AppState<'a> {
 ass: AppStateSender<'a>,
 asr: AppStateReceiver<'a>,
}

impl<'a> AppState<'a> {
 fn new(config: &'static Config) -> Self {
  let (sender, receiver) = mpsc::channel();
  let running: &mut AtomicBool = Box::leak(Box::new(AtomicBool::new(true)));
  let paused: &mut AtomicBool = Box::leak(Box::new(AtomicBool::new(false)));
  Self {
   ass: AppStateSender::new(config, running, paused, sender.clone()),
   asr: AppStateReceiver::new(config, running, paused, receiver, sender),
  }
 }
}

pub fn main() {
 let args = Args::parse();
 // q3jhk95ow6
 if args.load_and_append_ndjson.is_some() && args.append_ndjson.is_some() {
  eprintln!("Error: --load-and-append-ndjson cannot be used together with --append-ndjson");
  std::process::exit(1);
 }

 {
  // aborts
  let mut exit = false;

  if false {
   // TODO : better usability checks for DISPLAY
   if var_os(DISPLAY).is_none() || var_os(DISPLAY) == Some("".into()) {
    // eprintln!("Environment variable DISPLAY is not set, program may misbehave");
    // thread::sleep(Duration::from_millis(500));
    eprintln!("Environment variable DISPLAY is not set");
    exit = true;
   }
  }

  if exit {
   eprintln!("exiting");
   return;
  }
 }

 {
  // warnings

  let mut delay: u64 = 0;

  // TODO : better usability checks for DISPLAY
  if var_os(DISPLAY).is_none() || var_os(DISPLAY) == Some("".into()) {
   eprintln!("Environment variable DISPLAY is not set, program may misbehave");
   delay += 1;
  }
  // TODO : reactivate linuxeditor first, then editor
  if var_os(EDITOR).is_none() || var_os(EDITOR) == Some("".into()) {
   eprintln!("Environment variable EDITOR not set, editor may not be available");
   delay += 1;
  }
  if delay > 0 {
   thread::sleep(Duration::from_millis(min(5000, 1000 * delay)));
  }
 }

 let config = Box::leak(Box::new(Config::from_args(&args)));

 // let appstate = AppState::new();
 // let appstate = Box::leak(Box::new(AppState::new(config)));
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

 let mt = MouseThread::new(config);
 let mtjh = mt.run(&appstate.ass);

 if config.debug {
  monitor2("ct");
 }

 let mut tt = TicksThread::new();
 let ttjh = tt.run(&appstate.ass);

 let mut ct = ClipboardThread::new();
 let ctjh = ct.run(&appstate.ass, &appstate.asr.data.cbs.cfmap);

 if config.debug {
  monitor2("ms");
 }

 let mut ms = MySignalsLoop::new();
 let _msjh = ms.run_thread(&appstate.ass);

 if config.debug {
  monitor2("tl");
 }

 // let mut tl = TermionLoop::new();
 // let _tljh = tl.run_loop(&appstate.ass);
 let _tljh = &appstate.asr.tl.run_loop(&appstate.ass);

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

 ttjh.join().unwrap().unwrap(); // needed

 ctjh.join().unwrap().unwrap(); // needed

 if config.debug {
  monitor2("mtjh");
 }

 mtjh.join().unwrap().unwrap(); // currently not needed, but possible, for the sake of tidyness

 if config.debug {
  monitor2("end");
 }

 // drop(appstate);
 // thread::sleep(Duration::from_millis(500));
 // poacutopn4
 print!("{}", Show); // doesn't get restored by ratatui::restore()
                     // tl.suspend_raw_mode();
 appstate.asr.tl.suspend_raw_mode();

 // stdout().flush().unwrap();
 // ratatui::restore();
 println!("{}", AnsiGenericString::title("Clipboardstealer ended"));

 // tljh.join(); // never!, that would block here, we don't want that
 // msjh.join(); // never!, that would block here, we don't want that
}
