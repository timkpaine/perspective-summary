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
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::IResult;

use super::combinator::*;
use super::selector_term::*;
use crate::parser::*;
use crate::render::*;

/// A linked-list-like data structure representing CSS selector lists, which are
/// selectors separated by combinators like `>`, `+` or most commonly just
/// whitespace.  When the first selector in a `SelectorList` is the special
/// selector `&`, as special case `PartialCons` elimantes `tag` as a field from
/// this first Selector, preventing the `SelectorList` from being serialized
/// before being flattened.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum SelectorPath<'a> {
    Cons(
        SelectorTerm<'a, Option<&'a str>>,
        Vec<(Combinator, SelectorTerm<'a, Option<&'a str>>)>,
    ),
    PartialCons(
        SelectorTerm<'a, ()>,
        Vec<(Combinator, SelectorTerm<'a, Option<&'a str>>)>,
    ),
}

use SelectorPath::*;

impl<'a> SelectorPath<'a> {
    /// Utility method for cons-ing a `Selector` to the head of a
    /// `SelectorList`.
    fn cons(
        &self,
        selector: &SelectorTerm<'a, ()>,
        tail: Vec<(Combinator, SelectorTerm<'a, Option<&'a str>>)>,
    ) -> Self {
        match self {
            //Nil => Nil,
            Cons(x, _) => Cons(x.join(selector), tail),
            PartialCons(x, _) => PartialCons(x.join(selector), tail),
        }
    }

    /// Utility method for accessing the tail of a `SelectorList`.
    fn tail(&self) -> Vec<(Combinator, SelectorTerm<'a, Option<&'a str>>)> {
        match self {
            // Nil => vec![],
            Cons(_, tail) => tail.clone(),
            PartialCons(_, tail) => tail.clone(),
        }
    }

    /// Append two `SelectorList`, properly merging `&` references along the
    /// way.  For example:
    ///
    /// ```css
    /// div {               
    ///     &.enabled {  
    ///         color: red;
    ///     }
    /// }
    /// ```
    ///
    /// becomes
    ///
    /// ```css
    /// div.enabled {
    ///     color: red;
    /// }
    /// ```
    pub fn join(&self, other: &Self) -> Self {
        match (&self, other) {
            (head, Cons(selector, tail)) => {
                let mut new_tail = head.tail();
                new_tail.push((Combinator::Null, selector.clone()));
                new_tail.append(&mut tail.clone());
                head.cons(&SelectorTerm::default(), new_tail)
            }
            (head, PartialCons(selector2, tail2)) => {
                let mut new_tail = head.tail();
                match new_tail.pop() {
                    Some((c, last)) => {
                        new_tail.push((c, last.join(selector2)));
                        new_tail.extend(tail2.iter().cloned());
                        head.cons(&SelectorTerm::default(), new_tail)
                    }
                    None => {
                        new_tail.append(&mut tail2.clone());
                        head.cons(selector2, new_tail)
                    }
                }
            }
        }
    }
}

impl<'a> RenderCss for SelectorPath<'a> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Cons(selector, rest) => {
                selector.render(f)?;
                rest.render(f)?;
            }
            PartialCons(selector, rest) => {
                write!(f, "&")?;
                selector.render(f)?;
                rest.render(f)?;
            }
        };

        Ok(())
    }
}

impl<'a> ParseCss<'a> for SelectorPath<'a> {
    fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        alt((parse_selector_self_list, parse_selector_list))(input)
    }
}

/// Parse a selector list without a leading `&` tag, see note on
/// `parse_selector_self_list`.
fn parse_selector_list<'a, E>(input: &'a str) -> IResult<&'a str, SelectorPath<'a>, E>
where
    E: ParseError<&'a str>,
{
    let (rest, x) = SelectorTerm::parse(input)?;
    let (rest, combinators) = many0(tuple((Combinator::parse, SelectorTerm::parse)))(rest)?;
    Ok((rest, Cons(x, combinators)))
}

/// Parse a selector list starting with the `&` special tag via the
/// `Selector<()>` impl for the `Parser` trait, which is chosen here by the
/// compiler due to type inference, from the return binding later used as an
/// argument to the `PartialCons` variant constructor.
fn parse_selector_self_list<'a, E>(input: &'a str) -> IResult<&'a str, SelectorPath<'a>, E>
where
    E: ParseError<&'a str>,
{
    let (rest, x) = SelectorTerm::parse(input)?;
    let (rest, combinators) = many0(tuple((Combinator::parse, SelectorTerm::parse)))(rest)?;
    Ok((rest, PartialCons(x, combinators)))
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_null() {
        assert_matches!(
            SelectorPath::parse::<()>("div img"),
            Ok((
                "",
                SelectorPath::Cons(
                    SelectorTerm {
                        tag: Some("div"),
                        ..
                    },
                    xs
                )
            )) if xs.len() == 1 && xs[0].1.tag == Some("img")
        )
    }

    #[test]
    fn test_desc() {
        assert_matches!(
            SelectorPath::parse::<()>("div > img"),
            Ok((
                "",
                SelectorPath::Cons(
                    SelectorTerm {
                        tag: Some("div"),
                        ..
                    },
                    xs
                )
            )) if xs.len() == 1 && xs[0].1.tag == Some("img")
        )
    }

    #[test]
    fn test_self() {
        assert_matches!(
            SelectorPath::parse::<()>("& > img"),
            Ok((
                "",
                SelectorPath::PartialCons(
                    _,
                    xs
                )
            )) if xs.len() == 1 && xs[0].1.tag == Some("img")
        )
    }

    #[ignore]
    #[test]
    fn test_inner_self() {
        assert_matches!(
            SelectorPath::parse::<()>("@nest div & img"),
            Ok((
                "",
                SelectorPath::PartialCons(
                    _,
                    xs
                )
            )) if xs.len() == 1 && xs[0].1.tag == Some("img")
        )
    }
}
