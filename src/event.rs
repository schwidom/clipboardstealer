use signal_hook::consts::SIGINT;
use termion::event::{Event, Key};

use std::{
 ffi::c_int,
 sync::{
  mpsc::{self, Receiver, SyncSender},
  Arc, Mutex,
 },
};

use crate::libmain::MyError;

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

pub enum EventPusher<'a> {
 NothingToSend,
 ToSend {
  ev: &'a MyEvent,
  sender: SyncSender<MyEvent>,
 },
}

impl<'a> EventPusher<'a> {
 pub fn send(&self) {
  match self {
   EventPusher::NothingToSend => {}
   EventPusher::ToSend { ev, sender } => {
    // ic4q5snjyp t 6, alt t 7, fddt4zu0y5 t 8
    sender.send(ev.clone().clone());
    ()
   }
  }
 }
}

/** handles MyEvent events */
pub struct MyEventHandler {
 sender: Option<SyncSender<MyEvent>>,
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
   sender: Some(sender),
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
  // self.sender.send(MyEvent::EOF); // deadlock
  // self.sender.try_send(MyEvent::EOF); // don't send
  self.sender = None;
 }

 pub fn get_stop_threads(&self) -> bool {
  self.stopthreads
 }

 pub fn push_event<'a>(&mut self, ev: &'a MyEvent) -> Result<(), MyError> {
  // fddt4zu0y5 t 8 // MyEventHandler.push_event
  self.push_event_preparation(ev)?.send();
  Ok(())
 }

 pub fn push_event_preparation<'a>(&mut self, ev: &'a MyEvent) -> Result<EventPusher<'a>, ()> {
  if ev.is_stop_event() {
   self.set_stop_threads();
   Err(())
  } else {
   match ev {
    MyEvent::MouseButton1(pressed) => {
     self.mouse_button_1_is_pressed = *pressed;
     Ok(EventPusher::NothingToSend)
    }
    MyEvent::Shift(pressed) => {
     self.shift_is_pressed = *pressed;
     Ok(EventPusher::NothingToSend)
    }
    _ => {
     // println!("before send {:?}", ev);
     match &self.sender {
      Some(sender) => {
       // x9kwvw3yj0
       // sender.send(ev.clone());
       Ok(EventPusher::ToSend {
        ev,
        sender: sender.clone(),
       })
      }
      None => Ok(EventPusher::NothingToSend),
     }
     // println!("after send {:?}", ev);
    }
   }
  }
 }

 pub fn get_receiver(&mut self) -> Arc<Mutex<Receiver<MyEvent>>> {
  self.receiver.clone()
 }
}
