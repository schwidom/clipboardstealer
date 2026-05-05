use crate::pager::Pager;

use crate::libmain::StatusSeverity;

use super::ScreenEditorPage;
use crate::config;

// use menu::ScreenMenu;

use super::ScreenMenu;
use super::ScreenViewPage;

// use super::Stack;

// use super::IgnoreBasicEvents;

use termion::event::Event;
use termion::event::Key;
use tracing::trace;
use unicode_width::UnicodeWidthStr;

// use super::NoNextTsp;

use super::NextTsp;

use crate::event::MyEvent;

use crate::constants::HELP_FIRST_PAGE;

use super::TwoScreenDefaultWidget;

use super::LineStringsConfig;

use super::render_scroller_lines4;

// use super::S;

use super::LineStringsType;

use super::LineStrings;

use std::cell::RefCell;
use std::rc::Rc;

use ratatui::layout::Margin;

use crate::layout_ratatui::PagerLayoutLR;

use crate::layout_ratatui::PagerLayoutTB;

use crate::layout_ratatui::PagerLayoutBase;

use super::RatatuiVariables;

use crate::libmain::AppStateReceiverData;

use ratatui::DefaultTerminal;

use super::ScreenPainter;

use crate::clipboards::cbentry::CBEntry;

use std::cell::RefMut;

// use super::Second;

use crate::tools::flatline;

use crate::tools::tabfix;

use std::cmp::Ordering;

use enum_iterator::all;

use crate::clipboards::CBType;


// use super::Main;

use crate::clipboards::AppendedCBEntry;

use super::ActiveArea;

use crate::clipboards::AcbeId;

use std::collections::VecDeque;

use regex::Regex;

use crate::layout::Layout;

use crate::scroller::Scroller;

use crate::config::Config;

pub(crate) struct ScreenFirstPage {
 pub(crate) config: &'static Config,
 // scroller_main: WrapScroller,
 pub(crate) scroller_main: Scroller,
 // scroller_second: Scroller,
 pub(crate) layout: Layout,
 pub(crate) flipstate: u8,
 pub(crate) wrapped: bool,
 pub(crate) paused: bool,
 pub(crate) regex_edit_mode: Option<String>,
 pub(crate) regex_edit_mode_state: String,
 pub(crate) regex_edit_mode_last_working: Option<Regex>,
 pub(crate) regex: Vec<Regex>,
 pub(crate) regex_filtered_cbs_entries: VecDeque<FilteredCbsEntries>,
 pub(crate) delete_confirm_mode: Option<AcbeId>,
 pub(crate) active_area: ActiveArea,
 pub(crate) main_width: usize,
 pub(crate) second_width: usize,
 pub(crate) prev_selected_text: Option<Vec<u8>>,
 pub(crate) needs_refilter: bool,
 pub(crate) last_entry_count: usize,
}

pub(crate) enum FilteredCbsEntries {
 ACE(AppendedCBEntry),
 Line,
 Empty,
}

// TODO : mode in the vicinity of first_page() definition (maybe inside)
impl ScreenFirstPage {
 pub(crate) fn new(config: &'static Config) -> Self {
  Self {
   config,
   // scroller_main: WrapScroller::default(),
   scroller_main: Scroller::default(),
   // scroller_second: Scroller::new(),
   layout: Layout::new(),
   flipstate: 1,
   wrapped: false,
   paused: false,
   regex_edit_mode: None,
   regex_edit_mode_state: "".to_string(),
   regex_edit_mode_last_working: None,
   regex: vec![],
   regex_filtered_cbs_entries: VecDeque::new(),
   delete_confirm_mode: None,
   active_area: ActiveArea::Main,
   main_width: 80,
   second_width: 80,
   prev_selected_text: None,
   needs_refilter: true,
   last_entry_count: 0,
  }
 }

 pub(crate) fn flipstate_next(&mut self) {
  self.flipstate = (self.flipstate + 1) % 3;
 }
 pub(crate) fn flipstate_prev(&mut self) {
  self.flipstate = (self.flipstate + 2) % 3;
 }

