use nom::{
    branch::alt,
    bytes::streaming::take,
    character::streaming::{char, digit0},
    combinator::{eof, map, opt, value, verify},
    sequence::{delimited, pair},
    IResult, Needed,
};

pub(crate) type Res<'a, O> = IResult<&'a [u8], O>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value<'a> {
    Integer(i64),
    ByteString(&'a [u8]),
    List,
    Dictionary,
}

pub fn next_nested(i: &[u8]) -> Res<Option<Value>> {
    alt((value(None, end), map(val, Some)))(i)
}

pub fn next_outer(i: &[u8]) -> Res<Option<Value>> {
    alt((value(None, eof), map(val, Some)))(i)
}

fn val(i: &[u8]) -> Res<Value> {
    alt((
        map(list_start, |()| Value::List),
        map(dictionary_start, |()| Value::Dictionary),
        map(byte_string, Value::ByteString),
        map(integer, Value::Integer),
    ))(i)
}

fn end(i: &[u8]) -> Res<()> {
    single_char(i, 'e')
}

fn dictionary_start(i: &[u8]) -> Res<()> {
    single_char(i, 'd')
}

fn list_start(i: &[u8]) -> Res<()> {
    single_char(i, 'l')
}

fn single_char(i: &[u8], c: char) -> Res<()> {
    map(char(c), |_| ())(i)
}

fn byte_string(i: &[u8]) -> Res<&[u8]> {
    let (rest, length) = uint(i)?;
    map(pair(char(':'), take(length)), |(_, s)| s)(rest)
}

fn integer(i: &[u8]) -> Res<i64> {
    delimited(char('i'), int_inner, char('e'))(i)
}

fn int_inner(i: &[u8]) -> Res<i64> {
    map(pair(minus, uint), |(minus, uint)| {
        uint as i64 * if minus { -1 } else { 1 }
    })(i)
}

fn minus(i: &[u8]) -> Res<bool> {
    map(opt(char('-')), |minus| minus.is_some())(i)
}

fn uint(i: &[u8]) -> Res<u64> {
    alt((uint_zero, uint_nonzero))(i)
}

fn uint_zero(i: &[u8]) -> Res<u64> {
    value(0, char('0'))(i)
}

fn uint_nonzero(i: &[u8]) -> Res<u64> {
    map(pair(verify(byte, is_nonzero), digit0), |(head, tail)| {
        ascii_to_uint(head, tail)
    })(i)
}

fn is_nonzero(b: &u8) -> bool {
    (b'1'..=b'9').contains(b)
}

fn byte(i: &[u8]) -> Res<u8> {
    match i {
        [head, tail @ ..] => Ok((tail, *head)),
        [] => Err(nom::Err::Incomplete(Needed::new(1))),
    }
}

fn ascii_to_uint(head: u8, tail: &[u8]) -> u64 {
    let mut out = (head - b'0') as u64;
    for c in tail {
        out = out * 10 + (c - b'0') as u64;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ints() {
        assert_eq!(integer(b"i0e"), Ok((&[][..], 0)));
        assert_eq!(integer(b"i115e"), Ok((&[][..], 115)));
        assert_eq!(integer(b"i-12e"), Ok((&[][..], -12)));
    }

    #[test]
    fn valid_byte_string() {
        assert_eq!(byte_string(b"6:foobar"), Ok((&[][..], &b"foobar"[..])));
    }

    #[test]
    fn valid_list() {
        let i = b"l3:fooi42e3:bare";
        let Ok((i, Some(Value::List))) = next_outer(i) else {
            panic!("Expected a list");
        };
        let Ok((i, Some(Value::ByteString(b"foo")))) = next_nested(i) else {
            panic!("Expected foo");
        };
        let Ok((i, Some(Value::Integer(42)))) = next_nested(i) else {
            panic!("Expected 42");
        };
        let Ok((i, Some(Value::ByteString(b"bar")))) = next_nested(i) else {
            panic!("Expected bar");
        };
        let Ok((b"", None)) = next_nested(i) else {
            panic!("Expected end of list");
        };
    }
}
