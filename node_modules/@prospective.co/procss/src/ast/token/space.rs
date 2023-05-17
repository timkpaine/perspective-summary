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
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{anychar, multispace1};
use nom::error::ParseError;
use nom::multi::{many0, many1, many_till};
use nom::sequence::preceded;
use nom::IResult;

/// An "extension" trait for [`str`] which is used frequently to determine
/// whether whitepsace can be removed during [`crate::render::RenderCss`]
pub trait NeedsWhitespaceStringExt {
    /// Does this string needs a leading whitespace character?
    fn needs_pre_ws(&self) -> bool;

    /// Does this string needs a trailing whitespace character?
    fn needs_post_ws(&self) -> bool;
}

impl NeedsWhitespaceStringExt for str {
    fn needs_pre_ws(&self) -> bool {
        self.chars()
            .next()
            .map(|x| x.is_ascii_alphanumeric() || x == '-' || x == '_' || x == '%' || x == '+')
            .unwrap_or_default()
    }

    fn needs_post_ws(&self) -> bool {
        self.chars()
            .last()
            .map(|x| x.is_ascii_alphanumeric() || x == '-' || x == '_' || x == '%' || x == '+')
            .unwrap_or_default()
    }
}

/// Render `s` trimming all intermediate whitespace to a single character along
/// the way.
pub fn trim_whitespace(s: &str, f: &mut std::fmt::Formatter<'_>) {
    let mut last_alpha = false;
    s.split_whitespace().for_each(|w| {
        if last_alpha && w.needs_pre_ws() {
            write!(f, " ").unwrap();
        }

        last_alpha = w.needs_post_ws();
        write!(f, "{}", w).unwrap();
    });
}

// pub fn trim_whitespace(s: &str, f: &mut std::fmt::Formatter<'_>) {
//     let mut flag = false;
//     s.split_whitespace().for_each(|w| {
//         if flag {
//             write!(f, " ").unwrap();
//         }

//         flag = flag || !w.is_empty();
//         write!(f, "{}", w).unwrap();
//     });
// }

fn parse_comment<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str>,
{
    ignore(preceded(tag("//"), many0(is_not("\r\n"))))(input)
}

fn parse_multi_comment<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str>,
{
    ignore(preceded(tag("/*"), many_till(anychar, tag("*/"))))(input)
}

fn ignore<'a, T, E, F>(mut f: F) -> impl FnMut(&'a str) -> IResult<&'a str, (), E>
where
    F: FnMut(&'a str) -> IResult<&'a str, T, E>,
{
    move |input| {
        let (input, _) = f(input)?;
        Ok((input, ()))
    }
}

/// Parses 0 or more whitespace characters, including comments.
pub fn comment0<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str>,
{
    let (input, _) = many0(alt((
        ignore(multispace1),
        parse_comment,
        parse_multi_comment,
    )))(input)?;
    Ok((input, ()))
}

/// Parses 1 or more whitespace characters, including comments.
pub fn comment1<'a, E>(input: &'a str) -> IResult<&'_ str, (), E>
where
    E: ParseError<&'a str>,
{
    ignore(many1(alt((
        ignore(multispace1),
        parse_comment,
        parse_multi_comment,
    ))))(input)
}

/// Parses 0 or more whitespace characters, including comments and semicolons.
pub fn sep0<'a, E>(input: &'a str) -> IResult<&'_ str, (), E>
where
    E: ParseError<&'a str>,
{
    ignore(many0(alt((comment1, ignore(tag(";"))))))(input)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_multiline_comment() {
        assert_matches!(
            comment0::<()>(
                "
    /* 
     * test
     */"
            ),
            Ok(("", ()))
        )
    }

    #[test]
    fn test_forward_slash() {
        assert_matches!(comment0::<()>("// test"), Ok(("", ())))
    }

    #[test]
    fn test_semicolons() {
        assert_matches!(comment0::<()>("/* test; test */"), Ok(("", ())))
    }
}
