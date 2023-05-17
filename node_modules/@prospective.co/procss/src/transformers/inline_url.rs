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
use std::path::Path;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::sequence::delimited;

use crate::ast::{Css, Rule};
use crate::utils::fs;
#[cfg(feature = "iotest")]
use crate::utils::IoTestFs;

#[cfg(not(target_arch = "wasm32"))]
fn read_file_sync(path: &Path) -> Option<Vec<u8>> {
    fs::read(path).ok()
}

#[cfg(target_arch = "wasm32")]
fn read_file_sync(path: &Path) -> Option<Vec<u8>> {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(module = "fs")]
    extern "C" {
        #[wasm_bindgen(catch)]
        fn readFileSync(path: &str) -> Result<Vec<u8>, JsValue>;
    }

    readFileSync(&*path.to_string_lossy()).ok()
}

fn parse_url(input: &str) -> nom::IResult<&str, &str> {
    let unquoted = delimited(tag("url("), is_not(")"), tag(")"));
    let quoted = delimited(tag("url(\""), is_not("\""), tag("\")"));
    alt((quoted, unquoted))(input)
}

fn into_data_uri<'a>(path: &Path) -> Option<Cow<'a, str>> {
    if path.starts_with("data:") {
        return None;
    }

    let contents = read_file_sync(path)?;
    let encoded = base64::encode(contents);
    let fff = path.extension().unwrap_or_default().to_string_lossy();
    let fmt = match fff.as_ref() {
        "png" => "png",
        _ => "svg+xml",
    };

    Some(format!("url(\"data:image/{};base64,{}\")", fmt, encoded).into())
}

fn inline_url_impl<'a>(newpath: &str, flat: &mut Css<'a>) {
    flat.transform::<Rule<'a>>(|rule| {
        let path = parse_url(&rule.value)
            .ok()
            .and_then(|x| x.0.is_empty().then_some(Path::new(newpath).join(x.1)));

        if let Some(path) = &path {
            if path.starts_with(".") || path.starts_with("/") {
                if let Some(value) = into_data_uri(path) {
                    rule.value = value;
                }
            }
        }
    })
}

/// Inline `url()` rule properties containing local paths to be replace with the
/// base64 encoded contents of their respective files.
pub fn inline_url<'a: 'b, 'b>(newpath: &'b str) -> impl Fn(&mut Css<'a>) + 'b {
    |flat| inline_url_impl(newpath, flat)
}
