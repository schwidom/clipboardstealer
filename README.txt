clipboardstealer [--debug] [--debugfile <DEBUGFILE>] [--append-ndjson <APPEND_NDJSON>] [--load-ndjson <LOAD_NDJSON> | ...]

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

  orientation: Cursor Up, Cursor Down, PgUp, PgDown, Home, End
  orientation: Cursor Left, Cursor Right (not implemented yet)

  / (push), r (pop) ... stacked regex search

  (h)elp   ... this screen 
  (v)iew   ... shows the selected entry
  (e)dit   ... edit the selected entry

  (s)elect ... selects the chosen entry and 
               enforces it for the specific 
               primary, secondary or clipboard clipboards
  (fF)lip  ... the layout
  (w)rap  ... wraps the lines

  Esc    ... discard status messages
  Esc    ... stop regex editing

  (q)uit ... exits a screen
  e(x)it ... exits the program
  Ctrl-C ... exits the program
  

 Copyright : Frank Schwidom 2025 - 2026
 This software is licensed under the terms of the Apache-2.0 license.

