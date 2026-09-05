//! Keeps one [`AmsWorld`] alive across a document's repeated recompiles.
//!
//! This is what actually makes incremental recompilation possible: Typst's
//! memoization (`comemo`) is keyed off the identity of the values a `World`
//! returns, so a *new* `World` per keystroke would look like a brand-new
//! document every time. `CompileSession` is the boundary that keeps the same
//! `World` around and mutates it in place instead.

use std::path::PathBuf;

use super::world::AmsWorld;
use super::{compile, CompileError, CompileOutput};

/// A document's compile session: owns the [`AmsWorld`] for its lifetime.
pub struct CompileSession {
    world: AmsWorld,
}

impl CompileSession {
    /// Starts a session for a document on disk at `root`/`file_name`.
    pub fn new(root: PathBuf, file_name: &str, text: impl Into<String>) -> Self {
        Self { world: AmsWorld::new(root, file_name, text) }
    }

    /// Starts a session with no filesystem backing (e.g. for tests).
    pub fn detached(text: impl Into<String>) -> Self {
        Self { world: AmsWorld::detached(text) }
    }

    /// Replaces the document's text and recompiles, reusing the same
    /// [`AmsWorld`] (and thus Typst's memoized compilation state) from the
    /// previous call.
    pub fn recompile(&mut self, text: &str) -> Result<CompileOutput, CompileError> {
        self.world.begin_compile();
        self.world.set_text(text);
        compile(&self.world)
    }
}
