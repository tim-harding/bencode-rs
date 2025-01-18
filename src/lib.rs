use nom::{
    branch::alt,
    bytes::streaming::take,
    character::streaming::{char, digit0, digit1},
    combinator::{map, opt, value, verify},
    sequence::{delimited, pair, tuple},
    IResult, Needed,
};

type Res<'a, O> = IResult<&'a [u8], O>;

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

fn is_nonzero(&b: &u8) -> bool {
    b >= b'1' && b <= b'9'
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
}
