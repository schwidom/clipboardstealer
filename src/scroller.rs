use std::{
 cmp::{max, min},
 ops::{Deref, DerefMut, Range},
};

#[derive(Debug, Clone)]
pub(crate) struct Scroller {
 /// can be freely defined
 windowlength: usize, // NOTE: u16 would be enough but lesser casting operations

 windowposition: usize,
 /// can change when the contentlength changes or when the windowlength changes
 cursor: Option<usize>, // NOTE: u16 would be enough but lesser casting operations
 /// can be freely defined
 contentlength: usize,

 hoffset: usize,
 hwindowlength: usize,
 max_hoffset: usize,
}

// constraints:
// windowlength >= 0
// cursor < windowlength (or None)

// gtewxxi8oh
// TODO : in tools
impl Default for Scroller {
 fn default() -> Self {
  Self::new()
 }
}

impl Scroller {
 pub(crate) fn new() -> Self {
  Self {
   windowlength: 0,
   windowposition: 0,
   cursor: None,
   contentlength: 0,
   hoffset: 0,
   hwindowlength: 80, // WARN: ???
   max_hoffset: 0,
  }
 }

 /// returns the start position of the window in the content array
 fn get_safe_windowstart(&self) -> usize {
  min(self.contentlength, self.windowposition)
 }

 /// returns the start position of the window in the content array
 pub(crate) fn get_windowposition(&self) -> usize {
  self.windowposition
 }

 /// returns the end position of the window in the content array
 fn get_safe_windowend(&self) -> usize {
  min(self.contentlength, self.windowposition + self.windowlength)
 }

 /// returns the range of the window in the content array
 pub(crate) fn get_safe_windowrange(&self) -> Range<usize> {
  return self.get_safe_windowstart()..self.get_safe_windowend();
 }

 // TODO : umbenennen in get_cursor_in_content
 pub(crate) fn get_cursor_in_content_array(&self) -> Option<usize> {
  match self.cursor {
   None => None,
   Some(cursor) => {
    let res = self.windowposition + cursor;
    // assert!(res < self.contentlength); // crashes when regexes reduce the list gtewxxi8oh
    Some(min(res, self.contentlength))
   }
  }
 }

 pub(crate) fn cursor_increase(&mut self) -> bool {
  match (self.get_cursor_in_content_array(), Some(self.contentlength)) {
   // TODO : optimize
   (None, None) => {}
   (None, Some(cl)) => {
    if 0 == cl {
     return false;
    }
   }
   (Some(_), None) => {}
   (Some(cia), Some(cl)) => {
    if 0 == cl {
     self.cursor = None;
     return false;
    }
    // assert!(cia <= cl); // can crash when regexes reduce the list gtewxxi8oh
    if cia + 1 == cl {
     self.cursor = None;
     return false;
    }
   }
  }

  match self.cursor {
   None => {
    if self.windowlength > 0 {
     self.cursor = Some(0);
     self.windowposition = 0;
    }
    false
   }
   Some(value) => {
    let newcursor = value + 1;
    if newcursor < self.windowlength {
     self.cursor = Some(newcursor);
     return true;
    } else if self.windowposition < usize::MAX {
     self.windowposition += 1;
     return true;
    }
    false
   }
  }
 }

 pub(crate) fn cursor_home(&mut self) {
  if self.windowlength == 0 {
   self.cursor = None;
  } else if self.contentlength == 0 {
   self.cursor = None;
  } else {
   self.cursor = Some(0);
  }
  self.windowposition = 0;
 }

 pub(crate) fn cursor_end(&mut self) {
  if self.windowlength == 0 {
   self.cursor = None;
  } else if self.contentlength == 0 {
   self.cursor = None;
  } else {
   let cl = self.contentlength;
   {
    // izm8emilxi
    {
     if self.windowlength <= cl {
      self.cursor = Some(self.windowlength - 1);
      self.windowposition = cl - self.windowlength;
     } else {
      self.windowposition = 0;
      assert!((cl - 1) <= u16::MAX as usize);
      self.cursor = Some(cl - 1);
     }
    }
   }
  }
 }

