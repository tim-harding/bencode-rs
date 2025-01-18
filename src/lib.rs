mod parsing;

use parsing::{next_list_item, next_value, Res, Value as ParseValue};
use std::{error::Error, fmt::Display};

pub enum Value {
    Integer(i64),
    ByteString(Vec<u8>),
    List(Vec<Value>),
    Dictionary(Vec<(Value, Value)>),
}

pub fn parse_all(bytes: &[u8]) -> Result<Vec<Value>, ParseError> {
    let (_, out) = parse_list(bytes, next_value)?;
    Ok(out)
}

fn parse_list(
    mut bytes: &[u8],
    f: fn(&[u8]) -> Res<Option<ParseValue>>,
) -> Result<(&[u8], Vec<Value>), ParseError> {
    let mut out = vec![];
    loop {
        match parse_one(bytes, f)? {
            (_, None) => break,
            (i, Some(value)) => {
                bytes = i;
                out.push(value);
            }
        }
    }
    Ok((bytes, out))
}

#[allow(clippy::type_complexity)]
fn parse_dictionary(mut bytes: &[u8]) -> Result<(&[u8], Vec<(Value, Value)>), ParseError> {
    let mut out = vec![];
    loop {
        let (i, k) = parse_one(bytes, next_list_item)?;
        let Some(k) = k else { break };
        let (i, v) = parse_one(i, next_list_item)?;
        bytes = i;
        let Some(v) = v else { return Err(ParseError) };
        out.push((k, v));
    }
    Ok((bytes, out))
}

fn parse_one(
    bytes: &[u8],
    f: fn(&[u8]) -> Res<Option<ParseValue>>,
) -> Result<(&[u8], Option<Value>), ParseError> {
    Ok(match f(bytes) {
        Ok((i, None)) => (i, None),
        Ok((i, Some(ParseValue::Integer(int)))) => (i, Some(Value::Integer(int))),
        Ok((i, Some(ParseValue::ByteString(s)))) => (i, Some(Value::ByteString(s.to_vec()))),
        Ok((i, Some(ParseValue::List))) => {
            let (i, list) = parse_list(i, next_list_item)?;
            (i, Some(Value::List(list)))
        }
        Ok((i, Some(ParseValue::Dictionary))) => {
            let (i, dictionary) = parse_dictionary(i)?;
            (i, Some(Value::Dictionary(dictionary)))
        }
        Err(_) => return Err(ParseError),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid bencode")
    }
}

impl Error for ParseError {}