 pub(crate) fn update_filtered_entries(&mut self, cbs: &mut crate::clipboards::Clipboards) {
  if !self.needs_refilter {
   return;
  }
  trace!("update_filtered_entries");
  let entries = cbs.get_cbentries();

  // gtewxxi8oh
  self.regex_filtered_cbs_entries = entries
   .values()
   .rev()
   .filter_map(|line| {
    let mut res = true;
    let mut r = self.regex.clone();
    r.extend(self.regex_edit_mode_last_working.iter().cloned());
    for r in r {
     if !r.is_match(&line.cbentry.borrow().as_string()) {
      res = false;
      break;
     }
    }
    match res {
     true => Some(FilteredCbsEntries::ACE(line.clone())),
     false => None,
    }
   })
   .collect::<VecDeque<_>>();

  {
   let cbtype_enum_vector: Vec<CBType> = all::<CBType>().collect::<Vec<_>>();
   let mut last_entries = cbtype_enum_vector
    .iter()
    .map(|x| cbs.get_last_entries().get(x))
    .collect::<Vec<_>>();

   last_entries.sort_by(|a, b| match (a, b) {
    (None, None) => Ordering::Equal,
    (None, Some(_)) => Ordering::Less,
    (Some(_), None) => Ordering::Greater,
    (Some(c), Some(d)) => c
     .cbentry
     .borrow()
     .get_timestamp()
     .cmp(&d.cbentry.borrow().get_timestamp()),
   });

   self
    .regex_filtered_cbs_entries
    .push_front(FilteredCbsEntries::Line);

   last_entries
    .iter()
    .map(|x| match x {
     Some(v) => FilteredCbsEntries::ACE((*v).clone()),
     None => FilteredCbsEntries::Empty,
    })
    .for_each(|x| self.regex_filtered_cbs_entries.push_front(x));
  }

  self.last_entry_count = cbs.get_cbentries().len();
  self.needs_refilter = false;
 }

 pub(crate) fn get_max_hoffset_main(&self, _cbs: &crate::clipboards::Clipboards) -> usize {
  if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
   let entries = &self.regex_filtered_cbs_entries;
   if cursor < entries.len() {
    if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
     return tabfix(&flatline(&acbe.cbentry.borrow().as_string())).width();
    }
   }
  }
  0
 }

 pub(crate) fn get_max_hoffset_second(&self, _cbs: &crate::clipboards::Clipboards) -> usize {
  if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
   let entries = &self.regex_filtered_cbs_entries;
   if cursor < entries.len() {
    if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
     let cbentry_borrowed = acbe.cbentry.borrow();
     let scroller_second = cbentry_borrowed.get_scroller();
     if let Some(cursor_second) = scroller_second.get_cursor_in_content_array() {
      let lines = cbentry_borrowed.get_text();
      // return lines.iter().map(|l| tabfix(l).width()).max().unwrap_or(0);
      if cursor_second < lines.len() {
       return tabfix(&lines[cursor_second]).width();
      }
     }
    }
   }
  }
  0
 }

 pub(crate) fn toggle_active_area(&mut self) {
  self.active_area = match self.active_area {
   ActiveArea::Main => ActiveArea::Second,
   ActiveArea::Second => ActiveArea::Main,
  };
 }

 pub(crate) fn get_current_entry(&self) -> Option<RefMut<'_, CBEntry>> {
  if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
   let entries = &self.regex_filtered_cbs_entries;
   if cursor < entries.len() {
    if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
     let cbentry_borrowed = acbe.cbentry.borrow_mut();
     return Some(cbentry_borrowed);
    }
   }
  }
  None
 }

 // fn get_active_scroller<'a>(
 //  &'a mut self,
 //  cbe: Option<&'a mut CBEntry>,
 // ) -> Option<&'a mut Scroller> {
 //  match self.active_area {
 //   ActiveArea::Main => Some(&mut self.scroller_main),
 //   ActiveArea::Second => cbe.map(|x| x.get_scroller_mut()),
 //  }
 // }
}

