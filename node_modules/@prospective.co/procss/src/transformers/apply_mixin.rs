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

use std::collections::HashMap;

use crate::ast::Ruleset::{self};
use crate::ast::*;

/// Apply any in-scope mixin (defined using `@mixin`) to any `@include` in the
/// [`Tree`] `input`.  Similar to [Sass `@mixin`](https://sass-lang.com/documentation/at-rules/mixin)
///
/// # Example
///
/// ```
/// # use procss::{parse, transformers::apply_mixin, RenderCss};
/// let css = "
/// @mixin test {
///     opacity: 0;
/// }
/// div.open {
///     color: red;
///     @include test;
/// }
/// ";
/// let mut tree = parse(css).unwrap();
/// apply_mixin(&mut tree);
/// let css = tree.flatten_tree().as_css_string();
/// assert_eq!(css, "div.open{color:red;}div.open{opacity:0;}");
/// ```
pub fn apply_mixin<'a>(tree: &mut Tree<'a>) {
    let mut mixins: HashMap<&'a str, Vec<TreeRule<'a>>> = HashMap::new();
    tree.transform(|ruleset| {
        if let Ruleset::QualRuleset(crate::ast::QualRuleset(QualRule(name, Some(val)), props)) = ruleset {
            if *name == "mixin"  {
                mixins.insert(val.trim(), props.clone());
            }
        }

        if matches!(ruleset,  Ruleset::QualRuleset(QualRuleset(QualRule(name, _), _)) if *name == "mixin") {
            *ruleset = Ruleset::SelectorRuleset(SelectorRuleset(Selector::default(), vec![]))
        }
    });

    let mut count = 5;
    while count > 0 {
        let mut changed = false;
        tree.transform(|ruleset| {
            for (header, props) in mixins.iter() {
                if matches!(ruleset, Ruleset::QualRule(QualRule("include", Some(val))) if val == header ) {
                    changed = true;
                    *ruleset = Ruleset::SelectorRuleset(SelectorRuleset(
                        Selector::default(),
                        props.clone().into_iter().collect(),
                    ))
                }
            }
        });

        if !changed {
            count = 0;
        } else {
            count -= 1;
        }
    }
}
