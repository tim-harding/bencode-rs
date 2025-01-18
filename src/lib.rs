mod parsing;

use parsing::{next_nested, next_outer, Res, Value as ParseValue};
use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Integer(i64),
    ByteString(Vec<u8>),
    List(Vec<Value>),
    Dictionary(Vec<(Value, Value)>),
}

pub fn parse_all(bytes: &[u8]) -> Result<Vec<Value>, ParseError> {
    let (_, out) = parse_many(bytes, next_outer)?;
    Ok(out)
}

fn parse_many(
    mut bytes: &[u8],
    f: fn(&[u8]) -> Res<Option<ParseValue>>,
) -> Result<(&[u8], Vec<Value>), ParseError> {
    let mut out = vec![];
    loop {
        let (i, maybe_value) = parse_one(bytes, f)?;
        bytes = i;
        match maybe_value {
            Some(value) => out.push(value),
            None => break,
        }
    }
    Ok((bytes, out))
}

#[allow(clippy::type_complexity)]
fn parse_dictionary(mut bytes: &[u8]) -> Result<(&[u8], Vec<(Value, Value)>), ParseError> {
    let mut out = vec![];
    loop {
        let (i, k) = parse_one(bytes, next_nested)?;
        bytes = i;
        let Some(k) = k else { break };
        let (i, v) = parse_one(bytes, next_nested)?;
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
            let (i, list) = parse_many(i, next_nested)?;
            (i, Some(Value::List(list)))
        }
        Ok((i, Some(ParseValue::Dictionary))) => {
            let (i, dictionary) = parse_dictionary(i)?;
            (i, Some(Value::Dictionary(dictionary)))
        }
        Err(e) => {
            println!("{e:?}");
            return Err(ParseError);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_int() {
        assert_eq!(
            parse_all(b"i115ei-12e"),
            Ok(vec![Value::Integer(115), Value::Integer(-12)])
        );
    }

    #[test]
    fn valid_list() {
        assert_eq!(
            parse_all(b"li115ei-12ee"),
            Ok(vec![Value::List(vec![
                Value::Integer(115),
                Value::Integer(-12)
            ])])
        )
    }

    #[test]
    fn valid_dictionary() {
        assert_eq!(
            parse_all(b"di1ei2ee"),
            Ok(vec![Value::Dictionary(vec![(
                Value::Integer(1),
                Value::Integer(2),
            )])])
        )
    }

    #[test]
    fn complex_value() {
        // assert_eq!(
        //     parse_all(b"i1eli2el3:foo3:bared3:bazi3el7:listkeye5:valueee"),
        //     Ok(vec![
        //         Value::Integer(1),
        //         Value::List(vec![
        //             Value::Integer(2),
        //             Value::List(vec![
        //                 Value::ByteString(b"foo".to_vec()),
        //                 Value::ByteString(b"bar".to_vec()),
        //             ]),
        //             Value::Dictionary(vec![
        //                 (Value::ByteString(b"baz".to_vec()), Value::Integer(3)),
        //                 (
        //                     Value::List(vec![Value::ByteString(b"listkey".to_vec())]),
        //                     Value::ByteString(b"value".to_vec())
        //                 )
        //             ])
        //         ])
        //     ])
        // )
    }
}