impl ScreenPainter for ScreenFirstPage {
 /// the paint method opens a TwoScreenDefaultWidget which is later painted
 /// by the terminal.draw method
 fn paint(&mut self, terminal: &mut DefaultTerminal, assd: &mut AppStateReceiverData) {
  let cbs = &mut assd.cbs;

  if cbs.get_cbentries().len() != self.last_entry_count {
   self.needs_refilter = true;
  }
  trace!("needs_refilter {}", self.needs_refilter);
  // self.needs_refilter = false;
  self.update_filtered_entries(cbs);
  // return;

  let layout = &mut self.layout;

  let rv = if self.flipstate == 0 {
   &RatatuiVariables::new::<PagerLayoutBase>(terminal)
  } else if self.flipstate == 1 {
   &RatatuiVariables::new::<PagerLayoutTB>(terminal)
  } else {
   &RatatuiVariables::new::<PagerLayoutLR>(terminal)
  };

  self.main_width = rv.pl.get_main_area().width as usize;
  self.second_width = rv
   .pl
   .get_second_main_area()
   .map_or(self.main_width, |r| r.width as usize);

  {
   let inner_main_rect = rv.pl.get_main_area().inner(Margin::new(1, 1));
   layout.set_width_height_from_rect(inner_main_rect);
   let inner_second_rect = rv
    .pl
    .get_second_main_area()
    .map(|x| x.inner(Margin::new(1, 1)));

   // cawxd8rc8j 0%
   {
    let entries = &self.regex_filtered_cbs_entries;

    // let mut selected_string = Vec::<u8>::new();
    // let mut selected_lines = &Vec::<String>::new();
    let mut selected_cbentry: Option<Rc<RefCell<CBEntry>>> = None;
    let mut line_count2 = None;

    if self.config.debug {
     trace!("scroller.set_content_length(entries.len()) : {}", entries.len());
    }
    self.scroller_main.set_hwindowlength(self.main_width);

    // etzwepgkfl
    // self.scroller_second.set_hwindowlength(self.second_width);

    self.scroller_main.set_content_length(entries.len());

    // scroller.set_windowlength(height + 1 - layout.get_current_line());

    self
     .scroller_main
     .set_windowlength(inner_main_rect.height as usize);

    let second_area_height = inner_second_rect.map_or(0, |x| x.height as usize);

    // etzwepgkfl
    // self.scroller_second.set_windowlength(second_area_height);

    let numbers_width = (entries.len() as f64).log10().ceil() as usize;

    if self.config.debug {
     trace!("scroller.get_safe_windowrange() : {:?}", self.scroller_main.get_safe_windowrange());
    }

    // iwcqjc9i11 Example for the line selection
    // cawxd8rc8j 40%

    let mut lines = vec![];

    for (idx, entry) in entries
     .range(self.scroller_main.get_safe_windowrange())
     .enumerate()
    {
     if let FilteredCbsEntries::ACE(appended_cbentry) = entry {
      let mut bm = appended_cbentry.cbentry.borrow_mut();
      let scroller_mut = bm.get_scroller_mut();
      // etzwepgkfl
      scroller_mut.set_hwindowlength(self.second_width);
      // etzwepgkfl
      scroller_mut.set_windowlength(second_area_height);
     }

     // if &FilteredCbsEntries::ACE( entry) = entry {
     //  let bm = entry.borrow_mut();
     //  bm.
     // }

     let is_cursor = match self.scroller_main.get_cursor_in_window() {
      None => false,
      Some(value) => idx == value,
     };

     let cursor_star = if is_cursor { ">" } else { " " };

     match entry {
      FilteredCbsEntries::ACE(acbe) => {
       let cbentry = &acbe.cbentry;
       // let is_selected = entry.is_selected(cbs);
       let is_selected = cbs.is_fixated(cbentry);

       let selection_star = if is_selected { "*" } else { " " };

       let cbentry_borrowed = cbentry.borrow_mut();

       if is_cursor {
        if self.prev_selected_text.as_ref() != Some(cbentry_borrowed.get_data()) {
         // etzwepgkfl

         // self.scroller_second.reset_hoffset();
         // cbentry_borrowed.get_scroller_mut().reset_hoffset();
         self.prev_selected_text = Some(cbentry_borrowed.get_data().clone());
        }
        // selected_string = cbentry.data.clone();
        // selected_lines = cbentry_borrowed.get_text();
        selected_cbentry = Some(Rc::clone(cbentry));
        let _ = line_count2.insert(cbentry_borrowed.get_text().len());
       }

       {
        // let s002 = format!(
        //  "{} {} {:width$} {} {} : {}",
        //  cursor_star,
        //  selection_star,
        //  idx + self.scroller_main.get_windowposition(), // mqbojcmkot
        //  cbentry_borrowed.get_cbtype().get_info(),
        //  cbentry_borrowed.get_date_time(),
        //  // cbentry_borrowed.as_string(),
        //  "",
        // width = numbers_width,
        // );
        // // lines.push(layout.fixline(&s002));
        // // lines.push(flatline(&s002));
        // lines.push((flatline(&s002), flatline(&cbentry_borrowed.as_string().into_owned())));

        lines.push(LineStrings {
         wrapped: false,
         cursor: cursor_star.to_string(),
         line_number: format!(
          " {} {:width$} {} {} : ",
          selection_star,
          idx + self.scroller_main.get_windowposition(), // mqbojcmkot
          cbentry_borrowed.get_cbtype().get_info(),
          cbentry_borrowed.get_date_time(),
          width = numbers_width,
         ),
         text: LineStringsType::S(flatline(&cbentry_borrowed.as_string())),
        });
       }
      }
      FilteredCbsEntries::Line => {
       //  lines.push((layout.centerline("----- ↑ active ↑ ----- ↓ incoming ↓ -----"), "".to_string()));
       lines.push(LineStrings {
        wrapped: false,
        cursor: cursor_star.to_string(),
        line_number: layout
         .centerline("----- ↑ active ↑ ----- ↓ incoming ↓ -----")
         .to_string(),
        text: LineStringsType::S("".to_string()),
       });
      }
      FilteredCbsEntries::Empty => {
       //  lines.push(("".into(), "".into()));
       lines.push(LineStrings {
        wrapped: false,
        cursor: cursor_star.to_string(),
        line_number: "".to_string(),
        text: LineStringsType::S("".to_string()),
       });
      }
     }
    }

    // let all_lines = lines.join("\n");
    let all_lines = lines;

    // etzwepgkfl
    let mut hoffset_second: usize = 0;
    // cawxd8rc8j 40%
    let all_lines2 = {
     let string_lines = match &selected_cbentry {
      Some(rc) => rc.borrow().get_text().clone(),
      None => vec![],
     };
     // etzwepgkfl
     // self.scroller_second.set_content_length(string_lines.len());

     let mut bm;
     let scroller_second = if let Some(selected_cbentry) = selected_cbentry.as_ref() {
      bm = selected_cbentry.borrow_mut();
      let sm = bm.get_scroller_mut();
      sm.set_content_length(string_lines.len());
      hoffset_second = sm.get_hoffset();
      sm
     } else {
      &mut Scroller::default()
     };

     render_scroller_lines4(
      // &mut self.scroller_second,
      scroller_second,
      &string_lines,
      self.wrapped,
      layout,
      |cursor_star, idx, numbers_width, entry| {
       //  (format!("{} {:width$} : ", cursor_star, idx, width = numbers_width,), entry.to_string())
       LineStrings {
        wrapped: self.wrapped,
        cursor: cursor_star.to_string(),
        line_number: format!(" {:width$} : ", idx, width = numbers_width,),
        text: LineStringsType::S(entry.to_string()),
       }
      },
     )
    };
    // cawxd8rc8j 50%

    // wrap simulation gqhdbjurhn :
    // let all_lines = all_lines
    //  .iter()
    //  .flat_map(|x| vec![(*x).clone(), (*x).clone()])
    //  .collect::<Vec<LineStrings>>();

    let theme_colors = self.config.color_theme.get_or_default();

    let all_lines = LineStringsConfig {
     line_strings: &all_lines,
     wrapped: false,
     title: "entry list",
     line_count: Some(entries.len()),
     hoffset: self.scroller_main.get_hoffset(),
     theme_colors: theme_colors.clone(),
     cursor_color: if self.active_area == ActiveArea::Second {
      theme_colors.cursor_inactive
     } else {
      None
     },
    };

    {
     let window_wraps = all_lines
      .prepare2print(*rv.pl.get_main_area())
      .iter()
      .map(|x| x.len())
      .collect::<Vec<_>>();

     self.scroller_main.set_wrapped_window_length(&window_wraps);
    }

    let all_lines2 = LineStringsConfig {
     line_strings: &all_lines2,
     wrapped: self.wrapped,
     title: "selected content",
     line_count: line_count2,
     // etzwepgkfl
     // hoffset: self.scroller_second.get_hoffset(),
     hoffset: hoffset_second,
     theme_colors: theme_colors.clone(),
     cursor_color: if self.active_area == ActiveArea::Main {
      theme_colors.cursor_inactive
     } else {
      None
     },
    };

    if let Some(second_main_area) = rv.pl.get_second_main_area() {
     let window_wraps = all_lines2
      .prepare2print(*second_main_area)
      .iter()
      .map(|x| x.len())
      .collect::<Vec<_>>();

     let mut bm;
     let scroller_second = if let Some(selected_cbentry) = selected_cbentry.as_ref() {
      bm = selected_cbentry.borrow_mut();
      let sm = bm.get_scroller_mut();
      Some(sm)
     } else {
      None
     };
     // let mut bm = selected_cbentry.map( |x| x.borrow_mut());
     // self.scroller_main.set_wrapped_window_length(&window_wraps);
     if let Some(x) = scroller_second {
      x.set_wrapped_window_length(&window_wraps)
     }
    }

    let sw = TwoScreenDefaultWidget {
     helpline: HELP_FIRST_PAGE,
     rv,
     // tsfp: &self,
     all_lines,
     all_lines2,
     regex_edit_mode: self.regex_edit_mode.clone(),
     regex_edit_mode_state: self.regex_edit_mode_state.clone(),
     regex_count: self.regex.len() + self.regex_edit_mode.is_some() as usize,
     delete_confirm_mode: self.delete_confirm_mode,
     statusline_heap: assd.statusline_heap.clone(),
     paused: self.config.paused.is_paused(),
     active_area: self.active_area,
     theme_colors: theme_colors.clone(),
    };

    terminal
     .draw(|frame| frame.render_widget(sw, frame.area()))
     .unwrap();
   }
  }
 }

