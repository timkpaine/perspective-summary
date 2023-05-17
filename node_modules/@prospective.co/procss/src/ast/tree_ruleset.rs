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
use nom::bytes::complete::tag;
use nom::combinator::peek;
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::{terminated, tuple};
use nom::{IResult, Parser};

use super::flat_ruleset::FlatRuleset;
use super::ruleset::{QualNestedRuleset, QualRule, QualRuleset, Rule, Ruleset, SelectorRuleset};
use super::selector::Selector;
use super::token::{comment0, sep0};
use crate::parser::*;
use crate::render::*;
use crate::transform::TransformCss;

/// A tree node which expresses a recursive `T` over `Ruleset<T>`.  Using this
/// struct in place of `Rule` allows nested CSS selectors that can be later
/// flattened.
#[derive(Clone, Debug)]
pub enum TreeRule<'a> {
    Rule(Rule<'a>),
    Ruleset(TreeRuleset<'a>),
}

impl<'a> ParseCss<'a> for TreeRule<'a> {
    fn parse<E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        let block = terminated(TreeRuleset::parse, sep0).map(TreeRule::Ruleset);
        let rule = terminated(Rule::parse, sep0).map(TreeRule::Rule);
        alt((block, rule))(input)
    }
}

impl<'a> RenderCss for TreeRule<'a> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeRule::Rule(rule) => rule.render(f),
            TreeRule::Ruleset(block) => block.render(f),
        }
    }
}

impl<'a> TransformCss<Rule<'a>> for TreeRule<'a> {
    fn transform_each<F: FnMut(&mut Rule<'a>)>(&mut self, f: &mut F) {
        match self {
            TreeRule::Rule(rule) => f(rule),
            TreeRule::Ruleset(ruleset) => ruleset.transform_each(f),
        }
    }
}

impl<'a> TransformCss<TreeRuleset<'a>> for TreeRule<'a> {
    fn transform_each<F: FnMut(&mut TreeRuleset<'a>)>(&mut self, f: &mut F) {
        match self {
            TreeRule::Rule(_) => (),
            TreeRule::Ruleset(ruleset) => ruleset.transform_each(f),
        }
    }
}

impl<'a> TransformCss<Vec<TreeRuleset<'a>>> for TreeRule<'a> {
    fn transform_each<F: FnMut(&mut Vec<TreeRuleset<'a>>)>(&mut self, f: &mut F) {
        match self {
            TreeRule::Rule(_) => (),
            TreeRule::Ruleset(ruleset) => ruleset.transform_each(f),
        }
    }
}

/// A nested recursive block, ala popular CSS tools and the CSS nesting
/// proposal.
///
/// ```css
/// div {
///     color: green;
///     &#my_elem {
///         color: red;
///     }
///     .sub_elem {
///         color: purple;
///     }
/// }
/// ```
pub type TreeRuleset<'a> = Ruleset<'a, TreeRule<'a>>;

impl<'a> ParseCss<'a> for TreeRuleset<'a> {
    fn parse<E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        if let Ok((input, _)) = peek::<_, _, E, _>(tag("@"))(input) {
            let (input, qual_rule) = QualRule::parse(input)?;
            if let Ok((input, _)) = tag::<_, _, E>(";")(input) {
                Ok((input, Ruleset::QualRule(qual_rule)))
            } else {
                let (input, _) = tuple((tag("{"), sep0))(input)?;
                let (input, rules) = many1(TreeRule::parse::<E>)(input)?;
                let (input, _) = tuple((comment0, tag("}")))(input)?;
                Ok((input, Ruleset::QualRuleset(QualRuleset(qual_rule, rules))))
            }
        } else {
            let (input, selector_ruleset) = SelectorRuleset::parse(input)?;
            Ok((input, Ruleset::SelectorRuleset(selector_ruleset)))
        }
    }
}

impl<'a> TransformCss<TreeRuleset<'a>> for TreeRuleset<'a> {
    fn transform_each<F: FnMut(&mut TreeRuleset<'a>)>(&mut self, f: &mut F) {
        f(self);
        match self {
            Ruleset::QualRule(_) => (),
            Ruleset::QualRuleset(_) => (),
            Ruleset::QualNestedRuleset(..) => (),
            Ruleset::SelectorRuleset(ruleset) => {
                for rule in ruleset.1.iter_mut() {
                    rule.transform_each(f)
                }
            }
        }
    }
}

