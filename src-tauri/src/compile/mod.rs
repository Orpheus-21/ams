//! The in-process Typst compile engine: embeds the `typst` crate directly
//! (never shells out to a CLI) so its own memoization carries incremental
//! recompilation across edits. See `SPEC.md`'s Tech Stack section.

mod fonts;
mod world;

pub use world::AmsWorld;

use typst::diag::{SourceDiagnostic, Warned};
use typst::ecow::EcoVec;
use typst_layout::PagedDocument;

/// A successful compile: the laid-out pages plus any non-fatal warnings.
pub struct CompileOutput {
    pub document: PagedDocument,
    pub warnings: EcoVec<SourceDiagnostic>,
}

/// One or more fatal Typst diagnostics from a failed compile.
#[derive(Debug, Clone)]
pub struct CompileError(pub EcoVec<SourceDiagnostic>);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, diag) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", diag.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for CompileError {}

/// Compiles `world`'s current main document to a [`PagedDocument`].
///
/// Never panics on a malformed document — compile errors surface as
/// [`CompileError`].
pub fn compile(world: &AmsWorld) -> Result<CompileOutput, CompileError> {
    let Warned { output, warnings } = typst::compile::<PagedDocument>(world);
    match output {
        Ok(document) => Ok(CompileOutput { document, warnings }),
        Err(errors) => Err(CompileError(errors)),
    }
}
