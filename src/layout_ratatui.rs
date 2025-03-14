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

pub trait FrameNew {
 fn new(frame: &Frame) -> Self;
}
pub struct PagerLayout {
 pub title_area: Rect,
 pub main_area: Rect,
 pub status_area: Rect,
}

impl FrameNew for PagerLayout {
 fn new(frame: &Frame) -> Self {
  use Constraint::{Fill, Length, Min};

  let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
  let [title_area, main_area, status_area] = vertical.areas(frame.area());

  // let horizontal = Layout::horizontal([Fill(1); 2]);
  // let [left_area, right_area] = horizontal.areas(main_area);

  // frame.render_widget(Block::bordered().title("Title Bar"), title_area);
  // frame.render_widget(Block::bordered().title("Status Bar"), status_area);
  // frame.render_widget(Block::bordered().title("Left"), left_area);
  // frame.render_widget(Block::bordered().title("Right"), right_area);

  Self {
   title_area,
   main_area,
   status_area,
  }
 }
}
