# ams v1 — Task List

Full detail (acceptance criteria, verification, files, dependencies) lives in `tasks/plan.md`.
This file is the checklist `/build` and future sessions track progress against.

## Phase 1: Foundation & Packaging Risk

- [ ] Task 1: Scaffold Tauri 2 + Vite + TypeScript project
- [ ] Task 2: CI cross-platform packaging proof (Windows + Linux)

### Checkpoint: Foundation
- [ ] Empty scaffold builds via `pnpm tauri build` locally
- [ ] CI produces a Windows `.msi`/`.exe` and a Linux AppImage as artifacts
- [ ] Both artifacts install/launch
- [ ] Reviewed with human

## Phase 2: Compile Engine

- [ ] Task 3: Embed `typst` crate + minimal `World` implementation
- [ ] Task 4: Bundle default font set
- [ ] Task 5: Incremental recompilation via cache reuse
- [ ] Task 6: Sibling-path resolution for images/`.bib` files
- [ ] Task 7: Large-document recompile-latency benchmark test (100ms budget)

### Checkpoint: Compile Engine
- [ ] `cargo test --workspace` passes, including the benchmark test
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] Reviewed with human

## Phase 3: Core Loop — Editor + Preview + File I/O

- [ ] Task 8: Tauri commands for file I/O + compile invocation
- [ ] Task 9: Editor pane (CodeMirror 6 + Typst syntax highlighting)
- [ ] Task 10: Preview pane (virtualized canvas rendering)
- [ ] Task 11: Wire the full edit → compile → preview loop end-to-end

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
