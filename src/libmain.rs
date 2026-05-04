// #![allow(dead_code)]
// #![allow(unused)]

extern crate enum_iterator;
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

use enum_iterator::all;

use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;

use xcb_1::{
 x::{KeyButMask, QueryPointer},
 ConnError, Connection,
};

use std::{
 cell::RefCell,
 cmp::min,
 collections::{HashMap, HashSet},
 env::var_os,
 ffi::OsString,
 fs::read_to_string,
 io::{stdin, stdout, Write},
 os::fd::AsFd,
 path::Path,
 rc::Rc,
 sync::{
  atomic::{AtomicBool, AtomicUsize, Ordering},
  mpsc::{self, Receiver, Sender},
  Arc, Mutex, RwLockWriteGuard,
 },
 thread,
 thread::JoinHandle,
 time::Duration,
};

use crossbeam_skiplist::SkipMap;

use crate::{
 clipboards::*,
 color_theme::ThemeColors,
 config::{sleep_default, Config, Paused},
 constants::{DISPLAY, EDITOR},
 debug::*,
 event::MyEvent,
 termionscreen::{TermionScreenFirstPage, TermionScreenPainter},
 tools::MyTime,
};

use nu_ansi_term::AnsiGenericString;

use tracing::trace;

use clap::Parser;

use crate::clipboards::cbentry::CBEntry;
use crate::clipboards::cbentry::CBEntryString;
use crate::clipboards::{CBType, ClipboardFixation, ClipboardReaderWriter};

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
pub(crate) struct Args {
 #[arg(long, help = "appends clipboard information to file")]
 pub(crate) append_ndjson_bin: Option<String>,
 #[arg(long, help = "reads clipboard information from file")]
 pub(crate) load_ndjson_bin: Vec<String>,
 #[arg(long, help = "loads clipboard information from file and appends to it")]
 pub(crate) load_and_append_ndjson_bin: Option<String>,
 #[arg(
  long,
  help = "appends clipboard information to file (JSON String format)"
 )]
 pub(crate) append_ndjson: Option<String>,
 #[arg(
  long,
  help = "reads clipboard information from file (JSON String format)"
 )]
 pub(crate) load_ndjson: Vec<String>,
 #[arg(
  long,
  help = "loads clipboard information from file and appends to it (JSON String format)"
 )]
 pub(crate) load_and_append_ndjson: Option<String>,
 #[arg(
  long,
  help = "interprets the EDITOR environment variable always as editor"
 )]
 pub(crate) editor: bool,
 #[arg(long, help = "converts bin ndjson to string ndjson (input file)")]
 pub(crate) convert_bin_ndjson: Option<String>,
 #[arg(long, help = "output file for converted bin ndjson")]
 pub(crate) to_string_ndjson: Option<String>,
 #[arg(long, help = "converts string ndjson to bin ndjson (input file)")]
 pub(crate) convert_string_ndjson: Option<String>,
 #[arg(long, help = "output file for converted string ndjson")]
 pub(crate) to_bin_ndjson: Option<String>,
 // #[arg(short, long, default_value_t = crate::color_theme::ColorTheme::default(), help = "select color theme (default, nord, solarized, dracula)")]
 // pub(crate) color_theme: crate::color_theme::ColorTheme,
 #[arg(
  short,
  long,
  default_value_t = crate::color_theme::default_color_theme_name(),
  help = "select color theme (default, nord, solarized, dracula, ...)"
 )]
 pub(crate) color_theme: String,
 #[arg(long, default_value_t = false, help = "list available color themes")]
 pub(crate) color_themes: bool,
 #[arg(long, help = "load color theme from JSON file")]
 pub(crate) load_color_theme: Option<String>,
 #[arg(long, help = "save current color theme to JSON file")]
 pub(crate) save_color_theme: Option<String>,

 #[arg(long, help = "paused")]
 pub(crate) paused: bool,

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

pub(crate) struct TicksThread {}

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
pub(crate) struct ClipboardThread {
 cbtype: CBType,
 echofree: Arc<Mutex<HashSet<Vec<u8>>>>,
}

impl ClipboardThread {
 fn new(cbtype: CBType, cfmap: &HashMap<CBType, ClipboardFixation>) -> Self {
  let echofree = cfmap[&cbtype].crw.echofree();
  // let crw = ClipboardReaderWriter::from_cbtype_with_echofree(&cbtype, echofree).unwrap();
  Self { cbtype, echofree }
 }

