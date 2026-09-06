# Implementation Plan: ams v1

Source spec: `../SPEC.md`. Task list target: `tasks/todo.md` (default markdown checklist — no
external tracker designated).

## Overview

ams v1 is a Tauri 2 desktop app: a Rust core that embeds the `typst` crate in-process for
compilation, a CodeMirror 6 source editor pane, and a canvas-based live preview pane. The plan
below is ordered so the two highest-risk, hardest-to-fake things get proven first — cross-platform
packaging, and the "must not hang at 500 pages" performance requirement — before investing in the
full editor/preview UI.

## Dependency Graph

```
Project scaffold (Tauri + Vite + TS, empty window)
    │
    ├── CI cross-platform packaging (Windows .msi/.exe, Linux AppImage)  [risk: validate early]
    │
    └── Compile engine (typst crate embedded, World impl, fonts, incremental cache)
            │
            ├── Sibling-path resolution (images/.bib)
            ├── Large-document benchmark test (100ms budget)
            │
            └── Tauri commands (new/open/save, invoke compile)
                    │
                    ├── Editor pane (CodeMirror 6 + Typst grammar)
                    │       │
                    └── Preview pane (canvas render, viewport virtualization)
                            │
                            └── Full edit → compile → preview loop wired end-to-end
                                    │
                                    ├── Keyboard-first pass (shortcuts, focus mgmt)
                                    │
                                    └── Release packaging finalization + install verification
```

Implementation order follows this graph bottom-up, but Task 2 (CI packaging proof) is pulled
forward ahead of the compile engine — see Architecture Decisions.

## Architecture Decisions

- **Packaging risk goes first, not last.** Building a Windows `.msi`/`.exe` from a Linux dev
  machine means the real build has to happen on a Windows CI runner. If that pipeline is broken,
  it's better to find out in an hour on an empty scaffold than after weeks of feature work. Task 2
  is "prove an empty Tauri app installs on both target platforms via CI," before any real feature
  lands.
- **Vertical slice for the core loop, not horizontal layers.** Phase 3 builds "user can open, edit,
  see live preview, and save a document" as one connected slice (Tasks 8-11) rather than
  "finish all of the editor, then all of the preview, then wire them" — matches this skill's
  vertical-slicing guidance directly.
- **Benchmark test lands with the compile engine, not at the end.** The 100ms recompile budget is
  the spec's headline non-functional requirement. Task 7 writes it as an automated `cargo test`
  as soon as the compile engine exists, so every later task runs against a live regression guard
  instead of hoping performance holds up by the time someone thinks to check.

## Task List

### Phase 1: Foundation & Packaging Risk

- [x] Task 1: Scaffold Tauri 2 + Vite + TypeScript project
- [x] Task 2: CI cross-platform packaging proof (Windows + Linux)

