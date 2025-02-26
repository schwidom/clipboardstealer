use std::cmp::min;

use crate::tools::flatline;

pub struct Layout {
 current_line: u16,
 width: Option<u16>,
 height: Option<u16>,
}

impl Layout {
 pub fn new() -> Self {
  Self {
   current_line: 1,
   width: None,
   height: None,
  }
 }

 pub fn reset_current_line(&mut self) {
  self.current_line = 1;
 }

 pub fn set_width_height(&mut self, width: u16, height: u16) {
  self.width = Some(width);
  self.height = Some(height);
 }

 pub fn get_current_line(&self) -> u16 {
  self.current_line
 }
 fn fixline(&self, string: &str) -> String {
  let z = flatline(string); // lcibiwnao0
                            // NOTE : writes over the end because wie are not at the beginning of the line
  match self.width {
   Some(w) => &z[0..min(z.len(), w as usize)],
   None => &z[0..z.len()],
  }
  .to_owned()
 }

 pub fn print_line_wrap(&mut self) {}
 pub fn print_line_cut(&mut self, line: &str) {
  print!("{}", termion::cursor::Goto(1, self.current_line));
  print!("{}", self.fixline(line));
  print!("{}", termion::clear::UntilNewline);
  self.current_line += 1;
 }
}
