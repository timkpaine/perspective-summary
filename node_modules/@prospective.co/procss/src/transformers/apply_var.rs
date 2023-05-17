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

pub fn apply_var<'a>(tree: &mut Tree<'a>) {
    let mut mixins: HashMap<&'a str, &'a str> = HashMap::new();
    tree.transform(|ruleset| {
        let mut is_mixin = false;
        if let Ruleset::QualRule(QualRule(name, Some(val))) = ruleset {
            if let Some(val) = val.strip_prefix(':') {
                mixins.insert(name, val);
                is_mixin = true;
            }
        }

        if is_mixin {
            *ruleset = Ruleset::QualRuleset(QualRuleset(QualRule("", None), vec![]))
        }
    });

    let mut mixins = mixins.iter().collect::<Vec<_>>();
    mixins.sort_by(|(x, _), (a, _)| x.len().cmp(&a.len()));
    mixins.reverse();
    tree.transform(|rule: &mut Rule| {
        for (var, val) in mixins.iter() {
            rule.value = rule.value.replace(&format!("@{}", var), val).into();
        }
    });
}
