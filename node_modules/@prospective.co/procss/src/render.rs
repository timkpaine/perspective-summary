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

use std::borrow::Cow;
use std::fmt;

/// A trick to etract the top-level `Formatter` when rendering to a string.
struct Fix<F: Fn(&mut fmt::Formatter<'_>)>(F);

impl<F: Fn(&mut fmt::Formatter<'_>)> fmt::Display for Fix<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f);
        Ok(())
    }
}

// Another form of [`std::fmt::Display`] etc.  This one is explicitly for
// generating valid CSS as a string.
pub trait RenderCss {
    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Render `self` as a valid CSS [`String`], minified (with
    /// non-interpretation-impacting whitespace removed).
    fn as_css_string(&self) -> String {
        format!(
            "{}",
            Fix(|fmt| {
                self.render(fmt).unwrap();
            })
        )
    }
}

impl<'a> RenderCss for &'a str {
    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a, T: RenderCss + Clone> RenderCss for Cow<'a, T> {
    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().render(f)
    }
}

impl<T: RenderCss> RenderCss for Option<T> {
    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Some(x) => x.render(f),
            None => Ok(()),
        }
    }
}

impl RenderCss for () {
    fn render(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl<T: RenderCss> RenderCss for Vec<T> {
    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in self {
            x.render(f)?;
        }

        Ok(())
    }
}

impl<T: RenderCss, U: RenderCss> RenderCss for (T, U) {
    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.render(f)?;
        self.1.render(f)
    }
}