### Checkpoint: Foundation
- [x] Builds via `pnpm tauri build` locally (binary + `.deb`/`.rpm`; local AppImage bundling
      fails on Arch — linuxdeploy-plugin-gtk expects a Debian-style gdk-pixbuf layout — which is
      exactly why the authoritative Linux build is CI's Ubuntu runner)
- [x] CI produces a Windows `.msi` + NSIS `.exe` and a Linux AppImage as build artifacts,
      verified by downloading and inspecting them
- [ ] Both artifacts install/launch on a clean machine — deferred to Task 14
- [ ] Review with human before proceeding

### Phase 2: Compile Engine (Rust core)

- [x] Task 3: Embed `typst` crate + minimal `World` implementation
- [x] Task 4: Bundle default font set
- [x] Task 5: Incremental recompilation via cache reuse
- [x] Task 6: Sibling-path resolution for images/`.bib` files
- [x] Task 7: Large-document recompile-latency benchmark test (100ms budget)

### Checkpoint: Compile Engine
- [x] `cargo test --workspace` passes, including the benchmark test
- [x] `cargo clippy --workspace -- -D warnings` clean
- [x] Compile engine is exercised end-to-end via tests without any UI existing yet
- [ ] Review with human before proceeding

### Phase 3: Core Loop — Editor + Preview + File I/O (vertical slice)

- [x] Task 8: Tauri commands for file I/O + compile invocation
- [x] Task 9: Editor pane (CodeMirror 6 + Typst syntax highlighting)
- [x] Task 10: Preview pane (virtualized canvas rendering)
- [x] Task 11: Wire the full edit → compile → preview loop end-to-end

### Checkpoint: Core Loop
- [x] Manual smoke test: verified by the founder on his own Windows laptop, installed from the
      CI-built NSIS installer. Typing produces an immediate preview update — the app's core
      thesis is proven on the real target platform, not just in tests.
- [x] Compilation confirmed off the UI thread (typing stayed responsive)
- [x] Review with human before proceeding

**Finding from that session — UI affordances gap (v1 blocker, not polish).** The verdict was
"clunky as hell, nothing much in terms of UI". That is currently true in a way that directly
threatens SPEC.md's success criterion: *"hand the Windows installer to a friend cold, and they're
productive without you sitting next to them."* Right now the app has no menu bar, no visible
controls of any kind, and the only way to open or save a document is to already know that
Ctrl+N/O/S exist. A non-technical friend — the exact target user — would open ams, see two panes,
and have no way to discover how to do anything. Task 12 as written (shortcut coverage + focus
order, shortcut list "documented somewhere discoverable, e.g. README") does not close this: a
README does not help someone who has already double-clicked the app. See Task 15.

### Phase 4: Keyboard-First Pass + Minimum Usable UI

- [ ] Task 12: Full keyboard shortcut coverage + focus management
- [ ] Task 15: Minimum discoverable UI (menu bar, window title, unsaved indicator)

### Checkpoint: Keyboard-First
- [ ] Manual keyboard-only walkthrough: new, open, edit, save, undo/redo, switch editor↔preview
  focus — no mouse touched, nothing unreachable
- [ ] Someone who has never seen ams can open, edit, and save a document without being told any
  shortcut (the real test of SPEC.md's success criterion)
- [ ] Review with human before proceeding

### Phase 5: Release Readiness

- [ ] Task 13: Finalize bundler config (icons, metadata, versioning, WebView2 bootstrapper)
- [ ] Task 14: Clean-machine install verification (Windows + Linux)

### Checkpoint: v1 Complete
- [ ] All SPEC.md Success Criteria verified
- [ ] `cargo test --workspace`, `cargo clippy -- -D warnings`, `tsc --noEmit` pass in CI
- [ ] Ready for the founder + friends beta

---

## Task Detail

## Task 1: Scaffold Tauri 2 + Vite + TypeScript project

**Description:** Initialize the Tauri 2 project with a Vite + vanilla TypeScript frontend, matching
the `src-tauri/` / `src/` layout from SPEC.md. Empty window, no real functionality yet.

**Acceptance criteria:**
- [ ] `pnpm tauri dev` opens an empty window on Linux
- [ ] `pnpm tauri build` produces a local build without errors
- [ ] Directory layout matches SPEC.md's Project Structure section

**Verification:**
- [ ] Build succeeds: `pnpm tauri build`
- [ ] Manual check: window opens via `pnpm tauri dev`

**Dependencies:** None

**Files likely touched:** `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`,
`package.json`, `vite.config.ts`, `src/main.ts`, `index.html`

**Estimated scope:** M

---

## Task 2: CI cross-platform packaging proof

**Description:** Set up CI (GitHub Actions, matching the GitHub-hosted repo) that builds the
scaffold on a `windows-latest` runner (producing `.msi`/`.exe`) and a Linux runner (producing
AppImage), and uploads both as build artifacts. This is a pure risk-reduction task — it proves the
"simple installer from v1" requirement is achievable before any feature work depends on it.

**Acceptance criteria:**
- [ ] CI workflow builds the empty scaffold on push
- [ ] Windows job produces a `.msi` or NSIS `.exe` artifact
- [ ] Linux job produces an AppImage artifact
- [ ] Both artifacts are downloadable and installable/runnable

**Verification:**
- [ ] Build succeeds: CI workflow run is green
- [ ] Manual check: download each artifact, install/run it on the respective OS

**Dependencies:** Task 1

**Files likely touched:** `.github/workflows/build.yml`

**Estimated scope:** S

---

## Task 3: Embed `typst` crate + minimal `World` implementation

**Description:** Add the `typst` crate to `src-tauri`, implement the minimal `typst::World` trait
(in-memory source for a single document, file resolution for local paths) needed to call
`typst::compile()` and get back a rendered document, entirely in-process — no CLI subprocess.

**Acceptance criteria:**
- [ ] A Rust function compiles a Typst source string to a `PagedDocument` (or current equivalent)
      and returns per-page output
- [ ] Compile errors surface as a typed `Result`, never a panic
- [ ] Unit test compiles a trivial document and asserts page count / basic content

**Verification:**
- [ ] Tests pass: `cargo test --workspace`
- [ ] Manual check: none (headless, test-covered)

**Dependencies:** Task 1

**Files likely touched:** `src-tauri/Cargo.toml`, `src-tauri/src/compile/mod.rs`,
`src-tauri/src/compile/world.rs`, `src-tauri/tests/compile.rs`

**Estimated scope:** M

---

## Task 4: Bundle default font set

**Description:** Select an open-licensed (redistributable) font set and embed it into the compile
engine's font book, so documents render correctly with zero system fonts and zero network access.
Font selection itself (which specific typefaces) happens here, per SPEC.md's deferral.

**Acceptance criteria:**
- [ ] Chosen fonts have a license permitting redistribution (e.g. OFL); license noted in a
      `THIRD_PARTY_LICENSES` file or equivalent
- [ ] A document using default (unspecified) fonts renders correctly with no system fonts present
- [ ] Unit test asserts the font book resolves a default text font and a default math font

**Verification:**
- [ ] Tests pass: `cargo test --workspace`
- [ ] Manual check: render a doc in a sandboxed/clean environment with no system fonts installed

**Dependencies:** Task 3

**Files likely touched:** `src-tauri/src/compile/fonts.rs`, `src-tauri/assets/fonts/`,
`src-tauri/tests/compile.rs`

**Estimated scope:** S

---

## Task 5: Incremental recompilation via cache reuse

**Description:** Ensure the `World` implementation persists across repeated compiles of the same
document (not reconstructed per keystroke), so Typst's own memoization (`comemo`) can skip
re-evaluating unchanged parts of the document on each edit.

**Acceptance criteria:**
- [ ] The same `World`/compile-session object is reused across sequential compile calls for one
      document
- [ ] Unit test demonstrates a measurable recompile-time drop on a small edit vs. a cold compile
      of the same large document

**Verification:**
- [ ] Tests pass: `cargo test --workspace`

**Dependencies:** Task 3

**Files likely touched:** `src-tauri/src/compile/session.rs`, `src-tauri/tests/compile.rs`

**Estimated scope:** S

---

## Task 6: Sibling-path resolution for images/`.bib` files

**Description:** Implement path resolution so a document can reference images or a
`bibliography("refs.bib")` file sitting next to it on disk, resolved relative to the document's
directory — matching how the Typst CLI already behaves.

**Acceptance criteria:**
- [ ] A document referencing a relative image path compiles successfully when that file exists
      next to the `.typ` file
- [ ] A document using `#bibliography("refs.bib")` compiles successfully with a sibling `.bib` file
- [ ] A missing sibling file produces a clear compile error, not a panic

**Verification:**
- [ ] Tests pass: `cargo test --workspace`

**Dependencies:** Task 3

**Files likely touched:** `src-tauri/src/compile/world.rs`, `src-tauri/tests/compile.rs`,
`src-tauri/tests/fixtures/`

**Estimated scope:** S

---

## Task 7: Large-document recompile-latency benchmark test

**Description:** Write the regression guard named directly in SPEC.md: generate (or check in) a
synthetic ~500-page Typst document, perform a cold compile, make one small edit, and assert the
recompile completes within 100ms.

**Acceptance criteria:**
- [ ] A synthetic ~500-page fixture document exists (generated by a small script/build step, or
      checked in if generation is impractical)
- [ ] Test performs cold compile, then one small edit + recompile, asserting recompile time < 100ms
- [ ] Test runs as part of `cargo test --workspace` (not `--ignored`, unless CI runner timing
      makes that noisy — decide during implementation and note the reasoning if so)

**Verification:**
- [ ] Tests pass: `cargo test --workspace`

**Dependencies:** Task 5

**Files likely touched:** `src-tauri/tests/benchmark_large_doc.rs`, `src-tauri/tests/fixtures/`

**Estimated scope:** S

---

## Task 8: Tauri commands for file I/O + compile invocation

**Description:** Expose Tauri commands the frontend calls: `new_document`, `open_document`,
`save_document`, `compile_document`. Compilation runs on a background thread/async task — the
Tauri command handler never blocks the main/UI-relevant thread.

**Acceptance criteria:**
- [ ] Frontend can invoke new/open/save via Tauri's `invoke()` and get correct results
- [ ] `compile_document` runs off the main thread and returns compiled output or a structured error
- [ ] Unit/integration test covers each command's success and error paths

**Verification:**
- [ ] Tests pass: `cargo test --workspace`

**Dependencies:** Task 3, Task 6

**Files likely touched:** `src-tauri/src/commands.rs`, `src-tauri/src/main.rs`,
`src-tauri/tests/commands.rs`

**Estimated scope:** M

---

## Task 9: Editor pane (CodeMirror 6 + Typst syntax highlighting)

**Description:** Integrate CodeMirror 6 into the frontend as the source editor pane. Add Typst
syntax highlighting via a Lezer grammar — check for an existing community grammar first; write a
minimal one only if none is adequate (ladder rung 2 before rung 7).

**Acceptance criteria:**
- [x] CodeMirror 6 renders in the app, editable, with Typst-aware syntax highlighting
- [x] Editor handles a large (~500 page) document without input lag (CM6's own virtualization
      should cover this — verify it does)
- [x] Editor content is retrievable by the rest of the app (for compile + save)

**Verification:**
- [x] Build succeeds: `pnpm tauri build`
- [x] Manual check: open a large fixture doc, scroll and type, confirm no lag

**Dependencies:** Task 1

**Files likely touched:** `src/editor/setup.ts`, `src/editor/typst-lang.ts`, `src/main.ts`

**Estimated scope:** M

**Outcome note:** used the community `codemirror-lang-typst` package rather than writing a
grammar. Discovered its parser has no real incremental reparse (full reparse per edit), which at
large-document size blocked the main thread for ~300-450ms per keystroke — a direct violation of
the "must not hang" requirement. Mitigated with a line-count-gated `Compartment` that drops syntax
highlighting above 2000 lines (marked `ponytail:` in `src/editor/setup.ts` with the upgrade path:
real incremental reparsing upstream, or a proper `@lezer/lr` grammar). Net effect: very large
documents (a 500-page doc will likely cross 2000 lines) edit without lag but lose highlighting —
a real v1 product tradeoff, not just an implementation detail.

---

## Task 10: Preview pane (virtualized canvas rendering)

**Description:** Render compiled Typst output to canvas, one page at a time, only rasterizing pages
currently in the preview viewport (not the whole document on every compile).

**Acceptance criteria:**
- [ ] Preview pane displays compiled page(s) correctly for a simple document
- [ ] Scrolling the preview only triggers rendering of newly-visible pages, not the full document
- [ ] A ~500-page compiled document does not rasterize all 500 pages on load

**Verification:**
- [ ] Build succeeds: `pnpm tauri build`
- [ ] Manual check: open large fixture doc, scroll preview, confirm smooth scroll with no
      full-document re-render

**Dependencies:** Task 8

**Files likely touched:** `src/preview/render.ts`, `src/preview/viewport.ts`, `src/main.ts`

**Estimated scope:** M

---

## Task 11: Wire the full edit → compile → preview loop end-to-end

**Description:** Connect editor changes to a debounced compile call to the backend, and update the
preview pane with the result, completing the vertical slice.

**Acceptance criteria:**
- [ ] Typing in the editor updates the preview within a perceptibly-instant window (debounced, not
      per-keystroke)
- [ ] Typing during an in-flight large-document compile is never blocked or dropped
- [ ] Compile errors surface visibly in the UI (not silently swallowed)

**Verification:**
- [ ] Manual check: full golden path — new doc, type, see preview update, introduce an error, see
      it surface, fix it, save, reopen

**Dependencies:** Task 9, Task 10

**Files likely touched:** `src/main.ts`, `src/editor/setup.ts`, `src/preview/render.ts`

**Estimated scope:** S

---

## Task 12: Full keyboard shortcut coverage + focus management

**Description:** Ensure every interactive action (new, open, save, undo/redo, find, switch focus
between editor and preview) has a keyboard shortcut, and that focus/tab order never traps the user
in a mouse-only state. No fuzzy command palette in v1, per confirmed spec.

**Acceptance criteria:**
- [ ] Every menu/toolbar action has a documented keyboard shortcut
- [ ] Tab/focus order is sane and never traps keyboard-only navigation
- [ ] Shortcut list is documented somewhere discoverable (e.g. a help/shortcuts view or README)

**Verification:**
- [ ] Manual check: full keyboard-only walkthrough of every documented action, no mouse

**Dependencies:** Task 11

**Files likely touched:** `src/shortcuts.ts`, `src/main.ts`, `README.md` or `docs/shortcuts.md`

**Estimated scope:** S

---

## Task 15: Minimum discoverable UI

**Description:** Added after the Core Loop checkpoint, where a real install on Windows surfaced
that the app is undiscoverable to anyone who doesn't already know its shortcuts. Not a polish
task — SPEC.md's success criterion is a friend being productive without hand-holding, and that is
currently impossible. Scope is deliberately minimal: the smallest set of affordances that makes
the app self-explanatory, *not* a toolbar or any markup-inserting UI (still explicitly out of
scope per the confirmed intent).

**Acceptance criteria:**
- [ ] A native menu bar with File (New, Open…, Save, Save As…) and Help (Keyboard Shortcuts),
      each item showing its shortcut, so every action is reachable without prior knowledge
- [ ] Window title shows the open document's filename, or "Untitled" for a new one
- [ ] An unsaved-changes indicator (e.g. a dot or asterisk in the title), and a confirm prompt
      before discarding unsaved work on New/Open/close
- [ ] The editor/preview split is draggable, and the preview has a zoom control

**Verification:**
- [ ] Build succeeds: `pnpm tauri build`
- [ ] Manual check: someone unfamiliar with ams opens it and saves a document without being told
      anything

**Dependencies:** Task 11

**Files likely touched:** `src-tauri/src/lib.rs` (menu), `src-tauri/src/commands.rs`, `src/main.ts`,
`index.html`, `src/styles.css`

**Estimated scope:** M

---

## Task 13: Finalize bundler config

**Description:** Configure Tauri's bundler for real release artifacts: app icons, name/version
metadata, and the WebView2 bootstrapper option for Windows so end users without WebView2
pre-installed still get a working install.

**Acceptance criteria:**
- [ ] Windows `.msi`/`.exe` includes correct icon/metadata and WebView2 bootstrap handling
- [ ] Linux AppImage includes correct icon/metadata
- [ ] Version number is sourced from one place (not duplicated across config files)

**Verification:**
- [ ] Build succeeds: `pnpm tauri build` on both platforms (via CI)
- [ ] Manual check: installed app shows correct name/icon/version

**Dependencies:** Task 2, Task 12

**Files likely touched:** `src-tauri/tauri.conf.json`, `src-tauri/icons/`

**Estimated scope:** S

---

## Task 14: Clean-machine install verification

**Description:** Install the release-config Windows and Linux artifacts on machines without any
prior dev setup (a clean VM or a friend's actual machine), confirming the v1 success criterion
directly: a non-technical person installs cold and is productive without help.

**Acceptance criteria:**
- [ ] Windows installer runs cold on a clean Windows machine/VM with no prior setup
- [ ] Linux AppImage runs cold on a clean Linux machine with no prior setup
- [ ] A person unfamiliar with the app can create and save a simple document without guidance

**Verification:**
- [ ] Manual check: the install + first-use walkthrough itself, ideally with an actual friend per
      the original success criterion, not just the founder

**Dependencies:** Task 13

**Files likely touched:** None (verification task)

**Estimated scope:** XS

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Windows installer can't actually be produced/verified from a Linux dev machine | High | Task 2 proves the CI cross-build pipeline before any feature work depends on it |
| WebView2 missing/outdated on a friend's Windows machine | Medium | Tauri bundler's WebView2 bootstrapper option, enabled in Task 13 |
| Comemo/incremental caching doesn't actually keep 500-page recompile under budget | High | Task 7's benchmark test is an automated, CI-enforced regression guard, not a one-time manual check |
| Font licensing blocks redistribution | Medium | Verify license (OFL or equivalent) before embedding in Task 4 |
| No mature Typst Lezer grammar exists for CodeMirror 6 | Medium | Task 9 checks for an existing community grammar first; falls back to a minimal custom one only if needed |
| `typst` crate's public API changes before implementation (young, fast-moving library) | Medium | Pin an exact version in `Cargo.toml`; re-verify API against current docs at Task 3 implementation time (`source-driven-development`) |

## Open Questions

None outstanding. Resolved 2026-09-05: CI is GitHub Actions, confirmed. Task 14's clean-machine
verification will use the founder's own Windows laptop — no separate VM needed.

## Parallelization Opportunities

- **Task 2 (CI proof) ‖ Phase 2 (compile engine, Tasks 3-7).** Both depend only on Task 1. They
  touch disjoint files (`.github/workflows/` vs `src-tauri/src/compile/`) and share no state.
  Safe to run concurrently.
- **Task 9 (editor pane, frontend) ‖ Phase 2 (compile engine, Tasks 3-7).** This is the largest
  opportunity: Task 9 depends only on Task 1, not on the compile engine existing yet (CodeMirror
  integration is UI-only until Task 11 wires it to real compiled output). Touches `src/` while
  Phase 2 touches `src-tauri/src/compile/` — disjoint. Running these concurrently means the
  frontend track isn't blocked behind the entire Rust backend track.
- **Within Phase 2: Tasks 4, 5, 6 can parallelize once Task 3 lands.** Each targets a distinct
  file (`fonts.rs`, `session.rs`, `world.rs` respectively). One adjustment to fully avoid
  contention: give each its own test file (`tests/fonts.rs`, `tests/incremental_cache.rs`,
  `tests/path_resolution.rs`) instead of all three appending to `tests/compile.rs`.
- **Task 8 ↔ Task 10 needs coordination, not just parallelization.** Task 10 (preview pane)
  consumes the compiled-document/page-output shape that Task 8 (Tauri commands) defines. Fix that
  IPC data contract first (a short type definition, not the full implementation), then Task 8 and
  Task 10 can proceed concurrently against the agreed shape.
- **Must stay sequential:** Task 1 (everything depends on it), Task 7 (needs Task 5's cache
  behavior to exist to benchmark it), Task 11 (needs both 9 and 10 finished), Tasks 12-14
  (each depends on the previous).

Net effect if parallelized: Phase 1 + Phase 2 + Task 9 could collapse into roughly two concurrent
tracks (backend: 2→3→{4,5,6}→7; frontend: 9) instead of one long sequential chain, with Task 8/10
as the sync point before Task 11. Not spinning up parallel agents for this yet — flagging the
opportunity now since it changes how Phase 1-3 could be scheduled once implementation starts.