impl<'a> TreeRuleset<'a> {
    /// Flatten into a `FlatRuleset`, replacing this struct's inner `TreeRule`
    /// recursive type with a regular `Rule`, removing arbitrary nesting of
    /// `SelectorRuleset` variants.
    pub fn flatten_tree(&self) -> Vec<FlatRuleset<'a>> {
        match self {
            Ruleset::SelectorRuleset(ruleset) => ruleset.flatten_tree(),
            Ruleset::QualRule(x) => vec![Ruleset::QualRule(x.clone())],
            Ruleset::QualRuleset(rules) => {
                let mut new_rules: Vec<Rule<'a>> = vec![];
                let mut new_rulesets: Vec<FlatRuleset<'a>> = vec![];
                for rule in rules.1.iter() {
                    match rule {
                        TreeRule::Rule(rule) => new_rules.push(rule.clone()),
                        TreeRule::Ruleset(ruleset) => {
                            let sub_rules = ruleset.flatten_tree().into_iter();
                            new_rulesets.extend(sub_rules)
                        }
                    }
                }

                let mut ret = vec![];
                if !new_rules.is_empty() {
                    let ruleset = QualRuleset(rules.0.clone(), new_rules);
                    ret.push(Ruleset::QualRuleset(ruleset));
                }

                if !new_rulesets.is_empty() {
                    ret.push(Ruleset::QualNestedRuleset(QualNestedRuleset(
                        rules.0.clone(),
                        new_rulesets,
                    )))
                }

                ret
            }
            Ruleset::QualNestedRuleset(ruleset) => {
                vec![Ruleset::QualNestedRuleset(QualNestedRuleset(
                    ruleset.0.clone(),
                    ruleset.1.iter().flat_map(|x| x.flatten_tree()).collect(),
                ))]
            }
        }
    }
}

type TreeSelectorRuleset<'a> = SelectorRuleset<'a, TreeRule<'a>>;

impl<'a> ParseCss<'a> for TreeSelectorRuleset<'a> {
    fn parse<E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        let (input, selector) = Selector::parse(input)?;
        let (input, _) = tuple((comment0, tag("{"), sep0))(input)?;
        let (input, rules) = many0(TreeRule::parse)(input)?;
        let (input, _) = tuple((comment0, tag("}")))(input)?;
        Ok((input, SelectorRuleset(selector, rules)))
    }
}

impl<'a> TransformCss<Vec<TreeRuleset<'a>>> for TreeRuleset<'a> {
    fn transform_each<F: FnMut(&mut Vec<TreeRuleset<'a>>)>(&mut self, f: &mut F) {
        match self {
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

impl<'a> TreeSelectorRuleset<'a> {
    /// Flatten a TreeRuleset's SelectorRuleset into a `FlatRuleset`m erging any
    /// nested rulesets which are not allowed in the latter.
    pub fn flatten_tree(&self) -> Vec<Ruleset<'a, Rule<'a>>> {
        let mut new_rules: Vec<Rule<'a>> = vec![];
        let mut new_rulesets: Vec<FlatRuleset<'a>> = vec![];
        for rule in self.1.iter() {
            match rule {
                TreeRule::Rule(rule) => new_rules.push(rule.clone()),
                TreeRule::Ruleset(ruleset) => {
                    if !new_rules.is_empty() {
                        let ruleset = SelectorRuleset(self.0.clone(), new_rules);
                        new_rulesets.push(Ruleset::SelectorRuleset(ruleset));
                        new_rules = vec![];
                    }

                    let sub_rules = ruleset
                        .flatten_tree()
                        .into_iter()
                        .map(|ruleset| self.join(ruleset));
                    new_rulesets.extend(sub_rules)
                }
            }
        }

        if !new_rules.is_empty() {
            let ruleset = SelectorRuleset(self.0.clone(), new_rules);
            new_rulesets.push(Ruleset::SelectorRuleset(ruleset));
        }

        new_rulesets
    }

    /// Join a new `Ruleset` as an extension of self's selector.
    fn join(&self, rhs: Ruleset<'a, Rule<'a>>) -> Ruleset<'a, Rule<'a>> {
        match rhs {
            Ruleset::SelectorRuleset(inner_ruleset) => {
                let joined_selector = self.0.join(&inner_ruleset.0);
                Ruleset::SelectorRuleset(SelectorRuleset(joined_selector, inner_ruleset.1))
            }
            ruleset => ruleset,
        }
    }
}
