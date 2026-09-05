//! Task 3: embedding the `typst` crate and a minimal `World`.

use ams_lib::compile::{compile, AmsWorld};

#[test]
fn compiles_a_trivial_document() {
    let world = AmsWorld::detached("Hello, world!");

    let output = compile(&world).expect("a trivial document should compile");

    assert_eq!(output.document.pages().len(), 1);
    assert!(output.warnings.is_empty());
}

#[test]
fn compile_errors_surface_as_a_result_not_a_panic() {
    let world = AmsWorld::detached("#set text(size: )");

    let result = compile(&world);

    assert!(result.is_err(), "malformed markup should be a compile error, not a panic");
}

#[test]
fn a_page_break_produces_multiple_pages() {
    let world = AmsWorld::detached("Page one.\n#pagebreak()\nPage two.");

    let output = compile(&world).expect("should compile");

    assert_eq!(output.document.pages().len(), 2);
}