 fn getCrw(&self) -> ClipboardReaderWriter {
  ClipboardReaderWriter::from_cbtype_with_echofree(&self.cbtype, Arc::clone(&self.echofree))
   .unwrap()
 }

 fn refresh_asr(
  &self,
  asr: &AppStateReceiver,
  // cfmap: &HashMap<CBType, ClipboardFixation>,
 ) {
  let cbtype = self.cbtype.clone();
  let crw = self.getCrw();

  {
   {
    {
     asr.config.wait_for_external_program();
     if !asr.is_running() {
      return;
     }
     let cb_string2 = crw.crw_read_nonblocking();

     {
      // if cb_strings2[i] != cb_strings[i] && !ass.paused.is_paused() && cb_strings2[i].is_some() {}
      if !asr.paused.is_paused() && cb_string2.is_some() {
       asr
        .data
        .sender
        .send(MyEvent::CbChanged(cbtype.clone(), cb_string2.clone()))
        .unwrap();
      }
     }

     // sleep_default(); // cgyeofnrzk
    }
   }
  }
 }
 fn refresh(
  &self,
  ass: &'static AppStateSender,
  // cfmap: &HashMap<CBType, ClipboardFixation>,
 ) {
  let cbtype = self.cbtype.clone();
  let crw = self.getCrw();

  {
   {
    {
     ass.config.wait_for_external_program();
     if !ass.is_running() {
      return;
     }
     let cb_string2 = crw.crw_read_nonblocking();

     {
      // if cb_strings2[i] != cb_strings[i] && !ass.paused.is_paused() && cb_strings2[i].is_some() {}
      if !ass.paused.is_paused() && cb_string2.is_some() {
       ass
        .sender
        .send(MyEvent::CbChanged(cbtype.clone(), cb_string2.clone()))
        .unwrap();
      }
     }

     // sleep_default(); // cgyeofnrzk
    }
   }
  }
 }

