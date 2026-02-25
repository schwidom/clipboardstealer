use std::fmt::Display;

use chrono::DateTime;
use chrono::TimeDelta;

use chrono::Local;
use chrono::Utc;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Serialize;
use x11_clipboard::Atoms;
use x11_clipboard::Clipboard;

#[derive(Clone, PartialEq, Debug, PartialOrd, Eq, Ord)]
pub struct MyTime {
 pub timestamp: DateTime<Local>,
}

impl Serialize for MyTime {
 fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
 where
  S: serde::Serializer,
 {
  serializer.serialize_str(&self.to_string())
 }
}

impl<'de> Deserialize<'de> for MyTime {
 fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
 where
  D: serde::Deserializer<'de>,
 {
  // MyTime::from_str( ... )
  struct MytimeCreator;
  impl<'de2> Visitor<'de2> for MytimeCreator {
   type Value = MyTime;

   fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(formatter, "expected a date in %- format")
   }

   fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Bool(v), &self))
   }

   fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_i64(v as i64)
   }

   fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_i64(v as i64)
   }

   fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_i64(v as i64)
   }

   fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Signed(v), &self))
   }

   // fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
   //  where
   //      E: serde::de::Error,
   //  {
   //      let mut buf = [0u8; 58];
   //      let mut writer = serde::format::Buf::new(&mut buf);
   //      std::fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as i128", v)).unwrap();
   //      Err(serde::de::Error::invalid_type(
   //          serde::de::Unexpected::Other(writer.as_str()),
   //          &self,
   //      ))
   //  }

   fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_u64(v as u64)
   }

   fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_u64(v as u64)
   }

   fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_u64(v as u64)
   }

   fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Unsigned(v), &self))
   }

   // fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
   //  where
   //      E: serde::de::Error,
   //  {
   //      let mut buf = [0u8; 57];
   //      let mut writer = serde::format::Buf::new(&mut buf);
   //      std::fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as u128", v)).unwrap();
   //      Err(serde::de::Error::invalid_type(
   //          serde::de::Unexpected::Other(writer.as_str()),
   //          &self,
   //      ))
   //  }

   fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_f64(v as f64)
   }

   fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Float(v), &self))
   }

   fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_str(v.encode_utf8(&mut [0u8; 4]))
   }

   fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    Ok(MyTime::from_str(v))
   }

   fn visit_borrowed_str<E>(self, v: &'de2 str) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_str(v)
   }

   fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_str(&v)
   }

   fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Bytes(v), &self))
   }

   fn visit_borrowed_bytes<E>(self, v: &'de2 [u8]) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_bytes(v)
   }

   fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    self.visit_bytes(&v)
   }

   fn visit_none<E>(self) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Option, &self))
   }

   fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
   where
    D: serde::Deserializer<'de2>,
   {
    let _ = deserializer;
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Option, &self))
   }

   fn visit_unit<E>(self) -> Result<Self::Value, E>
   where
    E: serde::de::Error,
   {
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Unit, &self))
   }

   fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
   where
    D: serde::Deserializer<'de2>,
   {
    let _ = deserializer;
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::NewtypeStruct, &self))
   }

   fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
   where
    A: serde::de::SeqAccess<'de2>,
   {
    let _ = seq;
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Seq, &self))
   }

   fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
   where
    A: serde::de::MapAccess<'de2>,
   {
    let _ = map;
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Map, &self))
   }

   fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
   where
    A: serde::de::EnumAccess<'de2>,
   {
    let _ = data;
    Err(serde::de::Error::invalid_type(serde::de::Unexpected::Enum, &self))
   }
  }
  let visitor = MytimeCreator;
  deserializer.deserialize_string(visitor)
 }
}

impl Display for MyTime {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
  self.timestamp.fmt(f)
 }
}

// cannot create a const function
pub fn create_local_unix_epoch() -> DateTime<Local> {
 const UNIX_EPOCH_UTC: DateTime<Utc> = DateTime::<Utc>::UNIX_EPOCH;
 let timestamp: DateTime<Local> = DateTime::from(UNIX_EPOCH_UTC);
 timestamp
}

use chrono::format::strftime;

impl MyTime {
 pub fn unix_epoch() -> Self {
  Self {
   timestamp: create_local_unix_epoch(),
  }
 }
 pub fn now() -> Self {
  Self {
   timestamp: Local::now(),
  }
 }

 pub fn from_str(s: &str) -> Self {
  // let tmp = DateTime::parse_from_str(s, "%+");
  Self {
   timestamp: DateTime::parse_from_str(s, "%+")
    .unwrap()
    .with_timezone(&Local),
  }
 }

 pub fn elapsed(&self) -> TimeDelta {
  Local::now() - self.timestamp
 }
}

#[cfg(test)]
mod tests {
 use chrono::format::strftime;
 use chrono::{DateTime, Local};
 use serde::Serialize;

 use super::MyTime;

 #[test]
 fn test_001() {
  let t = MyTime::now();
  let s = t.to_string();
  // panic!( "{s}"); // 2026-02-25 01:08:37.842114298 +01:00
  // let y = Local::from("2026-02-25 01:08:37.842114298 +01:00");
  // parse_from_rfc2822("Wed, 18 Feb 2015 23:16:09 GMT")
  // parse_from_rfc3339("1996-12-19T16:39:57-08:00")
  // parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
  // strftime
  // let y = DateTime::parse_from_str(s, fmt)

  let y = DateTime::parse_from_str("2026-02-25 01:08:37.842114298 +01:00", "%+").unwrap();
  assert_eq!("2026-02-25 01:08:37.842114298 +01:00", y.to_string());
  {
   let mt = MyTime {
    timestamp: y.naive_local().and_local_timezone(Local).unwrap(),
   };
   // let mt = MyTime{ timestamp : y.naive_utc().and_local_timezone(Local).unwrap()}; // differs
   assert_eq!("2026-02-25 01:08:37.842114298 +01:00", mt.to_string());
  }
  {
   let z = y.with_timezone(&Local);
   assert_eq!("2026-02-25 01:08:37.842114298 +01:00", z.to_string());
  }
  assert_eq!(
   "2026-02-25 01:08:37.842114298 +01:00",
   MyTime::from_str("2026-02-25 01:08:37.842114298 +01:00").to_string()
  );
 }

 #[test]
 fn test_002() {
  let timestring = "2026-02-25 01:08:37.842114298 +01:00";
  let mt = MyTime::from_str(timestring);
  let j = serde_json::to_string(&mt).unwrap();
  assert_eq!(j, "\"".to_string() + timestring + "\"");
  let k: MyTime = serde_json::from_str(&j).unwrap();
  assert_eq!(timestring, k.to_string());
 }
}

use lazy_static::lazy_static;

lazy_static! {
 pub static ref CB_ATOMS: Atoms = {
  let cb = Clipboard::new().unwrap();
  cb.setter.atoms.clone()
 };
}

pub fn flatline(string: &str) -> String {
 string.replace("\n", "\\n") // lcibiwnao0
}

pub fn tabfix(string: &str) -> String {
 string.replace("\t", "   ")
}
