use super::NextTsp;

use termion::event::{Event, Key};

use crate::event::MyEvent;

use ratatui::widgets::Paragraph;

use crate::layout_ratatui::PagerLayoutBase;

use super::RatatuiVariables;

use crate::libmain::AppStateReceiverData;

use ratatui::DefaultTerminal;

use super::ScreenPainter;

use std::cell::RefCell;

use std::rc::Rc;

use crate::config::Config;

pub(crate) struct ScreenStatusBarDialogYN {
 pub(crate) config: &'static Config,
 /// tsp_before is intended to allow the display of the previous dialog in a frozen state while the exit dialog is in effect
 ///
 /// currently is it not used
 pub(crate) tsp_before: Rc<RefCell<dyn ScreenPainter>>,
 pub(crate) question: String,
}

impl ScreenStatusBarDialogYN {
 pub(crate) fn new(
  config: &'static Config,
  tsp_before: Rc<RefCell<dyn ScreenPainter>>,
  question: String,
 ) -> Self {
  Self {
   config,
   tsp_before,
   question,
  }
 }
}

impl ScreenPainter for ScreenStatusBarDialogYN {
 fn paint(&mut self, terminal: &mut DefaultTerminal, _assd: &mut AppStateReceiverData) {
  let rv = &RatatuiVariables::new::<PagerLayoutBase>(terminal);

  // if let Some(rc) = &self.tsp_before {
  //  // rc.borrow_mut().handle_event(&MyEvent::Tick, assd);
  //  rc.borrow_mut().paint(terminal, assd);
  // }

  //  writes in the correct area but overwrites the upper part
  terminal
   .draw(|frame| {
    frame.render_widget(
     Paragraph::new(self.question.clone()),
     rv.pl.get_status_area().intersection(frame.area()),
    )
   })
   .unwrap();
 }

 fn handle_event(&mut self, evt: &MyEvent, _assd: &mut AppStateReceiverData) -> NextTsp {
  // if let Some(rc) = &self.tsp_before {
  //  // rc.borrow_mut().handle_event(evt, assd); // TODO : filter events
  // }

  match evt {
   MyEvent::Termion(Event::Key(Key::Char('y'))) => NextTsp::Quit,
   MyEvent::Termion(Event::Key(Key::Char('n'))) => NextTsp::PopThis,
   _ => NextTsp::NoNextTsp,
  }
 }

 fn is_sticky_dialog(&self) -> bool {
  true
 }
}
