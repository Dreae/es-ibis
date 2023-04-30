use std::{ffi::OsStr, cmp::Ordering};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum EVEValue<'a> {
    Tuple(Vec<EVEValue<'a>>),
    Dict(BTreeMap<HashableEVEValue<'a>, EVEValue<'a>>),
    Byte(u8),
    Short(i16),
    Integer(i64),
    Float(f64),
    String(&'a OsStr),
    OwnedString(String),
    None
}

#[derive(Debug)]
pub enum HashableEVEValue<'a> {
    Byte(u8),
    Short(i16),
    Integer(i64),
    Float(f64),
    String(&'a OsStr),
    OwnedString(String),
    None
}

impl <'a> TryInto<HashableEVEValue<'a>> for EVEValue<'a> {
    type Error = ();
    fn try_into(self) -> Result<HashableEVEValue<'a>, Self::Error> {
        use self::EVEValue::*;
        match self {
            None => Ok(HashableEVEValue::None),
            Byte(i) => Ok(i.into()),
            Short(i) => Ok(i.into()),
            Integer(i) => Ok(i.into()),
            Float(i) => Ok(i.into()),
            String(s) => Ok(s.into()),
            OwnedString(s) => Ok(s.into()),
            _ => Err(())
        }
    }
}

impl PartialEq for HashableEVEValue<'_> {
    fn eq(&self, other: &HashableEVEValue) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for HashableEVEValue<'_> {}

impl PartialOrd for HashableEVEValue<'_> {
    fn partial_cmp(&self, other: &HashableEVEValue) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// This implementaiton more or less comes from serde-pickle here:
// https://github.com/birkenfeld/serde-pickle/blob/5932524/src/value.rs#L219
// It's purpose is to replicate Python's ordering for values
// of different types, so they can be used as keys in dicts
impl Ord for HashableEVEValue<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        use self::HashableEVEValue::*;
        match *self {
            None => match *other {
                None => Ordering::Equal,
                _ => Ordering::Less
            },
            Short(i) => match *other {
                None => Ordering::Greater,
                Byte(j) => i.cmp(&(j as i16)),
                Short(j) => i.cmp(&j),
                Integer(j) => (i as i64).cmp(&j),
                _ => Ordering::Less
            },
            Byte(i) => match *other {
                None => Ordering::Greater,
                Byte(j) => i.cmp(&j),
                Short(j) => (i as i16).cmp(&j),
                Integer(j) => (i as i64).cmp(&j),
                _ => Ordering::Less
            },
            Integer(i) => match *other {
                None => Ordering::Greater,
                Byte(j) => i.cmp(&(j as i64)),
                Short(j) => i.cmp(&(j as i64)),
                Integer(j) => i.cmp(&j),
                _ => Ordering::Less
            },
            Float(i) => match *other {
                String(_) => Ordering::Less,
                OwnedString(_) => Ordering::Less,
                Float(j) => {
                    match i.partial_cmp(&j) {
                        Some(o) => o,
                        _ => Ordering::Less
                    }
                },
                _ => Ordering::Greater
            }
            String(s) => match *other {
                String(s2) => s.cmp(s2),
                OwnedString(ref s2) => s.cmp(s2.as_ref()),
                _ => Ordering::Greater
            },
            OwnedString(ref s) => match *other {
                String(s2) => AsRef::<OsStr>::as_ref(s).cmp(s2),
                OwnedString(ref s2) => s.cmp(s2),
                _ => Ordering::Greater
            }
        }
    }
}

impl From<u8> for EVEValue<'_> {
    fn from(other: u8) -> Self {
        Self::Byte(other)
    }
}

impl From<i16> for EVEValue<'_> {
    fn from(other: i16) -> Self {
        Self::Short(other)
    }
}

impl From<i32> for EVEValue<'_> {
    fn from(other: i32) -> Self {
        Self::Integer(other as i64)
    }
}

impl From<i64> for EVEValue<'_> {
    fn from(other: i64) -> Self {
        Self::Integer(other)
    }
}

impl From<f32> for EVEValue<'_> {
    fn from(other: f32) -> Self {
        Self::Float(other as f64)
    }
}

impl From<f64> for EVEValue<'_> {
    fn from(other: f64) -> Self {
        Self::Float(other)
    }
}

impl <'a> From<&'a OsStr> for EVEValue<'a> {
    fn from(other: &'a OsStr) -> Self {
        Self::String(other)
    }
}

impl From<String> for EVEValue<'_> {
    fn from(other: String) -> Self {
        Self::OwnedString(other)
    }
}

impl From<u8> for HashableEVEValue<'_> {
    fn from(other: u8) -> Self {
        Self::Byte(other)
    }
}

impl From<i16> for HashableEVEValue<'_> {
    fn from(other: i16) -> Self {
        Self::Short(other)
    }
}

impl From<i32> for HashableEVEValue<'_> {
    fn from(other: i32) -> Self {
        Self::Integer(other as i64)
    }
}

impl From<i64> for HashableEVEValue<'_> {
    fn from(other: i64) -> Self {
        Self::Integer(other)
    }
}

impl From<f32> for HashableEVEValue<'_> {
    fn from(other: f32) -> Self {
        Self::Float(other as f64)
    }
}

impl From<f64> for HashableEVEValue<'_> {
    fn from(other: f64) -> Self {
        Self::Float(other)
    }
}

impl <'a> From<&'a OsStr> for HashableEVEValue<'a> {
    fn from(other: &'a OsStr) -> Self {
        Self::String(other)
    }
}

impl From<String> for HashableEVEValue<'_> {
    fn from(other: String) -> Self {
        Self::OwnedString(other)
    }
}
