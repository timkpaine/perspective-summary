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

use crate::ast::Ruleset::{self};
use crate::ast::*;

pub fn filter_refs(tree: &mut Tree) {
    *tree = Tree(
        tree.iter()
            .cloned()
            .filter(|y| match y {
                Ruleset::SelectorRuleset(_) => false,
                Ruleset::QualRule(_) => true,
                Ruleset::QualRuleset(_) => true,
                Ruleset::QualNestedRuleset(_) => true,
            })
            .collect(),
    )
}
