//! Task 6: sibling-path resolution for images/`.bib` files.
//!
//! A document can reference a file (image, bibliography, ...) sitting next
//! to it on disk via a relative path, resolved relative to the document's
//! own directory — matching the Typst CLI's own behavior.

use std::path::PathBuf;

use ams_lib::compile::{compile, AmsWorld};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sibling_paths")
}

#[test]
fn resolves_a_sibling_image() {
    let world = AmsWorld::new(fixtures_dir(), "doc.typ", r#"#image("photo.png")"#);

    let result = compile(&world);

    assert!(result.is_ok(), "sibling image should resolve: {:?}", result.err());
}

#[test]
fn resolves_a_sibling_bibliography() {
    let world = AmsWorld::new(
        fixtures_dir(),
        "doc.typ",
        "Typst is fast @typst2023.\n#bibliography(\"refs.bib\")",
    );

    let result = compile(&world);

    assert!(result.is_ok(), "sibling bibliography should resolve: {:?}", result.err());
}

#[test]
fn a_missing_sibling_file_is_a_compile_error_not_a_panic() {
    let world = AmsWorld::new(fixtures_dir(), "doc.typ", r#"#image("does-not-exist.png")"#);

    let result = compile(&world);

    assert!(result.is_err(), "a missing sibling file should be a compile error");
}
