# Spec: ams (v1)

Status: APPROVED (v1 scope). Open questions resolved 2026-09-05.
Intent source: `docs/intent/ams.md`.

## Objective

ams is a native, fast, keyboard-first desktop app for writing Typst documents. It pairs a
plain-markup source editor with an instantly live-rendered preview, so that people currently
writing everyday documents in Word — starting with the founder and friends, on Windows and Linux
— get Typst's speed and typographic control without needing a terminal, VS Code, or any
configuration. Power users are served by Typst's own package/template ecosystem, not by bespoke
app features.

v1 success: a real document is written start-to-finish using only ams, the Windows installer is
handed to a friend cold, and they're productive without help.

## Tech Stack

- **Core:** Rust. The `typst` crate is embedded in-process (not shelled out to as a CLI
  subprocess), so Typst's own memoization (`comemo`) carries incremental recompilation across
  edits instead of recompiling the whole document every keystroke.
- **App shell:** Tauri 2 — Rust backend, OS-native webview (WebView2 on Windows, WebKitGTK on
  Linux). No bundled Chromium.
- **Frontend:** vanilla TypeScript + Vite. No UI framework (React/Svelte/Vue) — the surface is
  small enough (editor pane, preview pane, a handful of keyboard-driven actions) that a framework
  is unearned weight. *(Assumption — flagged for review.)*
- **Editor pane:** CodeMirror 6, with a Typst language mode (syntax highlighting via a Lezer
  grammar — reuse a community grammar if one exists at implementation time, otherwise write a
  minimal one). Chosen over a hand-rolled editor because CM6 already solves large-document
  virtualized rendering and incremental highlighting — the exact problem the "no hang at 500
  pages" requirement is about.
- **Preview pane:** Typst's per-page render output drawn to canvas, virtualized to only rasterize
  pages currently in the viewport.
