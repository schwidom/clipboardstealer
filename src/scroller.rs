use std::{cmp::min, ops::Range};

#[derive(Debug)]
pub struct Scroller {
 windowlength: u16,
 windowposition: usize,
 cursor: Option<u16>,
 contentlength: Option<usize>,
}

// constraints:
// windowlength >= 0
// cursor < windowlength (or None)

// TODO : in tools
impl Scroller {

 pub fn new() -> Self {
  Self {
   windowlength: 0,
   windowposition: 0,
   cursor: None,
   contentlength: None,
  }
 }

 fn get_windowstart(&self) -> usize {
  self.windowposition
 }

 pub fn get_windowposition(&self) -> usize {
  self.windowposition
 }

 fn get_windowend(&self) -> usize {
  let res = self.windowposition + self.windowlength as usize;
  match self.contentlength {
   None => res,
   Some(cl) => min(cl, res),
  }
 }
 pub fn get_windowrange(&self) -> Range<usize> {
  return self.get_windowstart()..self.get_windowend();
 }
 pub fn get_cursor_in_array(&self) -> Option<usize> {
  match self.cursor {
   None => None,
   Some(value) => {
    let res = self.windowposition + value as usize;
    if let Some(cl) = self.contentlength {
     assert!(res < cl);
    }
    Some(res)
   }
  }
 }

 pub fn cursor_increase(&mut self) -> bool {
  match (self.get_cursor_in_array(), self.contentlength) {
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
    assert!(cia <= cl);
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
    return false;
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
    return false;
   }
  }
 }

 pub fn cursor_home(&mut self) {
  if self.windowlength == 0 {
   self.cursor = None;
  } else if self.contentlength == Some(0) {
   self.cursor = None;
  } else {
   self.cursor = Some(0);
  }
  self.windowposition = 0;
 }

 pub fn cursor_end(&mut self) {
  if self.windowlength == 0 {
   self.cursor = None;
  } else if self.contentlength == Some(0) {
   self.cursor = None;
  } else {
   match self.contentlength {
    // izm8emilxi
    None => {}
    Some(cl) => {
     if self.windowlength as usize <= cl {
      self.cursor = Some(self.windowlength - 1);
      self.windowposition = cl - self.windowlength as usize;
     } else {
      self.windowposition = 0;
      assert!((cl - 1) <= u16::MAX as usize);
      self.cursor = Some((cl - 1) as u16);
     }
    }
   }
  }
 }

 pub fn cursor_decrease(&mut self) -> bool {
  match (self.get_cursor_in_array(), self.contentlength) {
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
    assert!(cia <= cl);
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
     match self.contentlength {
      // izm8emilxi
      None => {}
      Some(cl) => {
       if self.windowlength as usize <= cl {
        self.cursor = Some(self.windowlength - 1);
        self.windowposition = cl - self.windowlength as usize;
       } else {
        self.windowposition = 0;
        assert!((cl - 1) <= u16::MAX as usize);
        self.cursor = Some((cl - 1) as u16);
       }
      }
     }
    }
    return false;
   }
   Some(value) => {
    if value == 0 {
     if self.windowposition > 0 {
      self.windowposition -= 1;
      return true;
     }
     return false;
    } else {
     self.cursor = Some(value - 1);
     return true;
    }
   }
  }
 }

 pub fn cursor_decrease_by(&mut self, cr: CursorRepetitions) {
  let amount = match cr {
   CursorRepetitions::WindowLength => self.windowlength as usize,
   CursorRepetitions::Count(value) => value,
  };

  for _ in 0..amount {
   if !self.cursor_decrease() {
    break;
   }
  }
 }

 pub fn cursor_increase_by(&mut self, cr: CursorRepetitions) {
  let amount = match cr {
   CursorRepetitions::WindowLength => self.windowlength as usize,
   CursorRepetitions::Count(value) => value,
  };

  for _ in 0..amount {
   if !self.cursor_increase() {
    break;
   }
  }
 }

 pub fn set_windowlength(&mut self, len: u16) {
  if len == 0 {
   self.cursor = None;
  }
  if self.windowlength == len {
   return;
  }
  self.windowlength = len;

  match self.cursor {
   None => {}
   Some(value) => {
    if value >= self.windowlength {
     if self.windowlength > 0 {
      self.cursor = Some(self.windowlength - 1);
     } else {
      self.cursor = None;
     }
    }
   }
  }
 }

 // TODO : calculations
 pub fn set_content_length(&mut self, contentlength: Option<usize>) {
  self.contentlength = contentlength;
 }

 pub fn get_content_length(&self) -> Option<usize> {
  self.contentlength
 }

 pub fn get_windowlength(&self) -> u16 {
  self.windowlength
 }
 pub fn get_cursor(&self) -> Option<u16> {
  self.cursor
 }
}
pub enum CursorRepetitions {
 WindowLength,
 Count(usize),
}

