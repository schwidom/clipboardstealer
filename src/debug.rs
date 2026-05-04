use std::{thread, time::Duration};
use tracing::trace;

pub(crate) fn monitor() {
 trace!("1111111111111");
 thread::sleep(Duration::from_millis(100));
}

pub(crate) fn monitor2(info: &str) {
 trace!("1111111111111 {}", info);
 thread::sleep(Duration::from_millis(100));
}
