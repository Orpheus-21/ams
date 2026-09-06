//! Task 3: embedding the `typst` crate and a minimal `World`.

use ams_lib::compile::{compile, AmsWorld, CompileSession};

/// `#import "@preview/..."` used to be rejected outright by the file loader,
/// which quietly disabled the entire Typst template ecosystem. This assertion
/// doesn't need the network: offline the resolver reports a download failure,
/// online a missing package — either way it must no longer be the old flat
/// refusal.
#[test]
fn package_imports_reach_the_resolver_instead_of_being_refused() {
    let mut session = CompileSession::detached(String::new());
    let message = match session.recompile("#import \"@preview/this-package-does-not-exist:9.9.9\": *") {
        Ok(_) => panic!("a nonexistent package cannot compile"),
        Err(err) => err.to_string(),
    };
    assert!(
        !message.contains("not supported"),
        "package imports must be resolved, not refused outright; got: {message}"
    );
}

/// Fetches a real package from Typst Universe, so it's ignored by default and
/// CI stays hermetic. Run with `cargo test -- --ignored` to exercise the real
/// download-and-cache path.
#[test]
#[ignore = "hits the network"]
fn a_real_package_downloads_and_compiles() {
    let mut session = CompileSession::detached(String::new());
    let output = session
        .recompile("#import \"@preview/a2c-nums:0.0.1\": *\n\nHello.")
        .expect("a real package should download and compile");
    assert_eq!(output.document.pages().len(), 1);
}

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
