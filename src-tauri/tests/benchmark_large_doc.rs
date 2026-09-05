//! Task 7: the headline non-functional requirement from `SPEC.md` — editing
//! anywhere in a ~500-page document must recompile in under 100ms. This is
//! the regression guard for "must not hang."

use std::time::{Duration, Instant};

use ams_lib::compile::CompileSession;

const PAGES: usize = 500;
const RECOMPILE_BUDGET: Duration = Duration::from_millis(100);

/// A synthetic ~500-page document: one heading + a paragraph of generated
/// lorem-ipsum text per page.
fn large_doc() -> String {
    let mut source = String::new();
    for i in 0..PAGES {
        source.push_str(&format!("= Section {i}\n#lorem(120)\n"));
        if i + 1 < PAGES {
            source.push_str("#pagebreak()\n");
        }
    }
    source
}

#[test]
fn small_edit_recompiles_within_budget() {
    let text = large_doc();

    let mut session = CompileSession::detached(text.clone());
    session.recompile(&text).expect("cold compile of the large document should succeed");

    // A small, localized edit near the start of the document — the kind a
    // real keystroke produces.
    let edited = text.replacen("Section 0", "Section 0 (edited)", 1);

    let start = Instant::now();
    let output = session.recompile(&edited).expect("recompile after a small edit");
    let elapsed = start.elapsed();

    eprintln!("recompile of a {PAGES}-page document after a small edit: {elapsed:?}");
    assert_eq!(output.document.pages().len(), PAGES);
    assert!(
        elapsed < RECOMPILE_BUDGET,
        "recompile after a small edit took {elapsed:?}, budget is {RECOMPILE_BUDGET:?}"
    );
}
