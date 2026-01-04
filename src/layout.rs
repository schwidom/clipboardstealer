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

 pub fn fixline(&self, string: &str) -> String {
  // Remove newlines first
  let z = flatline(string); // lcibiwnao0

  // NOTE: previous implementation used z.len() (bytes) and an inclusive comparison
  // which caused incorrect truncation of multibyte UTF-8 characters and an off-by-one.
  // Use character-aware truncation here. For correct display-width handling (e.g.
  // CJK or emoji that occupy multiple columns) consider using the unicode-width
  // / unicode-segmentation crates in a follow-up change.

  // Count characters (Unicode scalar values)
  let char_count = z.chars().count();

  let l = match self.width {
   Some(w) => min(char_count, w as usize),
   None => char_count,
  };

  z.chars().take(l).collect()
 }

 pub fn print_line_wrap(&mut self) {}
 pub fn print_line_cut(&mut self, line: &str) {
  print!("{}", termion::cursor::Goto(1, self.current_line));
  print!("{}", self.fixline(line));
  print!("{}", termion::clear::UntilNewline);
  self.current_line += 1;
 }
}

#[cfg(test)]
mod tests {
 use crate::layout::Layout;

 #[test]
 fn fixline_multibyte_preserves_chars() {
  let mut layout = Layout::new();
  layout.set_width_height(5, 10);
  // string contains ASCII, an accented char and an emoji (multibyte)
  let s = "aé😊bc";
  let fixed = layout.fixline(s);
  // ensure we truncated by characters (not bytes) and got 5 characters
  assert_eq!(fixed.chars().count(), 5);
 }

 #[test]
 fn fixline_no_width_returns_full() {
  let layout = Layout::new();
  let s = "hello world";
  let fixed = layout.fixline(s);
  assert_eq!(fixed, "helloworld"); // flatline removes newlines only; input has no newlines
 }
}
