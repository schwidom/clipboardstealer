use termion::event::{Event, Key};

use crate::{
 event::MyEvent,
 scroller::{CursorRepetitions, Scroller},
};

pub struct Pager;

impl Pager {
 pub fn handle_event(scroller: &mut Scroller, evt: &MyEvent) {
  match evt {
   MyEvent::Termion(Event::Key(Key::Up)) => {
    scroller.cursor_decrease();
   }
   MyEvent::Termion(Event::Key(Key::Down)) => {
    scroller.cursor_increase();
   }
   MyEvent::Termion(Event::Key(Key::Home)) => {
    scroller.cursor_home();
   }
   MyEvent::Termion(Event::Key(Key::End)) => {
    scroller.cursor_end();
   }
   MyEvent::Termion(Event::Key(Key::PageUp)) => {
    scroller.cursor_decrease_by(CursorRepetitions::WindowLength);
   }
   MyEvent::Termion(Event::Key(Key::PageDown)) => {
    scroller.cursor_increase_by(CursorRepetitions::WindowLength);
   }
   MyEvent::Termion(Event::Key(Key::Left)) => {
    scroller.scroll_left();
   }
   MyEvent::Termion(Event::Key(Key::ShiftLeft)) => {
    scroller.reset_hoffset();
   }
   MyEvent::Termion(Event::Key(Key::Right)) => {
    scroller.scroll_right();
   }
   MyEvent::Termion(Event::Key(Key::ShiftRight)) => {
    scroller.scroll_right_to_end();
   }
   _ => {}
  }
 }
}
