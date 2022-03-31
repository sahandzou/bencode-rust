extern crate nom;

use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fs;
use std::str::from_utf8;
use nom::{Err, IResult};
use nom::branch::alt;
use nom::bytes::complete::take;
use nom::sequence::{terminated, preceded, pair};
use nom::character::complete::{char, digit1};
use nom::combinator::{map, map_res, opt, recognize};
use nom::multi::many1;

#[derive(Debug, PartialEq, Clone)]
pub enum BValue {
    BNumber(i64),
    BString(String),
    BBytes(Vec<u8>),
    BList(Vec<BValue>),
    BDict(HashMap<String, BValue>),
}

fn parse_number(i: &[u8]) -> IResult<&[u8], i64> {
    let num = recognize(pair(opt(char('-')), digit1));
    let parsed_num = map_res(num, |s: &[u8]| from_utf8(s).unwrap().parse::<i64>());
    terminated(preceded(char('i'), parsed_num), char('e'))(i)
}

fn parse_string(i: &[u8]) -> IResult<&[u8], String> {
    let (right, len) = parse_length(i)?;
    map_res(take(len), |s: &[u8]| from_utf8(s).unwrap().parse::<String>())(right)
}

fn parse_bytes(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (right, len) = parse_length(i)?;
    map(take(len), |b: &[u8]| b.to_vec())(right)
}

fn parse_length(i: &[u8]) -> IResult<&[u8], u32> {
    let len = terminated(digit1, char(':'));
    map_res(len, |s: &[u8]| from_utf8(s).unwrap().parse::<u32>())(i)
}

fn parse_list(i: &[u8]) -> IResult<&[u8], Vec<BValue>> {
    terminated(preceded(char('l'), many1(parse)), char('e'))(i)
}

fn parse_dict(i: &[u8]) -> IResult<&[u8], HashMap<String, BValue>> {
    let key_values = many1(pair(parse_string, parse));
    let dict_parser = terminated(preceded(char('d'), key_values), char('e'));
    map(dict_parser, |v| v.into_iter().collect())(i)
}

fn parse(i: &[u8]) -> IResult<&[u8], BValue> {
    let b_number = map(parse_number, BValue::BNumber);
    let b_string = map(parse_bytes, BValue::BBytes);
    let b_list = map(parse_list, BValue::BList);
    let b_dict = map(parse_dict, BValue::BDict);
    alt((b_number, b_string, b_list, b_dict))(i)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse(b"i-1337e"), Ok((&b""[..], BValue::BNumber(-1337))));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse(b"4:spam"), Ok((&b""[..], BValue::BBytes("spam".into()))));
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(parse(b"l4:spami42ee"), Ok((&b""[..], BValue::BList(vec![BValue::BBytes("spam".into()), BValue::BNumber(42)]))));
    }

    #[test]
    fn test_parse_dict() {
        let dict = HashMap::from([
            ("bar".to_string(), BValue::BBytes("spam".into())),
            ("foo".to_string(), BValue::BNumber(42))
        ]);
        assert_eq!(parse(b"d3:bar4:spam3:fooi42ee"), Ok((&b""[..], BValue::BDict(dict))));
    }
}

fn main() {
    let contents = include_bytes!("../examples/ubuntu-20.04.4-desktop-amd64.iso.torrent");
    let parser = parse(contents.as_slice());
}
