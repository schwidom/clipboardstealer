use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use crate::clipboards::cbentry::{CBEntry, CBEntryString};

pub(crate) fn convert_bin_to_string(input_path: &str, output_path: &str) -> Result<(), String> {
 let input_file = File::open(input_path)
  .map_err(|e| format!("Failed to open input file '{}': {}", input_path, e))?;
 let mut output_file = OpenOptions::new()
  .create(true)
  .append(true)
  .open(output_path)
  .map_err(|e| format!("Failed to open output file '{}': {}", output_path, e))?;

 let reader = BufReader::new(input_file);
 let mut count = 0;
 for line in reader.lines() {
  let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
  if line.trim().is_empty() {
   continue;
  }
  let cbentry: CBEntry = serde_json::from_str(&line)
   .map_err(|e| format!("Failed to deserialize CBEntry from '{}': {}", line, e))?;
  let string_entry = cbentry.as_json_entry();
  let json = serde_json::to_string(&string_entry)
   .map_err(|e| format!("Failed to serialize CBEntryString: {}", e))?;
  writeln!(output_file, "{}", json).map_err(|e| format!("Failed to write line: {}", e))?;
  count += 1;
 }
 println!("Converted {} entries from '{}' to '{}'", count, input_path, output_path);
 Ok(())
}

pub(crate) fn convert_string_to_bin(input_path: &str, output_path: &str) -> Result<(), String> {
 let input_file = File::open(input_path)
  .map_err(|e| format!("Failed to open input file '{}': {}", input_path, e))?;
 let mut output_file = OpenOptions::new()
  .create(true)
  .append(true)
  .open(output_path)
  .map_err(|e| format!("Failed to open output file '{}': {}", output_path, e))?;

 let reader = BufReader::new(input_file);
 let mut count = 0;
 for line in reader.lines() {
  let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
  if line.trim().is_empty() {
   continue;
  }
  let string_entry: CBEntryString = serde_json::from_str(&line)
   .map_err(|e| format!("Failed to deserialize CBEntryString from '{}': {}", line, e))?;
  let cbentry = CBEntry::from_json_entry(string_entry);
  let json =
   serde_json::to_string(&cbentry).map_err(|e| format!("Failed to serialize CBEntry: {}", e))?;
  writeln!(output_file, "{}", json).map_err(|e| format!("Failed to write line: {}", e))?;
  count += 1;
 }
 println!("Converted {} entries from '{}' to '{}'", count, input_path, output_path);
 Ok(())
}

pub(crate) fn copy_bin(input_path: &str, output_path: &str) -> Result<(), String> {
 let input_file = File::open(input_path)
  .map_err(|e| format!("Failed to open input file '{}': {}", input_path, e))?;
 let mut output_file = OpenOptions::new()
  .create(true)
  .append(true)
  .open(output_path)
  .map_err(|e| format!("Failed to open output file '{}': {}", output_path, e))?;

 let reader = BufReader::new(input_file);
 let mut count = 0;
 for line in reader.lines() {
  let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
  if line.trim().is_empty() {
   continue;
  }
  let _: CBEntry = serde_json::from_str(&line)
   .map_err(|e| format!("Failed to deserialize CBEntry from '{}': {}", line, e))?;
  writeln!(output_file, "{}", line).map_err(|e| format!("Failed to write line: {}", e))?;
  count += 1;
 }
 println!("Copied {} entries from '{}' to '{}'", count, input_path, output_path);
 Ok(())
}

pub(crate) fn copy_string(input_path: &str, output_path: &str) -> Result<(), String> {
 let input_file = File::open(input_path)
  .map_err(|e| format!("Failed to open input file '{}': {}", input_path, e))?;
 let mut output_file = OpenOptions::new()
  .create(true)
  .append(true)
  .open(output_path)
  .map_err(|e| format!("Failed to open output file '{}': {}", output_path, e))?;

 let reader = BufReader::new(input_file);
 let mut count = 0;
 for line in reader.lines() {
  let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
  if line.trim().is_empty() {
   continue;
  }
  let _: CBEntryString = serde_json::from_str(&line)
   .map_err(|e| format!("Failed to deserialize CBEntryString from '{}': {}", line, e))?;
  writeln!(output_file, "{}", line).map_err(|e| format!("Failed to write line: {}", e))?;
  count += 1;
 }
 println!("Copied {} entries from '{}' to '{}'", count, input_path, output_path);
 Ok(())
}
