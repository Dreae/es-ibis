#![feature(let_chains)]

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::os::unix::prelude::OsStrExt;

use nom::{IResult, Err as NomErr};
use nom::error::{Error, ErrorKind};
use nom::number::complete::{le_u8, le_u32, le_f64, le_i32, le_i64, le_i16, le_u16};
use nom::combinator::{all_consuming, map};
use nom::multi::{length_count, fold_many0};
use nom::bytes::complete::{tag, take};
use nom::sequence::{preceded, tuple};
use nom::branch::alt;

mod value;
mod opcodes;

use opcodes::EVEOpCode;
use value::EVEValue;

use crate::value::HashableEVEValue;

pub fn decode_payload<'a>(payload: &'a [u8]) -> IResult<&'a [u8], Vec<EVEValue>> {
    let (payload, len) = le_u32(payload)?;
    log::trace!("Len {}", len);
    assert!(payload.len() == len as usize);

    let (payload, _tilde) = tag([0x7e])(payload)?;
    let (payload, _save_count) = le_u32(payload)?;
    log::trace!("Got save_count {}", _save_count);
    all_consuming(
        fold_many0(
            crate::decode_value,
            Vec::new,
            |mut acc, item| {
                acc.push(item);
                acc
            }
        )
    )(payload)
}

fn decode_value<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    let (payload, opcode) = le_u8(payload)?;
    log::trace!("Got opcode {:#04x}", opcode);
    match opcode {
        _ if opcode == EVEOpCode::None.into() => Ok((payload, EVEValue::None)),
        _ if opcode == EVEOpCode::Long.into() => map(le_i32, |v| v.into())(payload),
        _ if opcode == EVEOpCode::LongLong.into() => map(le_i64, |v| v.into())(payload),
        _ if opcode == EVEOpCode::SignedShort.into() => map(le_i16, |v| v.into())(payload),
        _ if opcode == EVEOpCode::Byte.into() => map(le_u8, |v| v.into())(payload),
        _ if opcode == EVEOpCode::IntegerNegativeOne.into() => Ok((payload, EVEValue::Integer(-1))),
        _ if opcode == EVEOpCode::IntegerZero.into() => Ok((payload, EVEValue::Integer(0))),
        _ if opcode == EVEOpCode::IntegerOne.into() => Ok((payload, EVEValue::Integer(1))),
        _ if opcode == EVEOpCode::Real.into() => map(le_f64, |v| v.into())(payload),
        _ if opcode == EVEOpCode::RealZero.into() => Ok((payload, EVEValue::Float(0.0))),
        _ if opcode == EVEOpCode::ShortString.into() => crate::decode_string(payload),
        _ if opcode == EVEOpCode::StringTableString.into() => crate::decode_stringtable_string(payload),
        _ if opcode == EVEOpCode::WStringUCS2.into() => crate::decode_wstring_ucs2(payload),
        _ if opcode == EVEOpCode::LongString.into() => crate::decode_string(payload),
        _ if opcode == EVEOpCode::Tuple.into() => crate::decode_tuple(payload),
        _ if opcode == EVEOpCode::Dict.into() => crate::decode_dict(payload),
        _ if opcode == EVEOpCode::TwoTuple.into() => crate::decode_two_tuple(payload),
        _ if opcode == EVEOpCode::WStringUTF8.into() => crate::decode_wstring_utf8(payload),
        x => crate::invalid_opcode(x, payload)
    }
}

fn invalid_opcode<'a>(opcode: u8, payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    log::error!("Invalid opcode {:#04x} in net message", opcode);
    panic!()
}

fn decode_tuple<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    let (payload, size) = crate::decode_size(payload)?;
    log::trace!("Decoding tuple with {} items", size);

    let mut payload_ptr = payload;
    let mut list = Vec::new();
    for _ in 0..size {
        let (payload, item) = crate::decode_value(payload_ptr)?;
        list.push(item);
        payload_ptr = payload;
    }
    Ok((payload_ptr, EVEValue::Tuple(list)))
}

fn decode_two_tuple<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    log::trace!("Decoding two tuple");
    let (payload, (item1, item2)) = tuple((crate::decode_value, crate::decode_value))(payload)?;
    Ok((payload, EVEValue::Tuple(vec![item1, item2])))
}

fn decode_string<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
let (payload, size) = crate::decode_size(payload)?;
    log::trace!("Decoding {} length string", size);

    let (payload, value) = take(size)(payload)?;
    let string = OsStr::from_bytes(value);
    log::trace!("Decoded string {:?}", string);
    Ok((payload, EVEValue::String(string)))
}

fn decode_wstring_ucs2<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    let (payload, data) = length_count(crate::decode_size, le_u16)(payload)?;
    log::trace!("Decoding {} length wstring", data.len());

    let mut buffer = vec![0u8; data.len() * 2];
    if let Ok(_) = ucs2::decode(&data, &mut buffer) && let Ok(string) = String::from_utf8(buffer) {
        log::trace!("Decoded string {}", string);
        Ok((payload, EVEValue::OwnedString(string)))
    } else {
        log::warn!("Error decoding wstring in net message");
        unimplemented!()
    }
}

fn decode_wstring_utf8<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    let (payload, data) = length_count(crate::decode_size, le_u8)(payload)?;
    log::trace!("Decoding {} length wstring", data.len());

    if let Ok(string) = String::from_utf8(data) {
        log::trace!("Decoded string {}", string);
        Ok((payload, EVEValue::OwnedString(string)))
    } else {
        log::warn!("Error decoding wstring in net message");
        unimplemented!()
    }
}

fn decode_stringtable_string<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    let (payload, index) = le_u8(payload)?;
    unimplemented!()
}

fn decode_dict<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    let (payload, size) = crate::decode_size(payload)?;
    log::trace!("Decoding dict with {} items", size);

    let mut map: BTreeMap<HashableEVEValue, EVEValue> = BTreeMap::new();
    let mut payload_ptr = payload;
    for _ in 0..size {
        let (payload, value) = decode_value(payload_ptr)?;
        let (payload, key) = decode_value(payload)?;
        log::trace!("Decoded dict item {:?}: {:?}", key, value);

        if let Ok(key) = key.try_into() {
            map.insert(key, value);
        } else {
            return Err(NomErr::Failure(Error::new(payload, ErrorKind::Fail)));
        }
        payload_ptr = payload;
    }
    Ok((payload_ptr, EVEValue::Dict(map)))
}

fn decode_size<'a>(payload: &'a [u8]) -> IResult<&'a [u8], usize> {
    let (payload, size ) = alt((
        map(preceded(tag([0xff]), le_u32),
            |size| size as usize),
        map(le_u8, |size| size as usize)
    ))(payload)?;

    Ok((payload, size as usize))
}

#[cfg(test)]
mod tests {
    mod test_data;
    use super::*;

    fn decode_and_print(payload: &'static [u8]) -> IResult<&'_ [u8], Vec<EVEValue>> {
        let res = decode_payload(payload);
        log::trace!("{:?}", res);
        res
    }

    #[test_log::test]
    fn test_parse_packet1() {
        assert!(decode_and_print(test_data::PACKET1).is_ok());
    }

    #[test_log::test]
    fn test_parse_packet2() {
        assert!(decode_and_print(test_data::PACKET2).is_ok());
    }
}
