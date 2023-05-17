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

use crate::render::RenderCss;

/// A wrapper around [`Vec`] which guarantees at least `N` elements.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MinVec<T, const N: usize>([T; N], Vec<T>);

impl<T, const N: usize> MinVec<T, N>
where
    T: std::fmt::Debug,
{
    /// Create a new N-element-guaranteed collection.
    pub fn new(head: [T; N], tail: Vec<T>) -> Self {
        MinVec(head, tail)
    }

    /// Iterate over the values in this collection.
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> {
        self.0.iter().chain(self.1.iter())
    }

    /// Iterate over the values in this collection.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut T> {
        self.0.iter_mut().chain(self.1.iter_mut())
    }
}

impl<T: RenderCss, const N: usize> RenderCss for MinVec<T, N> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in self.0.iter() {
            x.render(f)?;
        }

        for x in self.1.iter() {
            write!(f, ",")?;
            x.render(f)?;
        }

        Ok(())
    }
}

/// Givens a root path `outdir` and a relative path `path`, remove the extension
/// to the latter and join with the former.  If the latter path is not relative,
/// return the former.  Useful for moving directory trees while retaining their
/// relative structure to some root.
pub fn join_paths(outdir: &Path, path: &Path) -> PathBuf {
    if let Some(parent) = path.parent() {
        PathBuf::from(outdir).join(parent)
    } else {
        PathBuf::from(outdir)
    }
}

#[cfg(feature = "iotest")]
mod mock {
    #[mockall::automock]
    pub trait IoTestFs {
        fn read_to_string(path: &std::path::Path) -> std::io::Result<String>;
        // where
        //     P: AsRef<std::path::Path> + 'static;

        fn read(path: &std::path::Path) -> std::io::Result<Vec<u8>>;

        fn create_dir_all<P>(path: P) -> std::io::Result<()>
        where
            P: AsRef<std::path::Path> + 'static;

        fn write<P, C>(path: P, content: C) -> std::io::Result<()>
        where
            P: AsRef<std::path::Path> + 'static,
            C: AsRef<[u8]> + 'static;
    }
}

#[cfg(not(feature = "iotest"))]
pub use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "iotest")]
pub use mock::{IoTestFs, MockIoTestFs as fs};
