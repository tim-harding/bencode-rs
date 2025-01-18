use nom::{
    branch::alt,
    character::streaming::{char, digit0},
    combinator::{map, opt, value, verify},
    sequence::{delimited, tuple},
    IResult, Needed,
};

fn int(i: &[u8]) -> IResult<&[u8], i64> {
    delimited(char('i'), alt((zero, nonzero)), char('e'))(i)
}

fn nonzero(i: &[u8]) -> IResult<&[u8], i64> {
    map(nonzero_raw, |(minus, head, tail)| {
        let mut out = (head - b'0') as i64;
        for c in tail {
            out = out * 10 + (c - b'0') as i64;
        }
        out * if minus { -1 } else { 1 }
    })(i)
}

fn nonzero_raw(i: &[u8]) -> IResult<&[u8], (bool, u8, &[u8])> {
    tuple((minus, verify(byte, is_nonzero), digit0))(i)
}

fn minus(i: &[u8]) -> IResult<&[u8], bool> {
    map(opt(char('-')), |minus| minus.is_some())(i)
}

fn zero(i: &[u8]) -> IResult<&[u8], i64> {
    value(0, char('0'))(i)
}

fn is_nonzero(&b: &u8) -> bool {
    b >= b'1' && b <= b'9'
}

fn byte(i: &[u8]) -> IResult<&[u8], u8> {
    match i {
        [head, tail @ ..] => Ok((tail, *head)),
        [] => Err(nom::Err::Incomplete(Needed::new(1))),
    }
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
}
