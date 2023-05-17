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

mod rule;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::multi::many0;
use nom::IResult;

pub use self::rule::Rule;
use super::selector::{Selector, SelectorPath};
use super::token::{
    comment0, parse_string_literal, parse_symbol, trim_whitespace, NeedsWhitespaceStringExt,
};
use crate::parser::ParseCss;
use crate::render::*;
use crate::transform::TransformCss;

/// ```css
/// div {
///     color: red;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SelectorRuleset<'a, T>(pub Selector<'a>, pub Vec<T>);

impl<'a, T: RenderCss> RenderCss for SelectorRuleset<'a, T> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.render(f)?;
        write!(f, "{{")?;
        self.1.render(f)?;
        write!(f, "}}")
    }
}

impl<'a> TransformCss<SelectorPath<'a>> for SelectorRuleset<'a, Rule<'a>> {
    fn transform_each<F: FnMut(&mut SelectorPath<'a>)>(&mut self, f: &mut F) {
        self.0.transform_each(f)
    }
}

/// ```css
/// @import "test.css";
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct QualRule<'a>(pub &'a str, pub Option<&'a str>);

impl<'a> ParseCss<'a> for QualRule<'a> {
    fn parse<E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        let (input, _) = tag("@")(input)?;
        let (input, tagname) = parse_symbol(input)?;
        let (input, _) = comment0(input)?;
        let (input, property) =
            recognize(many0(alt((is_not("\";{}"), parse_string_literal()))))(input)?;

        let property = if property.is_empty() {
            None
        } else {
            Some(property)
        };

        Ok((input, QualRule(tagname, property)))
    }
}

impl<'a> RenderCss for QualRule<'a> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@")?;
        trim_whitespace(self.0, f);
        if let Some(val) = self.1 {
            if val.needs_pre_ws() {
                write!(f, " ")?;
            }

            trim_whitespace(val, f);
        }

        write!(f, ";")
    }
}

/// ```css
/// @font-face {
///     font-family: "My Font";
/// };
/// ```
#[derive(Debug, Clone)]
pub struct QualRuleset<'a, T>(pub QualRule<'a>, pub Vec<T>);

impl<'a, T: RenderCss> RenderCss for QualRuleset<'a, T> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.1.is_empty() {
            write!(f, "@{}", self.0 .0)?;
            if let Some(val) = self.0 .1 {
                if val.needs_pre_ws() {
                    write!(f, " ")?;
                }

                trim_whitespace(val, f);
            }

            write!(f, "{{")?;
            self.1.render(f)?;
            write!(f, "}}")
        } else {
            Ok(())
        }
    }
}

impl<'a, T: TransformCss<U>, U> TransformCss<U> for QualRuleset<'a, T> {
    fn transform_each<F: FnMut(&mut U)>(&mut self, f: &mut F) {
        for ruleset in self.1.iter_mut() {
            ruleset.transform_each(f);
        }
    }
}

/// ```css
/// @media (max-width:1250px) {
///     @media (min-width:300px) {
///         div {
///             color: red;
///         }
///     }
///     span {
///         color: green;
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct QualNestedRuleset<'a, T>(pub QualRule<'a>, pub Vec<Ruleset<'a, T>>);

impl<'a, T: RenderCss> RenderCss for QualNestedRuleset<'a, T> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.1.is_empty() {
            let QualNestedRuleset(QualRule(name, val), node) = self;
            write!(f, "@{}", name)?;
            if let Some(val) = val {
                if val.needs_pre_ws() {
                    write!(f, " ")?;
                }

                trim_whitespace(val, f);
            }

            write!(f, "{{")?;
            node.render(f)?;
            write!(f, "}}")
        } else {
            Ok(())
        }
    }
}

impl<'a, T, U> TransformCss<U> for QualNestedRuleset<'a, T>
where
    Ruleset<'a, T>: TransformCss<U>,
{
    fn transform_each<F: FnMut(&mut U)>(&mut self, f: &mut F) {
        for ruleset in self.1.iter_mut() {
            ruleset.transform_each(f);
        }
    }
}

/// A collection of rules (or other `T`), delimited by `{}`.  `Ruleset` is
/// generic over the specific type of its `rules` field, which ultimately
/// determines whether this struct is recursive (in its `TreeRuleset` variant)
/// or not (in its `FlatRuleset` variant).
///
/// `Ruleset` implements `RenderCss`, but the more-specific type alias
/// `TreeRuleset` implements `ParseCss`.
///
/// Not allowed:
///
/// ```css
/// @media (max-width:1250px) {
///     color: green;
///     div {
///         color: red;
///     }
/// }
/// ```
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub enum Ruleset<'a, T> {
    SelectorRuleset(SelectorRuleset<'a, T>),
    QualRule(QualRule<'a>),
    QualRuleset(QualRuleset<'a, T>),
    QualNestedRuleset(QualNestedRuleset<'a, T>),
}

impl<'a, T: RenderCss> RenderCss for Ruleset<'a, T> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ruleset::SelectorRuleset(selector_ruleset) => selector_ruleset.render(f),
            Ruleset::QualRule(rule) => rule.render(f),
            Ruleset::QualRuleset(ruleset) => ruleset.render(f),
            Ruleset::QualNestedRuleset(ruleset) => ruleset.render(f),
        }
    }
}

impl<'a, T: TransformCss<Rule<'a>>> TransformCss<Rule<'a>> for Ruleset<'a, T> {
    fn transform_each<F: FnMut(&mut Rule<'a>)>(&mut self, f: &mut F) {
        match self {
            Ruleset::QualRule(_) => (),
            Ruleset::QualRuleset(rules) => rules.transform_each(f),
            Ruleset::QualNestedRuleset(ruleset) => ruleset.transform_each(f),
            Ruleset::SelectorRuleset(SelectorRuleset(_, rules)) => {
                for rule in rules.iter_mut() {
                    rule.transform_each(f);
                }
            }
        }
    }
}