 pub(crate) fn cursor_decrease(&mut self) -> bool {
  match (self.get_cursor_in_content_array(), Some(self.contentlength)) {
   // TODO : optimize
   (None, None) => {}
   (None, Some(cl)) => {
    if 0 == cl {
     return false;
    }
   }
   (Some(_), None) => {}
   (Some(cia), Some(cl)) => {
    if 0 == cl {
     self.cursor = None;
     return false;
    }
    // assert!(cia <= cl); // can crash when regexes reduce the list gtewxxi8oh
    if cia == 0 {
     self.cursor = None;
     return false;
    }
   }
  }

  match self.cursor {
   None => {
    if self.windowlength > 0 {
     // self.cursor = Some(self.windowlength - 1);
     // self.
     let cl = self.contentlength;
     {
      // izm8emilxi
      {
       if self.windowlength <= cl {
        self.cursor = Some(self.windowlength - 1);
        self.windowposition = cl - self.windowlength;
       } else {
        self.windowposition = 0;
        assert!((cl - 1) <= u16::MAX as usize);
        self.cursor = Some(cl - 1);
       }
      }
     }
    }
    false
   }
   Some(value) => {
    if value == 0 {
     if self.windowposition > 0 {
      self.windowposition -= 1;
      return true;
     }
     false
    } else {
     self.cursor = Some(value - 1);
     true
    }
   }
  }
 }

 pub(crate) fn cursor_decrease_by(&mut self, cr: CursorRepetitions) {
  let amount = match cr {
   CursorRepetitions::WindowLength => self.windowlength,
   CursorRepetitions::Count(value) => value,
  };

  for _ in 0..amount {
   if !self.cursor_decrease() {
    break;
   }
  }
 }

 pub(crate) fn cursor_increase_by(&mut self, cr: CursorRepetitions) {
  let amount = match cr {
   CursorRepetitions::WindowLength => self.windowlength,
   CursorRepetitions::Count(value) => value,
  };

  for _ in 0..amount {
   if !self.cursor_increase() {
    break;
   }
  }
 }

 fn cursorfix(&mut self) {
  if let Some(cursor) = self.cursor {
   // let limit = min(self.windowlength, self.contentlength - self.windowposition);
   let limit = min(self.windowlength, self.contentlength.saturating_sub(self.windowposition));
   if cursor >= limit {
    if limit > 0 {
     self.cursor = Some(limit - 1);
    } else {
     self.cursor = None;
    }
   }
  }
 }

 pub(crate) fn set_windowlength(&mut self, len: usize) {
  if len == 0 {
   self.cursor = None;
  }
  if self.windowlength == len {
   return;
  }
  self.windowlength = len;
  self.cursorfix();
 }

 // TODO : calculations gtewxxi8oh
 pub(crate) fn set_content_length(&mut self, cl: usize) {
  self.contentlength = cl;
  self.cursorfix();
 }

 #[allow(unused)] // used in tests
 pub(crate) fn get_content_length(&self) -> usize {
  self.contentlength
 }

 #[allow(unused)] // used in tests
 pub(crate) fn get_windowlength(&self) -> usize {
  self.windowlength
 }
 pub(crate) fn get_cursor_in_window(&self) -> Option<usize> {
  self.cursor
 }

 pub(crate) fn set_hwindowlength(&mut self, len: usize) {
  self.hwindowlength = len;
 }

 #[allow(unused)] // maybe for tests
 pub(crate) fn get_hwindowlength(&self) -> usize {
  self.hwindowlength
 }

 pub(crate) fn get_hoffset(&self) -> usize {
  self.hoffset
 }

 pub(crate) fn scroll_left(&mut self) {
  let step = max(1, self.hwindowlength / 2);
  self.hoffset = self.hoffset.saturating_sub(step);
 }

 pub(crate) fn scroll_right(&mut self) {
  let step = max(1, self.hwindowlength / 2);
  self.hoffset = (self.hoffset + step).min(self.max_hoffset);
 }

