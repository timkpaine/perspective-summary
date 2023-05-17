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

use crate::ast::{Css, SelectorPath};
use crate::transform::*;

/// Remove `&` references from a flattened `Css`.
pub(crate) fn flat_self(css: &mut Css) {
    css.transform_each(&mut |x: &mut SelectorPath| {
        let res = match x {
            SelectorPath::Cons(..) => None,
            SelectorPath::PartialCons(_, tail) if !tail.is_empty() => {
                Some(SelectorPath::Cons(tail.remove(0).1, tail.clone()))
            }
            _ => None,
        };

        if let Some(y) = res {
            *x = y;
        }
    });
}
