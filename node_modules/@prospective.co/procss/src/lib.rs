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

//! A simple CSS parsing and transformation framework. Procss can be used to
//! quickly bundle a collection of CSS+ files, or write your own custom
//! transforms.
//!
//! # Usage
//!
//! Procss's parser understands a nested superset of CSS (which we refer to as
//! CSS+), similar to the [CSS nesting proposal](https://www.w3.org/TR/css-nesting-1/),
//! or languages like [Sass](https://sass-lang.com).  Start with source CSS+
//! as a [`str`], use [crate::parse] or [crate::parse_unchecked] to generate
//! an [`ast::Tree`].
//!
//! ```
//! use procss::{ast, parse};
//!
//! let ast = procss::parse("div{.open{color:red;}}").unwrap();
//! ```
//!
//! The resulting [`ast::Tree`] can be converted to a de-nested [`ast::Css`]
//! with the [`ast::Tree::flatten_tree`] method, which itself can then be
//! rendered as a plain browser-readable CSS string via the
//! [`RenderCss::as_css_string`] trait method.
//!
//! ```
//! # use procss::{parse, ast};
//! # let ast = procss::parse("div{.open{color:red;}}").unwrap();
//! use procss::RenderCss;
//!
//! let flat: ast::Css = ast.flatten_tree();
//! let css: String = flat.as_css_string();
//! assert_eq!(css, "div .open{color:red;}");
//! ```
//!
//! Intermediate structs [`ast::Css::transform`] amd [`ast::Tree::transform`]
//! can be used to recursively mutate a tree for a variety of node structs in
//! the [`ast`] module.  Some useful Example of such transforms can be
//! found in the [`transformers`] module.
//!
//! ```
//! # use procss::{parse, RenderCss};
//! use procss::transformers;
//!
//! let test = "
//! @mixin test {color: red;}
//! div {@include test;}
//! ";
//!
//! let mut ast = procss::parse(test).unwrap();
//! transformers::apply_mixin(&mut ast);
//! let flat = ast.flatten_tree().as_css_string();
//! assert_eq!(flat, "div{color:red;}");
//! ```
//!
//! For coordinating large builds on a tree of CSS files, the [`BuildCss`]
//! struct can parse and minify, applying all transforms (including
//! [`transformers::apply_import`]) as the compilation is left-folded over the
//! inputs.
//!
//! ```no_run
//! let mut build = procss::BuildCss::new("./src");
//! build.add_file("controls/menu.scss");
//! build.add_file("logout.scss"); // imports "controls/menu.scss"
//! build.add_file("my_app.scss"); // imports "controls/menu.scss" and "logout.scss"
//! build.compile().unwrap().write("./dist").unwrap();
//! ```

#![feature(assert_matches)]
#![feature(path_file_prefix)]

pub mod ast;
mod builder;
#[cfg(target_arch = "wasm32")]
mod js_builder;
mod parser;
mod render;
mod transform;
pub mod transformers;
#[cfg(feature = "iotest")]
pub mod utils;

#[cfg(not(feature = "iotest"))]
pub mod utils;

use self::ast::Tree;
pub use self::builder::BuildCss;
use self::parser::{unwrap_parse_error, ParseCss};
pub use self::render::RenderCss;

/// Parse CSS text to a [`Tree`] (where it can be further manipulated),
/// capturing detailed error reporting for a moderate performance impact (using
/// [`nom::error::VerboseError`]).
///
/// # Example
///
/// ```rust
/// let ast = procss::parse("div { .open { color: red; }}").unwrap();
/// ```
pub fn parse(input: &str) -> anyhow::Result<Tree<'_>> {
    let (_, tree) = Tree::parse(input).map_err(|err| unwrap_parse_error(input, err))?;
    Ok(tree)
}

/// Parse CSS text to a [`Tree`], without capturing error details, for maximum
/// performance without any error details when parsing fails.
///
/// # Example
///
/// ```rust
/// let ast = procss::parse_unchecked("div { .open { color: red; }}").unwrap();
/// ```
pub fn parse_unchecked(input: &str) -> anyhow::Result<Tree<'_>> {
    let (_, tree) = Tree::parse::<()>(input)?;
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_verbose_error() {
        assert_matches!(
            parse("div{color:red").map(|x| x.as_css_string()).as_deref(),
            Err(_)
        )
    }

    #[test]
    fn test_parse_unchecked() {
        assert_matches!(
            parse_unchecked("div{color:red}")
                .map(|x| x.as_css_string())
                .as_deref(),
            Ok("div{color:red;}")
        )
    }
}

// `iotest` feature flag stubs out disk-accessing and other performance
// neutering function
#[cfg(all(not(feature = "iotest"), test))]
compile_error!("Feature 'iotest' must be enabled, rerun with:\n\n> cargo xtest");
