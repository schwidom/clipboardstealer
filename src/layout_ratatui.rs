#![allow(unused)]

use ratatui::{
 self,
 layout::{Constraint, Layout, Margin, Rect},
 // termion::{
 //  event::{Event, Key},
 //  input::TermRead,
 // },
 text::Text,
 widgets::Block,
 Frame,
};

// TODO : rename
pub trait PagerLayout {
 fn new(frame: &Frame) -> Self
 where
  Self: Sized;
 fn get_title_area(&self) -> &Rect;
 fn get_main_area(&self) -> &Rect;
 fn get_second_main_area(&self) -> Option<&Rect>;
 fn get_status_area(&self) -> &Rect;
}
pub struct PagerLayoutBase {
 pub title_area: Rect,
 pub main_area: Rect,
 pub status_area: Rect,
}

impl PagerLayout for PagerLayoutBase {
 fn new(frame: &Frame) -> Self {
  use Constraint::{Fill, Length, Min};

  let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
  let [title_area, main_area, status_area] = vertical.areas(frame.area());

  Self {
   title_area,
   main_area,
   status_area,
  }
 }

 fn get_title_area(&self) -> &Rect {
  &self.title_area
 }

 fn get_main_area(&self) -> &Rect {
  &self.main_area
 }

 fn get_second_main_area(&self) -> Option<&Rect> {
  None
 }

 fn get_status_area(&self) -> &Rect {
  &self.status_area
 }
}

pub struct PagerLayoutTB {
 pub title_area: Rect,
 pub main_area_top: Rect,
 pub main_area_bottom: Rect,
 pub status_area: Rect,
}

impl PagerLayout for PagerLayoutTB {
 fn new(frame: &Frame) -> Self {
  use Constraint::{Fill, Length, Min, Percentage};

  let vertical = Layout::vertical([Length(1), Percentage(50), Percentage(50), Length(1)]);
  let [title_area, main_area_top, main_area_down, status_area] = vertical.areas(frame.area());

  Self {
   title_area,
   main_area_top,
   main_area_bottom: main_area_down,
   status_area,
  }
 }

 fn get_title_area(&self) -> &Rect {
  &self.title_area
 }

 fn get_main_area(&self) -> &Rect {
  &self.main_area_top
 }

 fn get_second_main_area(&self) -> Option<&Rect> {
  Some(&self.main_area_bottom)
 }

 fn get_status_area(&self) -> &Rect {
  &self.status_area
 }
}

pub struct PagerLayoutLR {
 pub title_area: Rect,
 pub main_area_left: Rect,
 pub main_area_right: Rect,
 pub status_area: Rect,
}

impl PagerLayout for PagerLayoutLR {
 fn new(frame: &Frame) -> Self {
  use Constraint::{Fill, Length, Min, Percentage};

  let vertical = Layout::vertical([Length(1), Percentage(100), Length(1)]);
  let [title_area, main_area, status_area] = vertical.areas(frame.area());

  let horizontal = Layout::horizontal([Percentage(50), Percentage(50)]);
  let [main_area_left, main_area_right] = horizontal.areas(main_area);

  Self {
   title_area,
   main_area_left,
   main_area_right,
   status_area,
  }
 }

 fn get_title_area(&self) -> &Rect {
  &self.title_area
 }

 fn get_main_area(&self) -> &Rect {
  &self.main_area_left
 }

 fn get_second_main_area(&self) -> Option<&Rect> {
  Some(&self.main_area_right)
 }

 fn get_status_area(&self) -> &Rect {
  &self.status_area
 }
}
