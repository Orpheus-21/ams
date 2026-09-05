//! Task 5: incremental recompilation via cache reuse.
//!
//! Reusing the same `AmsWorld`/`CompileSession` across compiles lets Typst's
//! own memoization (`comemo`) skip re-evaluating unchanged parts of the
//! document, so a small edit recompiles much faster than a cold compile of
//! the same document from scratch.

use std::time::Instant;

use ams_lib::compile::CompileSession;

/// A synthetic multi-page document: `pages` headed sections, each with a
/// paragraph of generated lorem-ipsum text.
fn synthetic_doc(pages: usize) -> String {
    let mut source = String::new();
    for i in 0..pages {
        source.push_str(&format!("= Section {i}\n#lorem(120)\n#pagebreak()\n"));
    }
    source
}

#[test]
fn edit_recompile_is_faster_than_cold_compile() {
    let pages = 150;
    let cold_text = synthetic_doc(pages);

    // Cold compile: brand-new session, nothing cached yet.
    let mut session = CompileSession::detached(cold_text.clone());
    let cold_start = Instant::now();
    session.recompile(&cold_text).expect("cold compile should succeed");
    let cold_elapsed = cold_start.elapsed();

    // Small edit near the start of the document, same session reused so
    // Typst's memoization carries over.
    let edited_text = cold_text.replacen("Section 0", "Section 0 (edited)", 1);
    let edit_start = Instant::now();
    session.recompile(&edited_text).expect("recompile after edit should succeed");
    let edit_elapsed = edit_start.elapsed();

    eprintln!("cold compile: {cold_elapsed:?}, edit recompile: {edit_elapsed:?}");
    assert!(
        edit_elapsed < cold_elapsed,
        "recompile after a small edit ({edit_elapsed:?}) should be faster than \
         the cold compile ({cold_elapsed:?}) thanks to cache reuse"
    );
}
