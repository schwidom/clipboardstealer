#![allow(dead_code)]
#![allow(unused)]

use signal_hook::consts::SIGINT;
use termion::event::{Event, Key};

use std::{
 ffi::c_int,
 sync::
  mpsc::{self, Receiver, Sender}
 ,
};

use crate::libmain::MyError;
use x11_clipboard as x11;
// use x11::xcb::Atom;
use x11::Atom;

/** termion or signal_hook events provided by TermionLoop or MySignalsLoop */
#[derive(Clone, Debug, PartialEq)]
pub enum MyEvent {
 Termion(Event),
 SignalHook(c_int),  // signal_hook didn't wrap that
 MouseButton1(bool), // pressed (true) / released (false)
 Shift(bool),        // pressed (true) / released (false)
 CbInserted,
 // CbInsertedPrimary,
 // CbInsertedSecondary,
 // CbInsertedClipboard,
 Unused,
 CbChanged(Atom, Option<String>),
}

impl MyEvent {
 pub fn is_stop_event(&self) -> bool {
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
