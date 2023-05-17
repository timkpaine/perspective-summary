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

mod attribute;
mod combinator;
mod selector_path;
mod selector_term;

use std::ops::Deref;

use nom::bytes::complete::tag;
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::{delimited, preceded};
use nom::IResult;

pub use self::attribute::SelectorAttr;
pub use self::combinator::Combinator;
pub use self::selector_path::SelectorPath;
pub use self::selector_term::SelectorTerm;
use super::token::comment0;
use crate::parser::*;
use crate::transform::TransformCss;
use crate::utils::*;

/// A set of selector alternatives separated by `,`, for example `div, span`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Selector<'a>(MinVec<SelectorPath<'a>, 1>);

impl<'a> Default for Selector<'a> {
    fn default() -> Self {
        //Selector::parse::<()>("&").unwrap().1
        Selector(MinVec::new(
            [SelectorPath::PartialCons(
                SelectorTerm {
                    tag: (),
                    ..SelectorTerm::default()
                },
                vec![],
            )],
            vec![],
        ))
    }
}

impl<'a> Deref for Selector<'a> {
    type Target = MinVec<SelectorPath<'a>, 1>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> ParseCss<'a> for Selector<'a> {
    fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let (input, selector) = SelectorPath::parse(input)?;
        let (input, extra) = many0(preceded(
            delimited(comment0, tag(","), comment0),
            SelectorPath::parse,
        ))(input)?;

        Ok((input, Selector(MinVec::new([selector], extra))))
    }
}

impl<'a> TransformCss<SelectorPath<'a>> for Selector<'a> {
    fn transform_each<F: FnMut(&mut SelectorPath<'a>)>(&mut self, f: &mut F) {
        for list in self.0.iter_mut() {
            f(list)
        }
    }
}

impl<'a> Selector<'a> {
    /// Create a new `SelectorGroup` from an `Iterator`, which will fail if
    /// there aren't enough elements.  This should asserted by the caller.
    fn new(mut iter: impl Iterator<Item = SelectorPath<'a>>) -> Option<Self> {
        Some(Selector(MinVec::new([iter.next()?], iter.collect())))
    }

    /// `SelectorGroup` uses the underlying `join()` method of the
    /// `SelectorList`, combined via the product of the two
    /// `SelectorGroup`'s items.  For example:
    ///
    /// ```css
    /// div, span {
    ///     .opened, :hover {
    ///         color: red;
    ///     }
    /// }
    /// ```
    ///
    /// becomes
    ///
    /// ```css
    /// div .opened, div :hover, span .opened, span :hover {
    ///     color: red;
    /// }
    /// ```
    pub fn join(&self, other: &Selector<'a>) -> Selector<'a> {
        let iter = self.0.iter().flat_map(|x| other.iter().map(|y| x.join(y)));
        Self::new(iter).unwrap()
    }
}
