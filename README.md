
![Animation](https://private-user-images.githubusercontent.com/1835921/575546613-4a3378b0-793c-4e35-ba2c-f08de0c3539f.gif?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzU2ODUzMzEsIm5iZiI6MTc3NTY4NTAzMSwicGF0aCI6Ii8xODM1OTIxLzU3NTU0NjYxMy00YTMzNzhiMC03OTNjLTRlMzUtYmEyYy1mMDhkZTBjMzUzOWYuZ2lmP1gtQW16LUFsZ29yaXRobT1BV1M0LUhNQUMtU0hBMjU2JlgtQW16LUNyZWRlbnRpYWw9QUtJQVZDT0RZTFNBNTNQUUs0WkElMkYyMDI2MDQwOCUyRnVzLWVhc3QtMSUyRnMzJTJGYXdzNF9yZXF1ZXN0JlgtQW16LURhdGU9MjAyNjA0MDhUMjE1MDMxWiZYLUFtei1FeHBpcmVzPTMwMCZYLUFtei1TaWduYXR1cmU9NWRhMzk5ZWNjOWI2YmQ3Mzc1NGI3MTIyNTRlYmQyMjBjMjEyNjE4OWMwMDU5MjFlNGRiNTQ4ZjFlNWJiYTM1MCZYLUFtei1TaWduZWRIZWFkZXJzPWhvc3QifQ.FXmZ-1cgXEc2wK-hTVgzz124jnrrEHh7qYYIGArCvCc)

# Usage: clipboardstealer [OPTIONS]

## Options:

- `--append-ndjson-bin <APPEND_NDJSON_BIN>`
  - appends clipboard information to file
- `--load-ndjson-bin <LOAD_NDJSON_BIN>`
  - reads clipboard information from file
- `--load-and-append-ndjson-bin <LOAD_AND_APPEND_NDJSON_BIN>`
  - loads clipboard information from file and appends to it
- `--append-ndjson <APPEND_NDJSON>`
  - appends clipboard information to file (JSON String format)
- `--load-ndjson <LOAD_NDJSON>`
  - reads clipboard information from file (JSON String format)
- `--load-and-append-ndjson <LOAD_AND_APPEND_NDJSON>`
  - loads clipboard information from file and appends to it (JSON String format)
- `--editor`
  - interprets the `EDITOR` environment variable always as editor
- `--convert-bin-ndjson <CONVERT_BIN_NDJSON>`
  -   converts bin ndjson to string ndjson (input file)
- `--to-string-ndjson <TO_STRING_NDJSON>`
  -   output file for converted bin ndjson
- `--convert-string-ndjson <CONVERT_STRING_NDJSON>`
  -   converts string ndjson to bin ndjson (input file)
- `--to-bin-ndjson <TO_BIN_NDJSON>`
  -   output file for converted string ndjson

- `--debug`
  - provides debug information
- `--debugfile <DEBUGFILE>`
  - writes debug information into file
- `-h, --help`
  - Print help
- `-V, --version`
  - Print version

## Overview:

- is a clipboard manager
- runs in a terminal window
- captures the X11 clipboards named: `primary`, `secondary`, and `clipboard`
- works also with `xwayland` (tested on Debian 13)
- allows selection of all three of them
- enforces the user choice (on shortcut `s`)
- allows editing of entries

## Installation:

- `apt-get install libxcb1-dev`  _(needed)_
- `cargo install clipboardstealer`

*This crate is not intended to be used as a library.*

## Keys:

- **orientation**: Up, Down, PgUp, PgDown, Home, End
- **orientation**: Left, Right, Shift Left, Shift Right

- `/` (push), `r` (pop) ... stacked regex search

- **Help and Navigation:**

```plaintext
(h)elp   ... this screen
(v)iew   ... shows the selected entry
(e)dit   ... edit the selected entry
(d)elete ... deletes the selected entry
(t)oggle ... toggles the contents of the clipboards 'primary' and 'clipboards'
```

- **Selection and Layout:**

```plaintext
(s)elect ... selects the chosen entry and enforces it for the specific primary, secondary, or clipboard clipboards
(fF)lip  ... the layout
(w)rap  ... wraps the lines
(p)ause ... pauses the clipboard scanning, continues with p
```

- **Other Controls:**

```plaintext
Esc    ... discard status messages
Esc    ... stop regex editing
Tab    ... switch windows
(q)uit ... exits a screen
e(x)it ... exits the program
Ctrl-C ... exits the program
```

## Copyright

Frank Schwidom 2025 - 2026  
This software is licensed under the terms of the Apache-2.0 license.
