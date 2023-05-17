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

use super::ruleset::{Rule, Ruleset};
use super::selector::SelectorPath;
use super::SelectorRuleset;
use crate::transform::TransformCss;

/// A flat (non-recursive) block, suitable for compatibility with modern
/// browsers.
///
/// ```css
/// div {
///     color: green;
/// }
/// div#my_elem {
///     color: red;
/// }
/// div .sub_elem {
///     color: purple;
/// }
/// ```
pub type FlatRuleset<'a> = Ruleset<'a, Rule<'a>>;

impl<'a> TransformCss<SelectorPath<'a>> for FlatRuleset<'a> {
    fn transform_each<F: FnMut(&mut SelectorPath<'a>)>(&mut self, f: &mut F) {
        match self {
            Ruleset::SelectorRuleset(ruleset) => ruleset.transform_each(f),
            Ruleset::QualRule(_) => (),
            Ruleset::QualRuleset(_) => (),
            Ruleset::QualNestedRuleset(ruleset) => ruleset.transform_each(f),
        }
    }
}

impl<'a> TransformCss<SelectorRuleset<'a, Rule<'a>>> for FlatRuleset<'a> {
    fn transform_each<F: FnMut(&mut SelectorRuleset<'a, Rule<'a>>)>(&mut self, f: &mut F) {
        match self {
            Ruleset::SelectorRuleset(ruleset) => f(ruleset),
            Ruleset::QualNestedRuleset(ruleset) => ruleset.transform_each(f),
            _ => (),
        }
    }
}
