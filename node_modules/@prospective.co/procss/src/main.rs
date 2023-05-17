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

/// This needs to be here or the wasm build will not export any library symbols.
#[allow(unused_imports)]
use procss::*;

#[cfg(not(target_arch = "wasm32"))]
mod init {
    use std::{env, fs};

    use procss::*;

    pub fn init() -> anyhow::Result<String> {
        let args: Vec<String> = env::args().collect();
        let contents = fs::read_to_string(&args[1]);
        let css = parse(&contents?)?.flatten_tree().as_css_string();
        Ok(css)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    match init::init() {
        Ok(x) => println!("{}", x),
        Err(x) => eprintln!("{}", x),
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