 fn run(
  &self,
  ass: &'static AppStateSender,
  // cfmap: &HashMap<CBType, ClipboardFixation>,
 ) -> JoinHandle<Result<(), CbsError>> {
  // let (cbtype, crw) = (self.cbtype, self.crw).clone();
  // let selfclone = self.clone();
  let cbtype = self.cbtype.clone();
  let crw = self.getCrw();

  let thread: JoinHandle<_> = thread::spawn(move || -> Result<(), CbsError> {
   {
    loop {
     ass.config.wait_for_external_program();
     if !ass.is_running() {
      break Ok(());
     }
     let cb_string2 = crw.crw_read_blocking();

     {
      // if cb_strings2[i] != cb_strings[i] && !ass.paused.is_paused() && cb_strings2[i].is_some() {}
      if !ass.paused.is_paused() && cb_string2.is_some() {
       ass
        .sender
        .send(MyEvent::CbChanged(cbtype.clone(), cb_string2.clone()))
        .unwrap();
      }
     }

     sleep_default(); // cgyeofnrzk
    }
   }
   // else {
   //  Ok(())
   // }
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
 pub(crate) fn new() -> Self {
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
 paused: &'a Paused,
 sender: Sender<MyEvent>,
}

impl<'a> AppStateSender<'a> {
 fn new(
  config: &'static Config,
  running: &'a AtomicBool,
  paused: &'a Paused,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum StatusSeverity {
 InfoShort = 0,
 Info = 1,
 Warning = 2,
 Error = 3,
}

#[derive(Clone, Debug)]
pub(crate) struct StatusMessage {
 pub(crate) severity: StatusSeverity,
 pub(crate) time: MyTime,
 pub(crate) seqnr: usize,
 pub(crate) text: String,
}

#[derive(Default)]
pub(crate) struct StatusSeqGenerator(AtomicUsize);

impl StatusSeqGenerator {
 pub(crate) fn next(&self) -> usize {
  self.0.fetch_add(1, Ordering::Relaxed)
 }
}

#[derive(Clone, Debug)]
pub(crate) struct StatusKey {
 pub(crate) severity: StatusSeverity,
 pub(crate) time: MyTime,
 pub(crate) seqnr: usize,
}

impl Ord for StatusKey {
 fn cmp(&self, other: &Self) -> std::cmp::Ordering {
  other
   .severity
   .cmp(&self.severity)
   .then_with(|| other.time.cmp(&self.time))
   .then_with(|| other.seqnr.cmp(&self.seqnr))
 }
}

impl PartialOrd for StatusKey {
 fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
  Some(self.cmp(other))
 }
}

impl Eq for StatusKey {}

impl PartialEq for StatusKey {
 fn eq(&self, other: &Self) -> bool {
  self.severity == other.severity && self.time == other.time && self.seqnr == other.seqnr
 }
}

pub(crate) struct StatusLineHeap {
 map: Arc<SkipMap<StatusKey, StatusMessage>>,
 seq_gen: Arc<StatusSeqGenerator>,
}

impl Default for StatusLineHeap {
 fn default() -> Self {
  Self::new()
 }
}

impl StatusLineHeap {
 pub(crate) fn new() -> Self {
  Self {
   map: Arc::new(SkipMap::new()),
   seq_gen: Arc::new(StatusSeqGenerator::default()),
  }
 }

 pub(crate) fn default() -> Self {
  Self::new()
 }

 pub(crate) fn push(&self, severity: StatusSeverity, text: String) {
  let time = MyTime::now();
  let seqnr = self.seq_gen.next();
  let key = StatusKey {
   severity,
   time: time.clone(),
   seqnr,
  };

  if severity == StatusSeverity::InfoShort {
   loop {
    match self.map.front() {
     Some(some) => {
      if some.key().severity == StatusSeverity::InfoShort {
       some.remove();
      } else {
       break;
      }
     }
     None => break,
    }
   }
  }

  let _entry = self.map.insert(
   key,
   StatusMessage {
    severity,
    time,
    seqnr,
    text,
   },
  );
 }

 pub(crate) fn pop(&self) -> Option<StatusMessage> {
  (*self.map).pop_front().map(|entry| entry.value().clone())
 }

 pub(crate) fn peek(&self) -> Option<StatusMessage> {
  (*self.map).front().map(|node| node.value().clone())
 }

 pub(crate) fn len(&self) -> usize {
  self.map.len()
 }

 pub(crate) fn is_empty(&self) -> bool {
  self.map.is_empty()
 }
}

impl Clone for StatusLineHeap {
 fn clone(&self) -> Self {
  Self {
   map: Arc::clone(&self.map),
   seq_gen: Arc::clone(&self.seq_gen),
  }
 }
}

pub(crate) struct AppStateReceiverData {
 pub(crate) cbs: Clipboards,
 pub(crate) statusline_heap: StatusLineHeap,
 pub(crate) sender: Sender<MyEvent>,
}

impl AppStateReceiverData {
 pub(crate) fn new(config: &'static Config, sender: Sender<MyEvent>) -> Self {
  let mut cbs = Clipboards::new();
  let statusline_heap = StatusLineHeap::new();
  for load_ndjson in &config.load_ndjson_bin {
   let p_load_ndjson = Path::new(load_ndjson);
   if !p_load_ndjson.is_file() && Some(load_ndjson) == config.append_ndjson_bin.as_ref() {
    continue;
   }
   let content = read_to_string(p_load_ndjson);
   if let Err(err) = content {
    let err_msg = format!("Failed to open load file: {:?} - {}", p_load_ndjson, err);
    eprintln!("{}", err_msg);
    statusline_heap.push(StatusSeverity::Warning, err_msg);
    continue;
   }
   let content = content.unwrap();
   let deserializer = serde_json::Deserializer::from_str(&content);
   let svec: Vec<CBEntry> = deserializer
    .into_iter::<CBEntry>()
    .map(|x| x.unwrap())
    .collect::<Vec<_>>();

   for cbentry in svec {
    cbs.push_back(cbentry);
   }
  }
  for load_ndjson in &config.load_ndjson_string {
   let p_load_ndjson = Path::new(load_ndjson);
   if !p_load_ndjson.is_file() && Some(load_ndjson) == config.append_ndjson_string.as_ref() {
    continue;
   }
   let content = read_to_string(p_load_ndjson);
   if let Err(err) = content {
    let err_msg = format!("Failed to open load ndjson file: {:?} - {}", p_load_ndjson, err);
    eprintln!("{}", err_msg);
    statusline_heap.push(StatusSeverity::Warning, err_msg);
    continue;
   }
   let content = content.unwrap();
   let deserializer = serde_json::Deserializer::from_str(&content);
   let svec: Vec<CBEntryString> = deserializer
    .into_iter::<CBEntryString>()
    .map(|x| x.unwrap())
    .collect::<Vec<_>>();

   for string_entry in svec {
    let cbentry = CBEntry::from_json_entry(string_entry);
    cbs.push_back(cbentry);
   }
  }
  Self {
   cbs,
   statusline_heap,
   sender,
  }
 }
}

pub(crate) struct AppStateReceiver<'a> {
 running: &'a AtomicBool,
 paused: &'a Paused,
 receiver: Receiver<MyEvent>,
 config: &'static Config,
 data: AppStateReceiverData,
 tl: TermionLoop,
}

impl<'a> AppStateReceiver<'a> {
 fn new(
  config: &'static Config,
  running: &'a AtomicBool,
  paused: &'a Paused,
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
     self.data.statusline_heap.pop();
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
       self.data.sender.send(MyEvent::CbInserted);
      }

      MyEvent::Shift(pressed) => {
       event_state.shift_is_pressed = pressed;
       event_state.update_clipboard(&mut self.data.cbs);
       self.data.sender.send(MyEvent::CbInserted);
      }

      // MyEvent::CbInserted => todo!(),
      // MyEvent::Unused => todo!(),
      MyEvent::CbChanged(cbtype, string) => {
       event_state.cb_changed = Some((cbtype, string));
       event_state.update_clipboard(&mut self.data.cbs);
       self.data.sender.send(MyEvent::CbInserted);
      }

      MyEvent::Tick => {
       if let Some(append_filename_string) = &self.config.append_ndjson_bin {
        if let Err(err_msg) = self.data.cbs.append_ndjson_bin(append_filename_string) {
         self
          .data
          .statusline_heap
          .push(StatusSeverity::Error, err_msg);
        }
       }
       if let Some(append_filename_string) = &self.config.append_ndjson_string {
        if let Err(err_msg) = self.data.cbs.append_ndjson_string(append_filename_string) {
         self
          .data
          .statusline_heap
          .push(StatusSeverity::Error, err_msg);
        }
       }
      }

      MyEvent::TogglePause => {
       self.toggle_paused();
      }
      _ => {}
     }
    }
   }
  }
  self.set_stopping(); // just in case someone breaks abifadosqa

  so.flush().unwrap();
 }

 fn toggle_paused(&self) -> bool {
  let current = self.paused.is_paused();
  self.paused.toggle();
  // current <=> "if ! paused", wenn die pause aufgehoben ist X11 clipboard fixations neu schreiben
  if current {
   for cbtype in all::<CBType>() {
    let ct = ClipboardThread::new(cbtype, &self.data.cbs.cfmap);
    // ct.refresh(&appstate.ass);
    // ct.refresh(&self.data.sender);
    // ct.refresh(&self.data.ass.sender);
    ct.refresh_asr(self);
   }
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
  // let paused: &mut AtomicBool = Box::leak(Box::new(AtomicBool::new(config.paused)));
  Self {
   ass: AppStateSender::new(config, running, &config.paused, sender.clone()),
   asr: AppStateReceiver::new(config, running, &config.paused, receiver, sender),
  }
 }
}