 fn handle_event(&mut self, evt: &MyEvent, assd: &mut AppStateReceiverData) -> NextTsp {
  let cbs = &mut assd.cbs;

  if evt == &MyEvent::CbInserted {
   self.needs_refilter = true;
   return NextTsp::NoNextTsp;
  }

  if let Some(mut regex_edit_mode) = self.regex_edit_mode.clone() {
   let regex = Regex::new(&regex_edit_mode);
   match regex {
    Ok(regex) => {
     self.regex_edit_mode_state = "".to_string();
     self.regex_edit_mode_last_working = Some(regex);
    }
    Err(_) => self.regex_edit_mode_state = "  < buggy regex".to_string(),
   }

   match evt {
    MyEvent::Termion(Event::Key(Key::Esc)) => {
     self.regex_edit_mode = None;
     self.regex_edit_mode_last_working = None;
    }
    MyEvent::Termion(Event::Key(Key::Char('\n'))) => {
     if let Ok(regex) = Regex::new(&regex_edit_mode) {
      self.regex_edit_mode = None;
      self.regex_edit_mode_last_working = None;
      self.regex.push(regex);
     }
    }
    MyEvent::Termion(Event::Key(Key::Backspace)) => {
     regex_edit_mode.pop();
     let _ = self.regex_edit_mode.insert(regex_edit_mode);
    }
    MyEvent::Termion(Event::Key(Key::Char(char))) => {
     regex_edit_mode.push(*char);
     let _ = self.regex_edit_mode.insert(regex_edit_mode);
    }
    _ => {}
   }
   self.needs_refilter = true;
   return NextTsp::IgnoreBasicEvents;
  } else if self.delete_confirm_mode.is_some() {
   match evt {
    MyEvent::Termion(Event::Key(Key::Esc)) => {
     self.delete_confirm_mode = None;
    }
    MyEvent::Termion(Event::Key(Key::Char('y'))) => {
     if let Some(id) = self.delete_confirm_mode {
      cbs.remove_by_seq(id);
      self.delete_confirm_mode = None;
      self.needs_refilter = true;
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('n'))) => {
     self.delete_confirm_mode = None;
    }
    _ => {}
   }
   return NextTsp::IgnoreBasicEvents;
  } else {
   match evt {
    MyEvent::Termion(Event::Key(Key::Char('r'))) => {
     self.regex.pop();
     self.needs_refilter = true;
    }
    //  MyEvent::SignalHook(SIGWINCH) => terminal_reinitialize = true,
    MyEvent::Termion(Event::Key(Key::Char('h'))) => {
     return NextTsp::Stack(Rc::new(RefCell::new(ScreenViewPage::new(
      self.config,
      "help".to_string(),
      CBEntry::new(config::USAGE.to_string().as_bytes()),
     ))));
    }
    MyEvent::Termion(Event::Key(Key::Char('m'))) => {
     return NextTsp::Stack(Rc::new(RefCell::new(ScreenMenu::new(self.config))));
    }
    MyEvent::Termion(Event::Key(Key::Char('f'))) => {
     self.flipstate_next();
    }
    MyEvent::Termion(Event::Key(Key::Char('F'))) => {
     self.flipstate_prev();
    }
    MyEvent::Termion(Event::Key(Key::Char('s'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
      let entries = &self.regex_filtered_cbs_entries;
      if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
       cbs.toggle_fixation(&(*acbe).clone());
      }
      self.needs_refilter = true;
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('t'))) => {
     cbs.toggle_clipboards();
    }
    MyEvent::Termion(Event::Key(Key::Char('v'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
      let entries = &self.regex_filtered_cbs_entries;
      if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
       return NextTsp::Stack(Rc::new(RefCell::new(ScreenViewPage::new(
        self.config,
        "view entry".to_string(),
        acbe.cbentry.borrow().clone(),
       ))));
      };
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('e'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
      let entries = &self.regex_filtered_cbs_entries;
      // let entry = &entries[cursor];

      if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
       match ScreenEditorPage::new(
        self.config,
        acbe.cbentry.borrow().as_string().into_owned(),
        acbe.id,
       ) {
        Ok(page) => return NextTsp::Stack(Rc::new(RefCell::new(page))),
        Err(e) => {
         eprintln!("Failed to create editor page: {}", e);
         assd
          .statusline_heap
          .push(StatusSeverity::Warning, format!("Failed to create editor page: {}", e));
         return NextTsp::NoNextTsp;
        }
       }
      }
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('w'))) => {
     self.wrapped = !self.wrapped;
    }
    MyEvent::Termion(Event::Key(Key::Char('\t'))) => {
     self.toggle_active_area();
    }
    MyEvent::Termion(Event::Key(Key::Char('d'))) => {
     if let Some(cursor) = self.scroller_main.get_cursor_in_content_array() {
      let entries = &self.regex_filtered_cbs_entries;
      if entries.is_empty() {
      } else if let FilteredCbsEntries::ACE(acbe) = &entries[cursor] {
       match assd.cbs.get_cbentries().get(&acbe.id) {
        Some(_) => self.delete_confirm_mode = Some(acbe.id),
        None => assd
         .statusline_heap
         .push(StatusSeverity::InfoShort, "not deletable".to_string()),
       }
      }
     }
    }
    MyEvent::Termion(Event::Key(Key::Char('p'))) => {
     assd.sender.send(MyEvent::TogglePause).unwrap();
    }
    MyEvent::Termion(Event::Key(Key::Char('/'))) => {
     self.regex_edit_mode = Some("".to_string());
     self.needs_refilter = true;
    }
    _ => {
     // TODO : optimize
     let max_offset = match self.active_area {
      ActiveArea::Main => self.get_max_hoffset_main(cbs),
      ActiveArea::Second => self.get_max_hoffset_second(cbs),
     };

     match self.active_area {
      ActiveArea::Main => {
       self.scroller_main.set_max_hoffset(max_offset);
       Pager::handle_event(&mut self.scroller_main, evt);
      }

      ActiveArea::Second => {
       let mut current_entry = self.get_current_entry();
       let current_entry = current_entry.as_deref_mut().map(|x| x.get_scroller_mut());
       current_entry.map(|x| {
        x.set_max_hoffset(max_offset);
        Pager::handle_event(x, evt);
       });
      }
     };
    }
   }
  }
  NextTsp::NoNextTsp
 }
}
