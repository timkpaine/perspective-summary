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

//! The complete AST structs for [`Css`] and [`Tree`]. These top-level structs
//! are part of the [`crate::parse`] API, and their various components structs
//! are necessary for writing any transformations on these between parsing and
//! rendering.

mod flat_ruleset;
mod ruleset;
mod selector;
mod token;
mod tree_ruleset;

use std::cmp::Ordering;

use nom::branch::alt;
use nom::combinator::eof;
use nom::error::ParseError;
use nom::multi::many1;
use nom::sequence::terminated;
use nom::{IResult, Parser};

pub use self::flat_ruleset::FlatRuleset;
pub use self::ruleset::{QualNestedRuleset, QualRule, QualRuleset, Rule, Ruleset, SelectorRuleset};
pub use self::selector::{Combinator, Selector, SelectorAttr, SelectorPath, SelectorTerm};
pub use self::tree_ruleset::{TreeRule, TreeRuleset};
use crate::parser::*;
use crate::render::*;
use crate::transform::*;
use crate::transformers;

/// A non-nested "flat" CSS representation, suitable for browser output. The
/// [`Css`] AST is typically generated via the
/// [`crate::ast::Tree::flatten_tree`] method.
#[derive(Clone, Debug)]
pub struct Css<'a>(pub Vec<FlatRuleset<'a>>);

impl<'a> Css<'a> {
    /// A mutable transform which walks this AST recursively, invoking `f` for
    /// all nodes of type `T`.
    ///
    /// # Example
    ///
    /// ```
    /// use procss::{ast, parse, RenderCss};
    /// let mut css = parse("div{color: red;}").unwrap().flatten_tree();
    /// css.transform(|rule: &mut ast::Rule| {
    ///     rule.value = "green".into();
    /// });
    ///
    /// assert_eq!(css.as_css_string(), "div{color:green;}");
    /// ```
    pub fn transform<T>(&mut self, mut f: impl FnMut(&mut T))
    where
        Self: TransformCss<T>,
    {
        self.transform_each(&mut f)
    }

    /// Iterate over the immediate children of this Tree (non-recursive).
    pub fn iter(&self) -> impl Iterator<Item = &'_ FlatRuleset<'a>> {
        self.0.iter()
    }
}

impl<'a, T> TransformCss<T> for Css<'a>
where
    Ruleset<'a, Rule<'a>>: TransformCss<T>,
{
    fn transform_each<F: FnMut(&mut T)>(&mut self, f: &mut F) {
        for rule in self.0.iter_mut() {
            rule.transform_each(f);
        }
    }
}

impl<'a> RenderCss for Css<'a> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in self.0.iter() {
            x.render(f)?;
        }

        Ok(())
    }
}

/// A nested CSS representation yielded from parsing. [`Tree`] also implements
/// [`RenderCss`] if this is needed, though this output can't be read by
/// browsers and is not identical to the input since whitespace has been
/// discarded.
#[derive(Clone, Debug)]
pub struct Tree<'a>(pub Vec<TreeRuleset<'a>>);

impl<'a> Tree<'a> {
    /// Flatten a nested [`Tree`] into a [`Css`], or in other words, convert the
    /// result of a parse into something that can be rendered.
    pub fn flatten_tree(&self) -> Css<'a> {
        let mut rules = self
            .0
            .iter()
            .flat_map(|x| x.flatten_tree())
            .collect::<Vec<_>>();

        rules.sort_by(|x, y| match (x, y) {
            (Ruleset::QualRule(_), Ruleset::QualRule(_)) => Ordering::Equal,
            (Ruleset::QualRule(_), _) => Ordering::Less,
            (_, Ruleset::QualRule(_)) => Ordering::Greater,
            _ => Ordering::Equal,
        });

        let mut css = Css(rules);
        transformers::flat_self(&mut css);
        css
    }

    /// A mutable transform which walks this AST recursively, invoking `f` for
    /// all nodes of type `T`.
    ///
    /// # Example
    ///
    /// ```
    /// use procss::{ast, parse, RenderCss};
    /// let mut tree = parse("div{color: red;}").unwrap();
    /// tree.transform(|rule: &mut ast::Rule| {
    ///     rule.value = "green".into();
    /// });
    ///
    /// let css = tree.flatten_tree().as_css_string();
    /// assert_eq!(css, "div{color:green;}");
    /// ```
    pub fn transform<T>(&mut self, mut f: impl FnMut(&mut T))
    where
        Self: TransformCss<T>,
    {
        self.transform_each(&mut f)
    }

    /// Iterate over the immediate children of this Tree (non-recursive).
    pub fn iter(&self) -> impl Iterator<Item = &'_ TreeRuleset<'a>> {
        self.0.iter()
    }
}

impl<'a> TransformCss<Rule<'a>> for Tree<'a> {
    fn transform_each<F: FnMut(&mut Rule<'a>)>(&mut self, f: &mut F) {
        for rule in self.0.iter_mut() {
            rule.transform_each(f);
        }
    }
}

impl<'a> TransformCss<TreeRuleset<'a>> for Tree<'a> {
    fn transform_each<F: FnMut(&mut TreeRuleset<'a>)>(&mut self, f: &mut F) {
        for rule in self.0.iter_mut() {
            f(rule);
            match rule {
                Ruleset::SelectorRuleset(ruleset) => {
                    for rule in ruleset.1.iter_mut() {
                        rule.transform_each(f)
                    }
                }
                Ruleset::QualRule(_) => (),
                Ruleset::QualRuleset(_) => (),
                Ruleset::QualNestedRuleset(ruleset) => {
                    for rule in ruleset.1.iter_mut() {
                        rule.transform_each(f)
                    }
                }
            }
        }
    }
}

impl<'a> TransformCss<Vec<TreeRuleset<'a>>> for Tree<'a> {
    fn transform_each<F: FnMut(&mut Vec<TreeRuleset<'a>>)>(&mut self, f: &mut F) {
        f(&mut self.0);
        for rule in self.0.iter_mut() {
            // f(rule);
            rule.transform_each(f);
        }
    }
}

impl<'a> ParseCss<'a> for Tree<'a> {
    fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let (input, _) = token::sep0(input)?;
        let (input, x) = alt((
            eof.map(|_| vec![]),
            terminated(many1(terminated(TreeRuleset::parse, token::sep0)), eof),
        ))(input)?;

        Ok((input, Tree(x)))
    }
}

impl<'a> RenderCss for Tree<'a> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in self.0.iter() {
            x.render(f)?;
        }

        Ok(())
    }
}
