//! Keeps one [`AmsWorld`] alive across a document's repeated recompiles.
//!
//! This is what actually makes incremental recompilation possible: Typst's
//! memoization (`comemo`) is keyed off the identity of the values a `World`
//! returns, so a *new* `World` per keystroke would look like a brand-new
//! document every time. `CompileSession` is the boundary that keeps the same
//! `World` around and mutates it in place instead.

use std::path::PathBuf;

use typst::syntax::DiagSpan;

use typst_layout::PagedDocument;

use super::world::AmsWorld;
use super::{compile, completions_at, CompileError, CompileOutput, Completions};

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

    /// Completions at `cursor` (a UTF-16 offset) for `text`.
    ///
    /// Updates the world's text without compiling: completions must reflect
    /// what the user has typed this instant, and waiting for a compile would
    /// make the popup lag behind the keyboard.
    pub fn completions(
        &mut self,
        text: &str,
        cursor: usize,
        document: Option<&PagedDocument>,
    ) -> Completions {
        self.world.set_text(text);
        completions_at(&self.world, document, cursor)
    }

    /// See [`crate::compile::jump_from_preview`].
    pub fn jump_from_preview(
        &self,
        document: &PagedDocument,
        page: usize,
        x_pt: f64,
        y_pt: f64,
    ) -> Option<usize> {
        super::jump_from_preview(&self.world, document, page, x_pt, y_pt)
    }

    /// See [`crate::compile::preview_position_of_cursor`].
    pub fn preview_position_of_cursor(
        &self,
        document: &PagedDocument,
        cursor: usize,
    ) -> Option<(usize, f64)> {
        super::preview_position_of_cursor(&self.world, document, cursor)
    }

    /// 1-based line and column of a diagnostic in the current text. See
    /// [`AmsWorld::location_of`].
    pub fn location_of(&self, span: DiagSpan) -> Option<(usize, usize)> {
        self.world.location_of(span)
    }
}
