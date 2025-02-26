use std::{thread, time::Duration};

pub fn monitor() {
 println!("1111111111111");
 thread::sleep(Duration::from_millis(100));
}

pub fn monitor2(info: &str) {
 println!("1111111111111 {}", info);
 thread::sleep(Duration::from_millis(100));
}