#[cfg(test)]
mod tests {
 use crate::scroller::Scroller;

 #[test]
 fn scroller_new_001() {
  assert!(true);
  let mut s = Scroller::new();

  assert_eq!(s.get_content_length(), None);
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowend(), 0);
  assert_eq!(s.get_windowlength(), 0);
  assert_eq!(s.get_windowrange(), 0..0);
  assert_eq!(s.get_windowstart(), 0);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), None);
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 0);
  assert_eq!(s.get_windowrange(), 0..0);

  s.cursor_decrease();

  assert_eq!(s.get_content_length(), None);
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 0);
  assert_eq!(s.get_windowrange(), 0..0);
 }

 #[test]
 fn scroller_windowlength_001() {
  assert!(true);
  let mut s = Scroller::new();
  s.set_windowlength(1);

  assert_eq!(s.get_content_length(), None);
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..1);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), None);
  assert_eq!(s.get_cursor_in_array(), Some(0));
  assert_eq!(s.get_cursor(), Some(0));
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..1);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), None);
  assert_eq!(s.get_cursor_in_array(), Some(1));
  assert_eq!(s.get_cursor(), Some(0));
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 1..2);
 }

 #[test]
 fn scroller_windowlength_002() {
  assert!(true);
  let mut s = Scroller::new();
  s.set_windowlength(1);
  s.set_content_length(Some(1));

  assert_eq!(s.get_content_length(), Some(1));
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..1);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), Some(1));
  assert_eq!(s.get_cursor_in_array(), Some(0));
  assert_eq!(s.get_cursor(), Some(0));
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..1);

  s.cursor_increase();

  assert!(None < Some(1));
  assert!(None < Some(0));
  assert!(None < Some(-1));
  // assert!( Option::<i32>::None == None);

  assert_eq!(s.get_content_length(), Some(1));
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..1);
 }

 #[test]
 fn scroller_windowlength_003() {
  assert!(true);
  let mut s = Scroller::new();
  s.set_windowlength(1);
  s.set_content_length(Some(0));

  assert_eq!(s.get_content_length(), Some(0));
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..0);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), Some(0));
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..0);

  s.cursor_increase();

  assert_eq!(s.get_content_length(), Some(0));
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..0);
 }

 #[test]
 fn scroller_windowlength_004() {
  assert!(true);
  let mut s = Scroller::new();
  s.set_windowlength(1);
  s.set_content_length(Some(0));

  assert_eq!(s.get_content_length(), Some(0));
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..0);

  s.cursor_decrease();

  assert_eq!(s.get_content_length(), Some(0));
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..0);

  s.cursor_decrease();

  assert_eq!(s.get_content_length(), Some(0));
  assert_eq!(s.get_cursor_in_array(), None);
  assert_eq!(s.get_cursor(), None);
  assert_eq!(s.get_windowlength(), 1);
  assert_eq!(s.get_windowrange(), 0..0);
 }

 #[test]
 fn test_option_comparison_001() {
  assert!(None < Some(1));
  assert!(None < Some(0));
  assert!(None < Some(-1));
  assert!(Option::<i32>::None == None);
 }
}