- **Packaging:** Tauri's built-in bundler — `.msi`/NSIS `.exe` on Windows, **AppImage only** on
  Linux (not `.deb` — one packaging target for v1, deb can follow later if needed). No custom
  installer tooling. *(This is what makes "simple installer from v1" cheap — it's already built
  into the framework we're using.)*
- **Fonts:** a font set is bundled with the app so a fresh install renders correctly with no
  system fonts required and no network access. Exact font selection (open-license, redistributable,
  good defaults for a Word-alternative audience) is a Plan/Tasks-phase decision, not a spec-level
  one — flagging as a task, not deciding the specific typefaces here.
- **Package registry access (`#import "@preview/..."`):** v1 requires internet access the first
  time a document references a not-yet-cached package — same behavior as Typst's own CLI, which
  caches fetched packages to disk after first use, so a document that has already pulled its
  packages once continues to compile offline afterward. The core edit/compile/preview loop for
  documents that reference no remote package is offline from the start; it's only registry fetches
  that need a network. **Roadmap, not v1:** ams is offline-first in direction — a later version
  should let the app work out of the box without requiring that first-fetch network hit (e.g. by
  bundling a common package cache or a first-run download bundle). Recording this now so it isn't
  lost, but it's explicitly deferred past v1.

## Commands

Assumes a standard Tauri 2 + Vite project layout (confirmed at Phase 2/Plan, actual scripts land
in `package.json` / `Cargo.toml` when scaffolded):

```
Dev:     pnpm tauri dev
Build:   pnpm tauri build            # produces platform installer + binary
Test:    cargo test --workspace      # Rust unit + benchmark-style tests
Lint:    cargo clippy --workspace -- -D warnings
Format:  cargo fmt --all  &&  pnpm exec prettier --write "src/**/*.ts"
Typecheck: pnpm exec tsc --noEmit
```

## Project Structure

```
src-tauri/           → Rust backend: Tauri commands, Typst compilation engine, World impl
src-tauri/src/compile/  → in-process typst crate integration, incremental cache, path resolution
src-tauri/tests/     → cargo tests, incl. large-document recompile-latency benchmark test
src/                 → frontend TypeScript: editor pane, preview pane, keyboard command wiring
src/editor/          → CodeMirror 6 setup, Typst language mode
src/preview/         → canvas rendering, viewport-based page virtualization
docs/                → intent docs, spec, ADRs
docs/intent/          → confirmed-intent documents (interview-me output)
tasks/                → plan.md and task list (planning-and-task-breakdown output)
```

## Code Style

Rust: standard `rustfmt` defaults, `clippy` clean (`-D warnings`). No unwrap()/expect() outside
tests and truly-unreachable states — file I/O and compilation errors are user-facing, not crashes.

```rust
// Good: user-facing error surfaces to the UI, doesn't panic
fn compile(world: &impl World) -> Result<PagedDocument, CompileError> {
    typst::compile(world).output.map_err(CompileError::from)
}
```

TypeScript: Prettier defaults, strict `tsconfig` (`strict: true`). No `any` without a comment
explaining why. Small, direct DOM/event wiring over introducing a state-management library —
matches the "no framework" call above; revisit only if state complexity actually grows past what
a plain module can hold.

## Testing Strategy

- **Rust unit tests** (`src-tauri/tests`, `cargo test`) for: incremental-cache correctness (an
  edit outside a page doesn't force a full recompile of unrelated pages), sibling-path resolution
  (images/`.bib` relative to the document), and compile error surfacing.
- **One benchmark-style test** asserting the headline non-functional requirement directly: given
  a synthetic ~500-page document, a small single-edit recompile completes under a **100ms**
  budget. This is the regression guard for "must not hang."
- **Frontend:** `tsc --noEmit` as a type-correctness gate. No automated e2e for v1 (per
  assumption above) — a manual keyboard-only smoke checklist covers open/edit/save/preview/export
  before each release, promoted to automated e2e later if manual QA becomes the bottleneck.
- No coverage percentage target for v1 — this is a small, focused codebase; coverage-by-number
  would be a vanity metric here. Revisit if/when the codebase grows past what one person can
  reason about directly.

## Boundaries

- **Always:** run `cargo test --workspace` and `cargo clippy` before any commit touching
  `src-tauri/`; run `tsc --noEmit` before any commit touching `src/`; keep compilation off the UI
  thread; keep every new interactive action reachable by keyboard.
- **Ask first:** adding any new runtime dependency beyond what's named in this spec (Tauri,
  typst, CodeMirror 6, Vite); changing the packaging/installer approach; anything that would
  reintroduce Electron, a visual/WYSIWYG editor, `.docx` import, or an in-app file browser — all
  explicitly out of scope per the confirmed intent; macOS support.
- **Never:** commit signing keys/secrets; ship a release where the large-document benchmark test
  is failing or skipped; add a feature not in this spec or a later-approved spec revision.

## Success Criteria

- A real document (e.g. an essay or report) is written start-to-finish inside ams, using only
  markup + preview, no other tool.
- The Windows build is a `.msi`/`.exe` installer produced by `pnpm tauri build`, installable by a
  non-technical friend with no manual setup steps.
- Editing anywhere in a ~500-page synthetic document keeps the UI thread unblocked and the
  benchmark test's recompile-latency budget passes.
- Every action in the app (open, save, new, undo/redo, navigate editor/preview) is reachable via
  keyboard alone.
- `cargo test --workspace`, `cargo clippy -- -D warnings`, and `tsc --noEmit` all pass in CI (CI
  setup itself is a Phase-2/Plan task, not decided here).

## Open Questions

None outstanding for v1. Resolved 2026-09-05:

1. Command palette: fixed shortcuts only for v1; fuzzy command-palette search is a later version.
2. Recompile-latency budget: 100ms, confirmed.
3. Linux packaging: AppImage only for v1.
4. Fonts: bundled, confirmed — specific font selection deferred to Plan/Tasks.
5. Package registry: v1 needs network on first fetch of an uncached package (matches Typst's own
   caching behavior); working out-of-the-box offline is an explicit post-v1 roadmap item.

## Non-Goals (v1)

Restated from confirmed intent for traceability: no visual/WYSIWYG editing or markup-inserting
toolbar, no `.docx` import, no in-app file browser/project sidebar, no bespoke
lawyer/publishing-house features (citation UI, prepress/bleed/CMYK/imposition), no macOS build.
