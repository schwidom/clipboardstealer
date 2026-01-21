use std::{thread, time::Duration};
use tracing::{Level, event, info, span, trace};


pub fn monitor() {
 trace!("1111111111111");
 thread::sleep(Duration::from_millis(100));
}

pub fn monitor2(info: &str) {
 trace!("1111111111111 {}", info);
 thread::sleep(Duration::from_millis(100));
}
