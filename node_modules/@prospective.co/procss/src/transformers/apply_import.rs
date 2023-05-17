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
use std::path::Path;

use super::filter_refs;
use crate::ast::Ruleset::{self};
use crate::ast::*;

pub fn apply_import<'a, 'b>(assets: &'b HashMap<&Path, Tree<'a>>) -> impl Fn(&mut Tree<'a>) + 'b {
    |tree| {
        tree.transform(|ruleset| {
            let mut replace = None;
            if let Ruleset::QualRule(QualRule(name, val)) = ruleset {
                if *name == "import" {
                    if let Some(val) = val {
                        if val.starts_with('\"') {
                            replace = assets.get(Path::new(&val[1..val.len() - 1])).cloned();
                            if replace.is_none() {
                                panic!("File not found: '{}'", &val[1..val.len() - 1])
                            }
                        } else if val.starts_with("url(\"ref://") {
                            replace = assets.get(Path::new(&val[11..val.len() - 2])).cloned().map(
                                |mut x| {
                                    filter_refs(&mut x);
                                    x
                                },
                            );

                            if replace.is_none() {
                                panic!("File not found: '{}'", &val[1..val.len() - 1])
                            }
                        }
                    }
                }
            }

            if let Some(replace) = replace {
                let rules: Vec<TreeRule<'a>> =
                    replace.iter().cloned().map(TreeRule::Ruleset).collect();

                *ruleset =
                    Ruleset::SelectorRuleset(SelectorRuleset(Selector::default(), rules.clone()))
            }
        });
    }
}
