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

use nom::bytes::complete::{is_not, tag};
use nom::combinator::opt;
use nom::error::ParseError;
use nom::sequence::{preceded, tuple};
use nom::IResult;

use crate::ast::token::*;
use crate::parser::*;

/// A selector which matches attributes, optionally against their value as well.
/// TODO doesn't support comma-separated multiple selectors.
///
/// # Example
///
/// ```css
/// div[name=test] {}
/// div[disabled,data-value="red"] {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SelectorAttr<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

impl<'a> ParseCss<'a> for SelectorAttr<'a> {
    fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let (rest, (_, name, value, _)) = tuple((
            tag("["),
            parse_symbol,
            opt(preceded(tag("="), is_not("]"))),
            tag("]"),
        ))(input)?;
        Ok((rest, SelectorAttr { name, value }))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_bool() {
        assert_matches!(
            SelectorAttr::parse::<()>("[disabled]"),
            Ok(("", SelectorAttr {
                name: "disabled",
                value: None
            }))
        )
    }

    #[test]
    fn test_value_quotes() {
        assert_matches!(
            SelectorAttr::parse::<()>("[data-value=\"red\"]"),
            Ok(("", SelectorAttr {
                name: "data-value",
                value: Some("\"red\"")
            }))
        )
    }

    #[ignore]
    #[test]
    fn test_multiple() {
        assert_matches!(
            SelectorAttr::parse::<()>("[disabled,data-value=\"red\"]"),
            Ok(("", SelectorAttr {
                name: "data-value",
                value: Some("\"red\"")
            }))
        )
    }
}
