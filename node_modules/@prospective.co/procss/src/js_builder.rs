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

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

/// An implementation of `BuildCss` which owns its data, suitable for use as an
/// exported type in JavaScript.
#[wasm_bindgen]
pub struct BuildCss {
    rootdir: String,
    inputs: HashMap<String, String>,
}

#[wasm_bindgen]
impl BuildCss {
    #[wasm_bindgen(constructor)]
    pub fn new(rootdir: String) -> Self {
        BuildCss {
            rootdir,
            inputs: HashMap::default(),
        }
    }

    pub fn add(&mut self, path: String, content: String) {
        self.inputs.insert(path, content);
    }

    pub fn compile(&self) -> core::result::Result<JsValue, JsError> {
        let mut build = crate::builder::BuildCss::new(self.rootdir.clone());
        for (k, v) in self.inputs.iter() {
            build.add_content(k, v.clone());
        }

        Ok(serde_wasm_bindgen::to_value(
            &build.compile().into_jserr()?.as_strings().into_jserr()?,
        )?)
    }
}

trait IntoJsError<T> {
    fn into_jserr(self) -> Result<T, wasm_bindgen::JsError>;
}

impl<T> IntoJsError<T> for Result<T, anyhow::Error> {
    fn into_jserr(self) -> Result<T, wasm_bindgen::JsError> {
        self.map_err(|x| JsError::from(x.root_cause()))
    }
}
