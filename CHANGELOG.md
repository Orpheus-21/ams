# changelog

## v0.1.0 — unreleased

First build worth handing to someone.

- Typst markup editor with a live preview that updates ~120ms after you stop typing.
  Compilation runs in-process against the `typst` crate and off the UI thread; a small edit in
  a 500-page document recompiles in ~48ms, guarded by a test.
- Preview renders one page at a time, only for pages on screen, fitted to the pane.
- Click a page to jump to the source that produced it; the preview follows the cursor back.
- Completions from the compiler itself — `#` offers real functions with descriptions.
- Packages work: `#import "@preview/..."` resolves from the same places the Typst CLI looks.
- Native menu bar with shortcuts shown against each item, `F1` for the full list.
- Errors show the line number and Typst's own hints; the last good render stays on screen.
- Unsaved changes are marked in the title and confirmed before being discarded.
- Bundled fonts, so a fresh install renders correctly with no system fonts and no network.
- Windows `.msi` and `.exe`, Linux AppImage, built in CI.

Known limits: the installer is unsigned, so Windows warns before running it. Syntax
highlighting switches off above 2000 lines because the available Typst grammar reparses the
whole document on every keystroke. Packages need internet the first time they're used.
