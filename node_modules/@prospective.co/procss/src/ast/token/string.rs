// ┌───────────────────────────────────────────────────────────────────────────┐
// │                                                                           │
// │  ██████╗ ██████╗  ██████╗   Copyright (C) 2022, The Prospective Company   │
// │  ██╔══██╗██╔══██╗██╔═══██╗                                                │
// │  ██████╔╝██████╔╝██║   ██║  This file is part of the Procss library,      │
// │  ██╔═══╝ ██╔══██╗██║   ██║  distributed under the terms of the            │
// │  ██║     ██║  ██║╚██████╔╝  Apache License 2.0.  The full license can     │
// │  ╚═╝     ╚═╝  ╚═╝ ╚═════╝   be found in the LICENSE file.                 │
// │                                                                           │
// └───────────────────────────────────────────────────────────────────────────┘

use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::char;
use nom::combinator::{map, recognize, verify};
use nom::error::{ErrorKind, ParseError};
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded};
use nom::IResult;

fn parse_escaped_char(input: &str) -> IResult<&str, &str> {
    recognize(preceded(char('\\'), is_not(" \r\t\n\"")))(input)
}

fn parse_literal(input: &str) -> IResult<&str, &str> {
    let not_quote_slash = is_not("\"\\");
    verify(not_quote_slash, |s: &str| !s.is_empty())(input)
}

enum StringFragment {
    Literal(usize),
    EscapedChar(usize),
}

impl StringFragment {
    fn len(&self) -> usize {
        match self {
            StringFragment::Literal(s) => *s,
            StringFragment::EscapedChar(s) => *s,
        }
    }
}

fn parse_fragment(input: &str) -> IResult<&str, StringFragment> {
    alt((
        map(parse_literal, |x| StringFragment::Literal(x.len())),
        map(parse_escaped_char, |x| StringFragment::EscapedChar(x.len())),
    ))(input)
}

pub fn parse_string_literal<'a, E: ParseError<&'a str>>(
) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, E> {
    move |input| {
        let build_string = fold_many0(parse_fragment, || 2, |len, frag| frag.len() + len);
        let offset = delimited(char('"'), build_string, char('"'));
        let res = map(offset, |x| &input[..x])(input);
        res.map_err(|_| nom::Err::Error(E::from_error_kind(input, ErrorKind::AlphaNumeric)))
    }
}
