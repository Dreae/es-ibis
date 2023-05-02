use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::os::unix::prelude::OsStrExt;

use nom::{IResult, Err as NomErr};
use nom::error::{Error, ErrorKind};
use nom::number::complete::{le_u8, le_u32, le_f64, le_i32, le_i64, le_i16, le_u16, le_i128};
use nom::combinator::{all_consuming, map};
use nom::multi::{length_count, fold_many0, length_data, length_value};
use nom::bytes::complete::{tag, take};
use nom::sequence::{preceded, tuple};
use nom::branch::alt;

use crate::opcodes::EVEOpCode;
use crate::string_table::DEFAULT_STRINGS;
use crate::value::EVEValue;

use crate::value::HashableEVEValue;

pub fn decode_payload<'a>(payload: &'a [u8]) -> IResult<&'a [u8], Vec<EVEValue>> {
    let (payload, len) = le_u32(payload)?;
    log::trace!("Len {}", len);
    assert!(payload.len() == len as usize);

    self::decode_payload_body(payload)
}

fn decode_payload_body<'a>(payload: &'a [u8]) -> IResult<&'a [u8], Vec<EVEValue>> {
    let (payload, _tilde) = tag([0x7e])(payload)?;
    let (payload, _save_count) = le_u32(payload)?;
    log::trace!("Decoding {} len body", payload.len());
    log::trace!("Got save_count {}", _save_count);
    fold_many0(
        self::decode_value,
        Vec::new,
        |mut acc, item| {
            acc.push(item);
            acc
        }
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
        _ if opcode == EVEOpCode::ShortString.into() => self::decode_string(payload),
        _ if opcode == EVEOpCode::StringTableString.into() => self::decode_stringtable_string(payload),
        _ if opcode == EVEOpCode::WStringUCS2.into() => self::decode_wstring_ucs2(payload),
        _ if opcode == EVEOpCode::LongString.into() => self::decode_string(payload),
        _ if opcode == EVEOpCode::Tuple.into() => self::decode_tuple(payload),
        _ if opcode == EVEOpCode::Dict.into() => self::decode_dict(payload),
        _ if opcode == EVEOpCode::Object.into() => self::decode_object(payload),
        _ if opcode == EVEOpCode::EmptyTuple.into() => Ok((payload, EVEValue::Tuple(vec![]))),
        _ if opcode == EVEOpCode::OneTuple.into() => self::decode_one_tuple(payload),
        _ if opcode == EVEOpCode::SubStream.into() => {
            map(length_value(self::decode_size, self::decode_payload_body), |vals| EVEValue::SubStream(vals))(payload)
        },
        _ if opcode == EVEOpCode::TwoTuple.into() => self::decode_two_tuple(payload),
        _ if opcode == EVEOpCode::WStringUTF8.into() => self::decode_wstring_utf8(payload),
        _ if opcode == EVEOpCode::VarInteger.into() => self::decode_var_int(payload),
        x => self::invalid_opcode(x, payload)
    }
}

fn invalid_opcode<'a>(opcode: u8, payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    log::error!("Invalid opcode {:#04x} in net message", opcode);
    Err(NomErr::Failure(Error::new(payload, ErrorKind::Fail)))
}

fn decode_size<'a>(payload: &'a [u8]) -> IResult<&'a [u8], usize> {
    let (payload, size ) = alt((
        map(preceded(tag([0xff]), le_u32),
            |size| size as usize),
        map(le_u8, |size| size as usize)
    ))(payload)?;

    Ok((payload, size as usize))
}

fn decode_tuple<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    log::trace!("Decoding tuple");
    map(length_count(self::decode_size, self::decode_value), |vals| EVEValue::Tuple(vals))(payload)
}

fn decode_two_tuple<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    log::trace!("Decoding two tuple");
    let (payload, (item1, item2)) = tuple((self::decode_value, self::decode_value))(payload)?;
    Ok((payload, EVEValue::Tuple(vec![item1, item2])))
}

fn decode_one_tuple<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    log::trace!("Decoding one tuple");
    map(self::decode_value, |val| EVEValue::Tuple(vec![val]))(payload)
}

fn decode_string<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
let (payload, size) = self::decode_size(payload)?;
    log::trace!("Decoding {} length string", size);

    let (payload, value) = take(size)(payload)?;
    let string = OsStr::from_bytes(value);
    log::trace!("Decoded string {:?}", string);
    Ok((payload, EVEValue::String(string)))
}

fn decode_wstring_ucs2<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    let (payload, data) = length_count(self::decode_size, le_u16)(payload)?;
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
    let (payload, data) = length_count(self::decode_size, le_u8)(payload)?;
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
    if (index as usize) < DEFAULT_STRINGS.len() {
        Ok((payload, EVEValue::String((*DEFAULT_STRINGS.get(index as usize).unwrap()).as_ref())))
    } else {
        unimplemented!()
    }
}

fn decode_dict<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    log::trace!("Decoding dict");
    let (payload, kvs) = length_count(self::decode_size, tuple((self::decode_value, self::decode_value)))(payload)?;

    let mut map = BTreeMap::new();
    for (value, key) in kvs {
        if let Ok(key) = key.try_into() {
            map.insert(key, value);
        } else {
            return Err(NomErr::Failure(Error::new(payload, ErrorKind::Fail)));
        }
    }
    Ok((payload, EVEValue::Dict(map)))
}

fn decode_object<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    let (payload, typ) = self::decode_value(payload)?;
    let (payload, arguments) = self::decode_value(payload)?;
    Ok((payload, EVEValue::Object(vec![typ, arguments])))
}

fn decode_var_int<'a>(payload: &'a [u8]) -> IResult<&'a [u8], EVEValue> {
    let (payload, buffer) = length_count(self::decode_size, le_u8)(payload)?;
    match buffer.len() {
        1 => {
            // CCP using 2 bytes to send 1...
            map(le_u8, |v| EVEValue::BigInt(v as i128))(payload)
        },
        4 => {
            let bytes: [u8; 4] = buffer.try_into().unwrap();
            Ok((payload, EVEValue::BigInt(i32::from_le_bytes(bytes) as i128)))
        },
        8 => {
            let bytes: [u8; 8] = buffer.try_into().unwrap();
            Ok((payload, EVEValue::BigInt(i64::from_le_bytes(bytes) as i128)))
        },
        16 => {
            let bytes: [u8; 16] = buffer.try_into().unwrap();
            Ok((payload, EVEValue::BigInt(i128::from_le_bytes(bytes) as i128)))
        },
        _ => {
            log::error!("Unexpected VarInt length in packet {} {:?}", buffer.len(), buffer);
            return Err(NomErr::Failure(Error::new(payload, ErrorKind::Fail)));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::test_data;
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

    #[test_log::test]
    fn test_macho_net_get_time() {
        assert!(decode_and_print(test_data::MACHONET_GETTIME).is_ok());
    }
}
