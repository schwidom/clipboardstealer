use signal_hook::consts::SIGINT;
use termion::event::{Event, Key};

use std::{
 ffi::c_int,
 sync::{
  mpsc::{self, Receiver, SyncSender},
  Arc, Mutex,
 },
};

/** termion or signal_hook events provided by TermionLoop or MySignalsLoop */
#[derive(Clone, Debug, PartialEq)]
pub enum MyEvent {
 Termion(Event),
 SignalHook(c_int),  // signal_hook didn't wrap that
 MouseButton1(bool), // pressed (true) / released (false)
 Shift(bool),        // pressed (true) / released (false)
 CbInsertedPrimary,
 CbInsertedClipboard,
 Unused,
 EOF,
}

impl MyEvent {
 fn is_stop_event(&self) -> bool {
  match self {
   // TODO : yes/no dialog
   MyEvent::Termion(tev) if tev == &Event::Key(Key::Char('x')) => true,
   // both is possible
   MyEvent::Termion(tev) if tev == &Event::Key(Key::Ctrl('c')) => true,
   MyEvent::SignalHook(shev) if shev == &SIGINT => true,
   _ => false,
  }
 }
}

/** handles MyEvent events */
pub struct MyEventHandler {
 sender: SyncSender<MyEvent>,
 receiver: Arc<Mutex<Receiver<MyEvent>>>,
 mouse_button_1_is_pressed: bool,
 shift_is_pressed: bool,
 stopthreads: bool,
}

// TODO : semaphore
impl MyEventHandler {
 pub fn new() -> Self {
  let (sender, receiver) = mpsc::sync_channel(0);
  MyEventHandler {
   sender,
   receiver: Arc::new(Mutex::new(receiver)),
   mouse_button_1_is_pressed: false,
   shift_is_pressed: false,
   stopthreads: false,
  }
 }

 pub fn get_mouse_button_1_is_pressed(&self) -> bool {
  self.mouse_button_1_is_pressed
 }

 pub fn get_shift_is_pressed(&self) -> bool {
  self.shift_is_pressed
 }

 pub fn set_stop_threads(&mut self) {
  self.stopthreads = true;
  // self.sender.send(MyEvent::EOF); // blockiert
  self.sender.try_send(MyEvent::EOF);
 }

 pub fn get_stop_threads(&self) -> bool {
  self.stopthreads
 }

 pub fn push_event(&mut self, ev: &MyEvent) {
  if ev.is_stop_event() {
   self.set_stop_threads();
  } else {
   match ev {
    MyEvent::MouseButton1(pressed) => self.mouse_button_1_is_pressed = *pressed,
    MyEvent::Shift(pressed) => self.shift_is_pressed = *pressed,
    _ => {
     // println!("before send {:?}", ev);
     self.sender.try_send(ev.clone());
     // println!("after send {:?}", ev);
    }
   }
  }
 }

 pub fn get_receiver(&mut self) -> Arc<Mutex<Receiver<MyEvent>>> {
  self.receiver.clone()
 }
}
