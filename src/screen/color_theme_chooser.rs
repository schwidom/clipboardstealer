use crate::pager::Pager;

use crate::libmain::StatusSeverity;

use ratatui::text::Line;
use termion::event::Event;
use termion::event::Key;

use super::constants;
use super::NextTsp;

use crate::event::MyEvent;

use super::ActiveArea;

use super::TwoScreenDefaultWidget;

use super::LineStringsConfig;

use super::LineStringsType;

use ratatui::style::Style;

use ratatui::text::Span;

use ratatui::style::Color;

use super::LineStrings;

use ratatui::layout::Margin;

use crate::layout_ratatui::PagerLayoutBase;

use super::RatatuiVariables;

use crate::libmain::AppStateReceiverData;

use ratatui::DefaultTerminal;

use super::ScreenPainter;

use crate::color_theme::ThemeColors;

use crate::scroller::Scroller;

use crate::config::Config;

pub(crate) struct ScreenColorThemeChooser {
 pub(crate) config: &'static Config,
 pub(crate) scroller: Scroller,
 pub(crate) themes: Vec<(String, ThemeColors)>,
}

impl ScreenColorThemeChooser {
 pub(crate) fn new(config: &'static Config) -> Self {
  // let themes: Vec<(String, ColorTheme)> = ColorTheme::all_themes()
  //  .iter()
  //  .map(|(name, theme)| (name.to_string(), *theme))
  //  .collect();
  let themes: Vec<(String, ThemeColors)> = config
   .all_color_themes
   .iter()
   .map(|x| (x.key().clone(), x.value().clone()))
   .collect::<Vec<_>>();

  Self {
   config,
   scroller: Scroller::new(),
   themes,
  }
 }

 pub(crate) fn total_entries(&self) -> usize {
  self.config.all_color_themes.len()
 }
}

impl ScreenPainter for ScreenColorThemeChooser {
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let rv = RatatuiVariables::new::<PagerLayoutBase>(terminal);

  {
   let inner_main_rect = rv.pl.get_main_area().inner(Margin::new(1, 1));

   let total = self.total_entries();
   self.scroller.set_content_length(total);
   self
    .scroller
    .set_windowlength(inner_main_rect.height as usize);

   let theme_colors = self.config.color_theme.get_or_default();

   let cursor_in_window = self.scroller.get_cursor_in_window();
   let window_position = self.scroller.get_windowposition();

   // let themes_count = self.config.all_color_themes.len();

   let mut lines: Vec<LineStrings> = Vec::new();

   for idx in self.scroller.get_safe_windowrange() {
    let is_cursor = match cursor_in_window {
     None => false,
     Some(value) => idx == window_position + value,
    };
    let cursor_star = if is_cursor { ">" } else { " " };

    let name = self.themes[idx].0.clone();

    let tc = self.themes[idx].1.clone();

    let swatch = |c: Option<Color>| -> Span {
     match c {
      Some(c) => Span {
       style: Style::new().bg(c).fg(c),
       content: "██".into(),
      }, // "██" .bg(c).fg(c),
      None => Span {
       style: Style::new(),
       content: "░░".into(),
      },
     }
    };
    let swatches = vec![
     Span {
      style: Style::new(),
      content: format!(" {:15}", name).into(),
     },
     swatch(tc.window_bg),
     swatch(tc.window_fg),
     swatch(tc.cursor),
     swatch(tc.cursor_inactive),
     swatch(tc.line_number),
     swatch(tc.text),
     swatch(tc.border),
     swatch(tc.border_inactive),
     swatch(tc.menu),
    ];

    lines.push(LineStrings {
     wrapped: false,
     cursor: cursor_star.to_string(),
     line_number: "".to_string(),
     //  text: format!("{}   {}", name, swatches),
     text: LineStringsType::L(vec![Line::default().spans(swatches)]),
    });
   }

   let all_lines = LineStringsConfig {
    line_strings: &lines,
    wrapped: false,
    title: "Color Theme",
    line_count: Some(total),
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

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
  match evt {
   MyEvent::Termion(Event::Key(Key::Char('\n'))) => {
    if let Some(cursor) = self.scroller.get_cursor_in_content_array() {
     if cursor < self.themes.len() {
      let theme_name = self.themes[cursor].0.clone();
      let theme = self.themes[cursor].1.clone();
      self.config.color_theme.set(theme);
      assd
       .statusline_heap
       .push(StatusSeverity::InfoShort, format!("Theme changed to {}", theme_name));
     }
     return NextTsp::NoNextTsp;
    }
   }
   _ => {
    Pager::handle_event(&mut self.scroller, evt);
   }
  }
  NextTsp::NoNextTsp
 }
}
