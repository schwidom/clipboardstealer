use std::env::var_os;
use std::error::Error;
use std::io::stdout;
use std::os::fd::AsFd;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use rustix::termios::{tcflush, QueueSelector};

pub(crate) fn edit_file(pb: &PathBuf) -> Result<(), Box<dyn Error>> {
 let mut editor: Option<PathBuf> = None;

 if let Some(editor_var) = var_os("EDITOR") {
  let executable = PathBuf::from(editor_var).canonicalize();
  if let Ok(executable) = executable {
   if executable.is_file() {
    let _ = editor.insert(executable);
   }
  }
 }

 match editor {
  None => {
   return Err(Box::from("no editor found"));
  }

  Some(editor) => {
   let _status = Command::new(&editor)
    // .args(&args)
    .arg(pb)
    // .env_clear()
    // .env_remove("PS1")
    // .env_remove("LD_PRELOAD")
    // .env_remove("OPTERR")
    // .env_remove("LS_COLORS")
    // .env_remove("HOME") // solves the problem
    // .env_remove("COLORTERM")
    // .env("TERM", "vt100") // solves the problem
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    // .stdout(Stdio::piped())
    // .stdout(Stdio::null()) // verhindert den Fehler, aber auch die Editor Darstellung
    // .stderr(Stdio::inherit())
    .stderr(Stdio::null())
    .output()?
    .status;
  }
 }

 // wait for garbage (don't work, no matter how long)
 std::thread::sleep(Duration::from_millis(100));
 // stdout().write_all("hello world".as_bytes()).unwrap();
 // stdout().flush().unwrap();
 tcflush(stdout().as_fd(), QueueSelector::IOFlush).unwrap();

 // let t = std::thread::spawn(|| for _ in stdin().events() {});

 Ok(())
}
