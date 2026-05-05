use crate::pager::Pager;

use super::color_theme_chooser::ScreenColorThemeChooser;
use super::constants;

use std::cell::RefCell;
use std::rc::Rc;

use termion::event::Event;
use termion::event::Key;

use super::NextTsp;

use crate::event::MyEvent;

use super::ActiveArea;

use super::TwoScreenDefaultWidget;

use super::LineStringsConfig;

use super::LineStringsType;

use super::LineStrings;

use crate::layout::Layout;

use super::render_scroller_lines4;

use ratatui::layout::Margin;

use crate::layout_ratatui::PagerLayoutBase;

use super::RatatuiVariables;

use crate::libmain::AppStateReceiverData;

use ratatui::DefaultTerminal;

use super::ScreenPainter;

use crate::scroller::Scroller;

use crate::config::Config;

pub(crate) struct ScreenMenu {
 pub(crate) config: &'static Config,
 pub(crate) scroller: Scroller,
 pub(crate) items: Vec<&'static str>,
}

impl ScreenMenu {
 pub(crate) fn new(config: &'static Config) -> Self {
  Self {
   config,
   scroller: Scroller::new(),
   items: vec!["Color Theme"],
  }
 }
}

impl ScreenPainter for ScreenMenu {
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let scroller = &mut self.scroller;

  let rv = RatatuiVariables::new::<PagerLayoutBase>(terminal);

  {
   let inner_main_rect = rv.pl.get_main_area().inner(Margin::new(1, 1));

   scroller.set_content_length(self.items.len());
   scroller.set_windowlength(inner_main_rect.height as usize);

   let all_lines = render_scroller_lines4(
    scroller,
    &self.items,
    false,
    &Layout::new(),
    |cursor_star, _idx, _numbers_width, entry| LineStrings {
     wrapped: false,
     cursor: cursor_star.to_string(),
     line_number: " ".to_string(),
     text: LineStringsType::S(entry.to_string()),
    },
   );

   let theme_colors = self.config.color_theme.get_or_default();

   let all_lines = LineStringsConfig {
    line_strings: all_lines.as_ref(),
    wrapped: false,
    title: "Menu",
    line_count: Some(self.items.len()),
    hoffset: 0,
    theme_colors: theme_colors.clone(),
    cursor_color: None,
   };

   {
    let window_wraps = all_lines
     .prepare2print(*rv.pl.get_main_area())
     .iter()
     .map(|x| x.len())
     .collect::<Vec<_>>();

    self.scroller.set_wrapped_window_length(&window_wraps);
   }

   let sw = TwoScreenDefaultWidget {
    helpline: constants::HELP_QXE,
    rv: &rv,
    all_lines,
    all_lines2: LineStringsConfig::default(),
    regex_edit_mode: None,
    regex_edit_mode_state: "".to_string(),
    regex_count: 0,
    delete_confirm_mode: None,
    statusline_heap: assd.statusline_heap.clone(),
    paused: false,
    active_area: ActiveArea::Main,
    theme_colors: theme_colors.clone(),
   };

   terminal
    .draw(|frame| frame.render_widget(sw, frame.area()))
    .unwrap();
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, _assd: &mut AppStateReceiverData) -> NextTsp {
  match evt {
   MyEvent::Termion(Event::Key(Key::Char('\n'))) => {
    if let Some(cursor) = self.scroller.get_cursor_in_content_array() {
     if cursor < self.items.len() && self.items[cursor] == "Color Theme" {
      return NextTsp::Stack(Rc::new(RefCell::new(ScreenColorThemeChooser::new(self.config))));
     }
    }
   }
   _ => {
    Pager::handle_event(&mut self.scroller, evt);
   }
  }
  NextTsp::NoNextTsp
 }
}
