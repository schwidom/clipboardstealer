use crate::pager::Pager;

 use termion::event::Event;
 use termion::event::Key;
 use unicode_width::UnicodeWidthStr;

 use super::NextTsp;

 use crate::event::MyEvent;

 use super::ActiveArea;

 use crate::constants::HELP_WQX;

 use super::TwoScreenDefaultWidget;

 use super::LineStringsConfig;

 use super::LineStringsType;

 use super::LineStrings;

 use super::render_scroller_lines4;

 use ratatui::layout::Margin;

 use crate::layout_ratatui::PagerLayoutBase;

 use super::RatatuiVariables;

 use crate::libmain::AppStateReceiverData;

 use ratatui::DefaultTerminal;

 use super::ScreenPainter;

 use crate::clipboards::cbentry::CBEntry;

 use crate::layout::Layout;

 use crate::scroller::Scroller;

 use crate::config::Config;

 pub(crate) struct ScreenViewPage {
  pub(crate) config: &'static Config,
  pub(crate) main_title: String,
  pub(crate) scroller: Scroller,
  pub(crate) layout: Layout,
  pub(crate) text: CBEntry,
  pub(crate) wrapped: bool,
 }

 impl ScreenViewPage {
  pub(crate) fn new(config: &'static Config, main_title: String, text: CBEntry) -> Self
  where
   Self: Sized,
  {
   Self {
config,
main_title,
scroller: Scroller::new(),
layout: Layout::new(),
text,
wrapped: false,
   }
  }

  // TODO : ?
  pub(crate) fn get_max_hoffset(&self) -> usize {
   let max_line_width = self
.text
.get_text()
.iter()
.map(|l| l.width())
.max()
.unwrap_or(0);
   // let window_width = 80;
   // max_line_width.saturating_sub(window_width / 2)
   max_line_width
  }
 }

 impl ScreenPainter for ScreenViewPage {
  fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
   let scroller = &mut self.scroller;
   let layout = &mut self.layout;

   let string_lines: Vec<String> = self
.text
.get_text()
.iter()
.map(|x| x.to_string())
.collect::<Vec<_>>();

   let rv = RatatuiVariables::new::<PagerLayoutBase>(terminal);

   {
let inner_main_rect = rv.pl.get_main_area().inner(Margin::new(1, 1));
layout.set_width_height_from_rect(inner_main_rect);

scroller.set_content_length(string_lines.len());
// scroller.set_windowlength(height + 1 - layout.get_current_line());
// scroller.set_windowlength(rv.pl.get_main_area().inner(Margin::new(0, 1)).height as usize);
scroller.set_windowlength(inner_main_rect.height as usize);

// TODO : render_scroller_lines2
let all_lines = render_scroller_lines4(
 scroller,
 &string_lines,
 self.wrapped,
 layout,
 |cursor_star, idx, numbers_width, entry| {
  // format!("{} {:width$} : {}", cursor_star, idx, entry, width = numbers_width,)
  //  (format!("{} {:width$} : ", cursor_star, idx, width = numbers_width,), entry.to_string())
  LineStrings {
   wrapped: self.wrapped,
   cursor: cursor_star.to_string(),
   line_number: format!(" {:width$} : ", idx, width = numbers_width,),
   text: LineStringsType::S(entry.to_string()),
  }
 },
);
// for R::VS
// let all_lines = all_lines.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
// for R::Old
// let all_lines = all_lines.join( "\n");

let theme_colors = self.config.color_theme.get_or_default();

let all_lines = LineStringsConfig {
 line_strings: all_lines.as_ref(),
 wrapped: self.wrapped,
 title: &self.main_title,
 line_count: Some(string_lines.len()),
 hoffset: self.scroller.get_hoffset(),
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
 helpline: HELP_WQX,
 rv: &rv,
 // all_lines: R::Old(&all_lines),
 // all_lines: LineStringsConfig::New2(all_lines.as_ref())
 all_lines,
 // all_lines: LineStringsConfig {
 //  line_strings: all_lines.as_ref(),
 //  wrapped: self.wrapped,
 //  title: &self.main_title,
 //  line_count: Some(string_lines.len()),
 //  hoffset: self.scroller.get_hoffset(),
 //  theme_colors: theme_colors.clone(),
 //  cursor_color: None,
 // },
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
MyEvent::Termion(Event::Key(Key::Char('w'))) => {
 self.wrapped = !self.wrapped;
}

//  MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
_ => {
 // Pager::handle_event(&mut scroller, &evt);
 // TODO : optimize
 self.scroller.set_max_hoffset(self.get_max_hoffset());
 Pager::handle_event(&mut self.scroller, evt);
}
   }
   NextTsp::NoNextTsp
  }
 }