 pub(crate) fn reset_hoffset(&mut self) {
  self.hoffset = 0;
 }

 pub(crate) fn scroll_right_to_end(&mut self) {
  self.hoffset = self.max_hoffset;
 }

 pub(crate) fn set_max_hoffset(&mut self, max_hoffset: usize) {
  self.max_hoffset = max_hoffset;
 }

 /// DOKU
 fn wrapped_window_length(windowlength: usize, window_wraps: &[usize]) -> usize {
  let rs = window_wraps.iter().enumerate().scan(0, |state, (e, x)| {
   *state += x;
   let ret = *state;
   if ret <= windowlength {
    Some(1 + e)
   } else {
    None
   }
  });

  rs.last().unwrap_or(1)
 }

 pub(crate) fn set_wrapped_window_length(&mut self, window_wraps: &[usize]) {
  let nwl = Self::wrapped_window_length(self.get_windowlength(), window_wraps);
  self.set_windowlength(nwl);
 }
}
pub(crate) enum CursorRepetitions {
 WindowLength,
 Count(usize),
}

#[cfg(test)]
mod tests {
 use std::ops::{Deref, DerefMut};

 use crate::scroller::{CursorRepetitions, Scroller, WrapScroller};

 #[test]
 fn scroller_new_001() {
  assert!(true);
  let mut s = Scroller::new();

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_safe_windowend(), 0);
  assert_eq!(s.get_windowlength(), 0);
  assert_eq!(s.get_safe_windowrange(), 0..0);
  assert_eq!(s.get_safe_windowstart(), 0);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 0);
  assert_eq!(s.get_safe_windowrange(), 0..0);

  s.cursor_decrease();

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 0);
  assert_eq!(s.get_safe_windowrange(), 0..0);
 }

 #[test]
 fn scroller_windowlength_001() {
  assert!(true);
  let mut s = Scroller::new();
  s.set_windowlength(1);

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..0);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..0);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..0);
 }

 #[test]
 fn scroller_windowlength_002() {
  assert!(true);
  let mut s = Scroller::new();
  s.set_windowlength(1);
  s.set_content_length(1);

  assert_eq!(s.get_content_length(), 1);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..1);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), 1);
  assert_eq!(s.get_cursor_in_content_array(), Some(0));
  assert_eq!(s.get_cursor_in_window(), Some(0));
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..1);

  s.cursor_increase();

  assert!(None < Some(1));
  assert!(None < Some(0));
  assert!(None < Some(-1));
  // assert!( Option::<i32>::None == None);

  assert_eq!(s.get_content_length(), 1);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..1);
 }

 #[test]
 fn scroller_windowlength_003() {
  assert!(true);
  let mut s = Scroller::new();
  s.set_windowlength(1);
  s.set_content_length(0);

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..0);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..0);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..0);
 }

 #[test]
 fn scroller_windowlength_004() {
  assert!(true);
  let mut s = Scroller::new();
  s.set_windowlength(1);
  s.set_content_length(0);

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..0);

  s.cursor_decrease();

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..0);

  s.cursor_decrease();

  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_safe_windowrange(), 0..0);
 }

 #[test]
 fn scroller_windowlength_005() {
  // gtewxxi8oh
  assert!(true);
  let mut s = Scroller::new();
  s.set_windowlength(10);
  s.set_content_length(10);
  s.cursor_increase();
  s.cursor_increase();
  s.cursor_increase();
  s.set_content_length(1);
  s.cursor_increase();
  s.cursor_increase();
  s.get_safe_windowrange();
  s.set_content_length(1);
  s.cursor_decrease();
  s.cursor_decrease();
  s.cursor_decrease();
 }

 #[test]

 fn scroller_crash_001() {
  // see scroller_new_001

  let mut s = Scroller::new();
  assert_eq!(s.get_content_length(), 0);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_safe_windowend(), 0);
  assert_eq!(s.get_windowlength(), 0);
  assert_eq!(s.get_safe_windowrange(), 0..0);
  assert_eq!(s.get_safe_windowstart(), 0);

  s.set_windowlength(10);
  s.set_content_length(15);
  s.cursor_decrease();
  s.set_content_length(0);
 }

 #[test]
 fn scroller_crash_002() {
  // see scroller_new_001

  fn test(wl: usize, cl: usize, creases: i64, cl2: usize) {
   let mut s = Scroller::new();
   s.set_windowlength(wl);
   s.set_content_length(cl);
   if creases < 0 {
    for _ in 0..-creases {
     s.cursor_decrease();
    }
   }
   if creases > 0 {
    for _ in 0..creases {
     s.cursor_increase();
    }
   }

   println!("{:?}", (wl, cl, creases, cl2));
   println!("{:?}", s.get_safe_windowrange());
   assert!(s.get_safe_windowstart() <= s.get_safe_windowend());
   assert!(s.get_safe_windowstart() <= cl);
   assert!(s.get_safe_windowend() <= cl);
   s.set_content_length(cl2);
   println!("{:?}", s.get_safe_windowrange());
   assert!(s.get_safe_windowstart() <= s.get_safe_windowend());
   assert!(s.get_safe_windowstart() <= cl2);
   assert!(s.get_safe_windowend() <= cl2);
  }

  for wl in 0..=2 {
   for cl in 0..=2 {
    for creases in -5..=5 {
     for cl2 in 0..=2 {
      test(wl, cl, creases, cl2);
     }
    }
   }
  }
  // panic!();
 }

 // maybe later
 // #[test]
 // fn scroller_safety_001() {
 //  // see scroller_new_001

 //  let mut s = Scroller::new();
 //  assert_eq!(s.get_content_length(), 0);
 //  assert_eq!(s.get_cursor_in_array(), None);
 //  assert_eq!(s.get_cursor(), None);
 //  assert_eq!(s.get_safe_windowend(), 0);
 //  assert_eq!(s.get_windowlength(), 0);
 //  assert_eq!(s.get_safe_windowrange(), 0..0);
 //  assert_eq!(s.get_windowstart(), 0);

 //  s.set_windowlength(1);
 // }

 #[test]
 fn test_option_comparison_001() {
  assert!(Some(-1) < Some(1));
  assert!(None < Some(1));
  assert!(None < Some(0));
  assert!(None < Some(-1));
  assert!(Option::<i32>::None == None);
 }

 #[test]
 fn test_subscroller_001() {
  // basics
  let mut s = Scroller::new();

  s.set_content_length(4);
  s.set_windowlength(2);
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_windowposition(), 0);

  s.cursor_increase_by(CursorRepetitions::Count(1)); // cf1mjwfa8w
  assert_eq!(s.get_cursor_in_window(), Some(0));
  assert_eq!(s.get_cursor_in_content_array(), Some(0));
  assert_eq!(s.get_windowposition(), 0);

  s.cursor_increase_by(CursorRepetitions::Count(1));
  assert_eq!(s.get_cursor_in_window(), Some(1));
  assert_eq!(s.get_cursor_in_content_array(), Some(1));
  assert_eq!(s.get_windowposition(), 0);

  s.cursor_increase_by(CursorRepetitions::Count(1));
  assert_eq!(s.get_cursor_in_window(), Some(1));
  assert_eq!(s.get_cursor_in_content_array(), Some(2));
  assert_eq!(s.get_windowposition(), 1);

  s.cursor_increase_by(CursorRepetitions::Count(1));
  assert_eq!(s.get_cursor_in_window(), Some(1));
  assert_eq!(s.get_cursor_in_content_array(), Some(3));
  assert_eq!(s.get_windowposition(), 2);

  s.cursor_increase_by(CursorRepetitions::Count(1));
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_windowposition(), 2);

  s.cursor_increase_by(CursorRepetitions::Count(1)); // cf1mjwfa8w
  assert_eq!(s.get_cursor_in_window(), Some(0));
  assert_eq!(s.get_cursor_in_content_array(), Some(0));
  assert_eq!(s.get_windowposition(), 0);
 }

 #[test]
 fn test_subscroller_002() {
  // basics
  // let mut s = Scroller::new();
  let mut ws = WrapScroller::default();

  ws.set_content_length(4);
  ws.set_windowlength(2);
  assert_eq!(ws.get_cursor_in_window(), None);
  assert_eq!(ws.get_cursor_in_content_array(), None);
  assert_eq!(ws.get_windowposition(), 0);

 }

 #[test]
 fn test_subscroller_003() {
  // basics
  // let mut s = Scroller::new();
  let mut s = Scroller::default();

  s.set_content_length(5);
  let windowlength = 3;
  s.set_windowlength(windowlength); // Zeichenbereich
  assert_eq!(s.get_cursor_in_window(), None);
  assert_eq!(s.get_cursor_in_content_array(), None);
  assert_eq!(s.get_windowposition(), 0);

  let content_wraps = [1, 2, 2, 1, 2];

  // panic!("ox: {:?}", new_window_length()); // 3

  let mut progress = || {
   s.set_windowlength(windowlength);
   let nwl =
    Scroller::wrapped_window_length(windowlength, &content_wraps[s.get_safe_windowrange()]);
   s.set_windowlength(nwl);
   s.cursor_increase();
   (nwl, s.get_safe_windowstart(), s.get_cursor_in_content_array(), s.get_cursor_in_window())
  };

  assert_eq!((2, 0, Some(0), Some(0)), progress());

  // Simulation des Ablaufs:
  assert_eq!((2, 0, Some(1), Some(1)), progress());
  assert_eq!((2, 1, Some(2), Some(1)), progress());
  assert_eq!((1, 2, Some(2), Some(0)), progress());
  assert_eq!((2, 2, Some(3), Some(1)), progress());
  assert_eq!((2, 3, Some(4), Some(1)), progress());
  assert_eq!((2, 3, None, None), progress());
  assert_eq!((2, 0, Some(0), Some(0)), progress());
 }

 #[test]
 fn test_subscroller_004() {
  assert_eq!(Scroller::wrapped_window_length(3, &[]), 1); // ?
  assert_eq!(Scroller::wrapped_window_length(3, &[1, 1, 1]), 3);
  assert_eq!(Scroller::wrapped_window_length(3, &[2, 2, 2]), 1);
  assert_eq!(Scroller::wrapped_window_length(3, &[3, 3, 3]), 1);
  assert_eq!(Scroller::wrapped_window_length(3, &[3, 3, 3, 3]), 1);
  assert_eq!(Scroller::wrapped_window_length(3, &[3, 3, 3, 3, 3]), 1);
  assert_eq!(Scroller::wrapped_window_length(3, &[3, 3]), 1);
  assert_eq!(Scroller::wrapped_window_length(3, &[3]), 1);
  assert_eq!(Scroller::wrapped_window_length(3, &[4, 4, 4]), 1);

  assert_eq!(Scroller::wrapped_window_length(5, &[3, 3, 3, 3, 3]), 1);
  assert_eq!(Scroller::wrapped_window_length(6, &[3, 3, 3, 3, 3]), 2);
  assert_eq!(Scroller::wrapped_window_length(7, &[3, 3, 3, 3, 3]), 2);
  assert_eq!(Scroller::wrapped_window_length(8, &[3, 3, 3, 3, 3]), 2);
  assert_eq!(Scroller::wrapped_window_length(9, &[3, 3, 3, 3, 3]), 3);
  assert_eq!(Scroller::wrapped_window_length(9, &[1, 3, 5, 3, 3]), 3);
  assert_eq!(Scroller::wrapped_window_length(9, &[5, 3, 1, 3, 3]), 3);
 }
}

#[derive(Default)]
pub(crate) struct WrapScroller {
 pub(crate) s: Scroller,
 /// subscroller
 pub(crate) sub: Scroller,
}

impl Deref for WrapScroller {
 type Target = Scroller;

 fn deref(&self) -> &Self::Target {
  &self.s
 }
}

impl DerefMut for WrapScroller {
 fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
  &mut self.s
 }
}
