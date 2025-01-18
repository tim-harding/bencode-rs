use nom::{
    branch::alt,
    bytes::streaming::take,
    character::streaming::{char, digit0, digit1},
    combinator::{map, opt, value, verify},
    sequence::{delimited, pair, tuple},
    IResult, Needed,
};

type Res<'a, O> = IResult<&'a [u8], O>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValuePull<'a> {
    Integer(i64),
    ByteString(&'a [u8]),
    List(ListIter),
    Dictionary(DictionaryIter),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListIter;

impl ListIter {
    pub fn next_value(self, i: &[u8]) -> Res<Option<ValuePull>> {
        maybe_val(i)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DictionaryIter;

impl DictionaryIter {
    pub fn next_pair(self, i: &[u8]) -> Res<Option<(ValuePull, ValuePull)>> {
        alt((value(None, end), map(key_value_pair, Some)))(i)
    }
}

fn key_value_pair(i: &[u8]) -> Res<(ValuePull, ValuePull)> {
    pair(val, val)(i)
}

fn val(i: &[u8]) -> Res<ValuePull> {
    alt((
        map(list_start, |()| ValuePull::List(ListIter)),
        map(dictionary_start, |()| ValuePull::Dictionary(DictionaryIter)),
        map(byte_string, ValuePull::ByteString),
        map(int, ValuePull::Integer),
    ))(i)
}

fn maybe_val(i: &[u8]) -> Res<Option<ValuePull>> {
    alt((value(None, end), map(val, Some)))(i)
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
    let (rest, length) = length(i)?;
    map(pair(char(':'), take(length)), |(_, s)| s)(rest)
}

fn length(i: &[u8]) -> Res<u64> {
    map(digit1, |digits: &[u8]| match digits {
        [head, tail @ ..] => ascii_to_uint(*head, tail),
        [] => unreachable!(),
    })(i)
}

fn int(i: &[u8]) -> Res<i64> {
    delimited(char('i'), alt((zero, nonzero)), char('e'))(i)
}

fn nonzero(i: &[u8]) -> Res<i64> {
    map(nonzero_raw, |(minus, head, tail)| {
        ascii_to_uint(head, tail) as i64 * if minus { -1 } else { 1 }
    })(i)
}

fn nonzero_raw(i: &[u8]) -> Res<(bool, u8, &[u8])> {
    tuple((minus, verify(byte, is_nonzero), digit0))(i)
}

fn minus(i: &[u8]) -> Res<bool> {
    map(opt(char('-')), |minus| minus.is_some())(i)
}

fn zero(i: &[u8]) -> Res<i64> {
    value(0, char('0'))(i)
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
        assert_eq!(int(b"i0e"), Ok((&[][..], 0)));
        assert_eq!(int(b"i115e"), Ok((&[][..], 115)));
        assert_eq!(int(b"i-12e"), Ok((&[][..], -12)));
    }

    #[test]
    fn valid_byte_string() {
        assert_eq!(byte_string(b"6:foobar"), Ok((&[][..], &b"foobar"[..])));
    }

    #[test]
    fn valid_list() {
        let i = b"l3:fooi42e3:bare";
        let Ok((i, Some(ValuePull::List(it)))) = maybe_val(i) else {
            panic!("Expected a list");
        };
        let Ok((i, Some(ValuePull::ByteString(b"foo")))) = it.next_value(i) else {
            panic!("Expected foo");
        };
        let Ok((i, Some(ValuePull::Integer(42)))) = it.next_value(i) else {
            panic!("Expected 42");
        };
        let Ok((i, Some(ValuePull::ByteString(b"bar")))) = it.next_value(i) else {
            panic!("Expected bar");
        };
        let Ok((b"", None)) = it.next_value(i) else {
            panic!("Expected end of list");
        };
    }

    #[test]
    fn valid_dictionary() {
        let i = b"d3:fooi42e3:bari69ee";
        let Ok((i, Some(ValuePull::Dictionary(it)))) = maybe_val(i) else {
            panic!("Expected a dictionary")
        };
        let Ok((i, Some((ValuePull::ByteString(b"foo"), ValuePull::Integer(42))))) =
            it.next_pair(i)
        else {
            panic!("Expected foo -> 42");
        };
        let Ok((i, Some((ValuePull::ByteString(b"bar"), ValuePull::Integer(69))))) =
            it.next_pair(i)
        else {
            panic!("Expected bar -> 69");
        };
        let Ok((b"", None)) = it.next_pair(i) else {
            panic!("Expected end of dictionary")
        };
    }
}
