# ams v1 — Task List

Full detail (acceptance criteria, verification, files, dependencies) lives in `tasks/plan.md`.
This file is the checklist `/build` and future sessions track progress against.

## Phase 1: Foundation & Packaging Risk

- [x] Task 1: Scaffold Tauri 2 + Vite + TypeScript project
- [x] Task 2: CI cross-platform packaging proof (Windows + Linux)

### Checkpoint: Foundation
- [x] Builds via `pnpm tauri build` locally (binary + `.deb`/`.rpm`; AppImage bundling fails on
      this Arch dev box because linuxdeploy-plugin-gtk expects a Debian-style gdk-pixbuf layout —
      not an app bug, and CI's Ubuntu runner produces the AppImage correctly)
- [x] CI produces a Windows `.msi` + NSIS `.exe` and a Linux AppImage as artifacts (verified by
      downloading them: real PE32 NSIS installer, real MSI database, real ELF AppImage)
- [ ] Both artifacts install/launch on a clean machine — Windows side is Task 14, on the
      founder's own Windows laptop
- [ ] Reviewed with human

## Phase 2: Compile Engine

- [x] Task 3: Embed `typst` crate + minimal `World` implementation
- [x] Task 4: Bundle default font set
- [x] Task 5: Incremental recompilation via cache reuse
- [x] Task 6: Sibling-path resolution for images/`.bib` files
- [x] Task 7: Large-document recompile-latency benchmark test (100ms budget)

### Checkpoint: Compile Engine
- [x] `cargo test --workspace` passes, including the benchmark test
- [x] `cargo clippy --workspace -- -D warnings` clean
- [ ] Reviewed with human

## Phase 3: Core Loop — Editor + Preview + File I/O

- [x] Task 8: Tauri commands for file I/O + compile invocation
- [x] Task 9: Editor pane (CodeMirror 6 + Typst syntax highlighting)
- [x] Task 10: Preview pane (virtualized canvas rendering)
- [x] Task 11: Wire the full edit → compile → preview loop end-to-end

### Checkpoint: Core Loop
- [ ] Manual smoke test: new doc → type → preview updates → save → reopen
- [ ] Compilation confirmed off the UI thread
- [ ] Reviewed with human

## Phase 4: Keyboard-First Pass

- [ ] Task 12: Full keyboard shortcut coverage + focus management

### Checkpoint: Keyboard-First
- [ ] Manual keyboard-only walkthrough passes, no mouse
- [ ] Reviewed with human

## Phase 5: Release Readiness

- [ ] Task 13: Finalize bundler config (icons, metadata, versioning, WebView2 bootstrapper)
- [ ] Task 14: Clean-machine install verification (Windows + Linux)

### Checkpoint: v1 Complete
- [ ] All SPEC.md Success Criteria verified
- [ ] `cargo test --workspace`, `cargo clippy -- -D warnings`, `tsc --noEmit` pass in CI
- [ ] Ready for the founder + friends beta
