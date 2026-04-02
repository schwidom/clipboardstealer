
Usage: clipboardstealer [OPTIONS]

Options:

      --append-ndjson-bin <APPEND_NDJSON_BIN>
          appends clipboard information to file
      --load-ndjson-bin <LOAD_NDJSON_BIN>
          reads clipboard information from file
      --load-and-append-ndjson-bin <LOAD_AND_APPEND_NDJSON_BIN>
          loads clipboard information from file and appends to it
      --append-ndjson <APPEND_NDJSON>
          appends clipboard information to file (JSON String format)
      --load-ndjson <LOAD_NDJSON>
          reads clipboard information from file (JSON String format)
      --load-and-append-ndjson <LOAD_AND_APPEND_NDJSON>
          loads clipboard information from file and appends to it (JSON String format)
      --editor
          interprets the EDITOR environment variable always as editor
      --debug
          provides debug information
      --debugfile <DEBUGFILE>
          writes debug information into file
  -h, --help
          Print help
  -V, --version
          Print version

Overview:

 - is a clipboard manager
 - runs in a terminal window, 
 - captures the X11 clipboards named: primary, secondary and clipboard
 - works also with xwayland (tested on debian 13)
 - allows selection of all three of them
 - enforces the user choice (on shortcut s)
 - allows editing of entries

 Installation:

 - apt-get install libxcb1-dev # needed
 - cargo install clipboardstealer

 - this crate is not intended to be used as a library

 Keys: 

  orientation: Up, Down, PgUp, PgDown, Home, End
  orientation: Left, Right, Shift Left, Shift Right

  / (push), r (pop) ... stacked regex search

  (h)elp   ... this screen 
  (v)iew   ... shows the selected entry
  (e)dit   ... edit the selected entry
  (d)elete ... deletes the selected entry
  (t)oggle ... toggles the contents of the clipboards 'primary' and 'clipboards'

  (s)elect ... selects the chosen entry and 
               enforces it for the specific 
               primary, secondary or clipboard clipboards
  (fF)lip  ... the layout
  (w)rap  ... wraps the lines
  (p)ause ... pauses the clipboard scanning, continues with p

  Esc    ... discard status messages
  Esc    ... stop regex editing

  Tab    ... switch windows

  (q)uit ... exits a screen
  e(x)it ... exits the program
  Ctrl-C ... exits the program
  

 Copyright : Frank Schwidom 2025 - 2026
 This software is licensed under the terms of the Apache-2.0 license.