pub fn main() {
 let args = Args::parse();
 // q3jhk95ow6
 if args.load_and_append_ndjson_bin.is_some() && args.append_ndjson_bin.is_some() {
  eprintln!("Error: --load-and-append-ndjson-bin cannot be used together with --append-ndjson-bin");
  std::process::exit(1);
 }

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

 // Handle --load-color-theme - load theme from JSON and use it
 let custom_theme_colors = if let Some(path) = &args.load_color_theme {
  match std::fs::read_to_string(path) {
   Ok(content) => match ThemeColors::from_json(&content) {
    Ok(theme_colors) => {
     println!("Loaded theme from {}", path);
     Some(theme_colors)
    }
    Err(e) => {
     eprintln!("Error parsing theme file: {}", e);
     std::process::exit(1);
    }
   },
   Err(e) => {
    eprintln!("Error reading theme file: {}", e);
    std::process::exit(1);
   }
  }
 } else {
  None
 };

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

 // Handle conversion options before creating Config
 if let (Some(input), Some(output)) = (&args.convert_bin_ndjson, &args.to_string_ndjson) {
  if input == output {
   eprintln!("Error: input and output files must be different");
   std::process::exit(1);
  }
  if let Err(e) = crate::log_conversion::convert_bin_to_string(input, output) {
   eprintln!("Conversion failed: {}", e);
   std::process::exit(1);
  }
  return;
 }

 if let (Some(input), Some(output)) = (&args.convert_string_ndjson, &args.to_bin_ndjson) {
  if input == output {
   eprintln!("Error: input and output files must be different");
   std::process::exit(1);
  }
  if let Err(e) = crate::log_conversion::convert_string_to_bin(input, output) {
   eprintln!("Conversion failed: {}", e);
   std::process::exit(1);
  }
  return;
 }

 if let (Some(input), Some(output)) = (&args.convert_bin_ndjson, &args.to_bin_ndjson) {
  if input == output {
   eprintln!("Error: input and output files must be different");
   std::process::exit(1);
  }
  if let Err(e) = crate::log_conversion::copy_bin(input, output) {
   eprintln!("Copy failed: {}", e);
   std::process::exit(1);
  }
  return;
 }

 if let (Some(input), Some(output)) = (&args.convert_string_ndjson, &args.to_string_ndjson) {
  if input == output {
   eprintln!("Error: input and output files must be different");
   std::process::exit(1);
  }
  if let Err(e) = crate::log_conversion::copy_string(input, output) {
   eprintln!("Copy failed: {}", e);
   std::process::exit(1);
  }
  return;
 }

 let config = Box::leak(Box::new(Config::from_args(&args)));

 {
  let mut color_theme = args.color_theme.clone();

  if let Some(value) = custom_theme_colors {
   let custom_color_name = String::from("custom");
   config
    .all_color_themes
    .insert(custom_color_name.clone(), value.clone());
   color_theme = custom_color_name;
  }

  if let Some(tc) = config.all_color_themes.get(&color_theme) {
   config.color_theme.set(tc.value().clone());
  }
 }

 // Handle --save-color-theme early exit (just save and exit)
 if let Some(path) = &args.save_color_theme {
  let json = config.color_theme.get_or_default().to_json();
  if let Err(e) = std::fs::write(path, &json) {
   eprintln!("Error saving theme file: {}", e);
   std::process::exit(1);
  }
  println!("Saved theme to {}", path);
  return;
 }

 // Handle --color-themes early exit
 if args.color_themes {
  println!("Available color themes:");
  for entry in &config.all_color_themes {
   println!("  {}", entry.key());
  }
  return;
 }

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

 for cbtype in all::<CBType>() {
  let ct = ClipboardThread::new(cbtype, &appstate.asr.data.cbs.cfmap);
  ct.refresh(&appstate.ass);
  let _ctjh = ct.run(&appstate.ass);
 }

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
  monitor2("ttjh.join()");
 }

 ttjh.join().unwrap().unwrap(); // needed

 // if config.debug {
 //  monitor2("ctjh.join()");
 // }

 // ctjh.join().unwrap().unwrap(); // no longer needed (is blocking)

 if config.debug {
  monitor2("mtjh.join()");
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

#[cfg(test)]
mod tests {
 use crate::libmain::{StatusLineHeap, StatusSeverity};

 #[test]
 fn test_statusline_1() {
  let slh = StatusLineHeap::new();
  slh.push(StatusSeverity::Error, "error".into());
  assert_eq!(slh.peek().unwrap().text, "error");
  slh.push(StatusSeverity::Warning, "warn".into());
  assert_eq!(slh.peek().unwrap().text, "error");
  slh.push(StatusSeverity::Info, "info".into());
  assert_eq!(slh.peek().unwrap().text, "error");
  assert_eq!(slh.pop().unwrap().text, "error");
  assert_eq!(slh.pop().unwrap().text, "warn");
  assert_eq!(slh.pop().unwrap().text, "info");
 }

 #[test]
 fn test_statusline_2() {
  let slh = StatusLineHeap::new();
  slh.push(StatusSeverity::Info, "info".into());
  assert_eq!(slh.peek().unwrap().text, "info");
  slh.push(StatusSeverity::Warning, "warn".into());
  assert_eq!(slh.peek().unwrap().text, "warn");
  slh.push(StatusSeverity::Error, "error".into());
  assert_eq!(slh.peek().unwrap().text, "error");
  assert_eq!(slh.pop().unwrap().text, "error");
  assert_eq!(slh.pop().unwrap().text, "warn");
  assert_eq!(slh.pop().unwrap().text, "info");
 }
 #[test]
 fn test_statusline_3() {
  let slh = StatusLineHeap::new();
  slh.push(StatusSeverity::Info, "info1".into());
  assert_eq!(slh.peek().unwrap().text, "info1");
  slh.push(StatusSeverity::Info, "info2".into());
  assert_eq!(slh.peek().unwrap().text, "info2");
  assert_eq!(slh.pop().unwrap().text, "info2");
  assert_eq!(slh.pop().unwrap().text, "info1");
 }
}
