# ams — Confirmed Intent

Captured via `interview-me`, confirmed 2026-09-05.

## Outcome
A native, fast, keyboard-first desktop app ("ams") — Typst markup on one side, instantly
live-rendered preview on the other — that removes the CLI/VS Code/extension barrier keeping
non-programmers away from Typst.

## User
Non-technical people currently doing everyday documents in Word (starting with the founder +
friends, Windows and Linux), plus power users who are served by Typst's own packages/templates/
ecosystem rather than bespoke app features.

## Why now
Typst is already fast and powerful, but has zero on-ramp for anyone who isn't comfortable with a
terminal and an editor extension. ams is that on-ramp.

## Success (v1)
- You write a real document start-to-finish using only ams, hand the Windows installer to a
  friend cold, and they're productive without you sitting next to them.
- Windows ships as a simple `.exe` + installer from the first version — not a later fast-follow.

## Constraints
- Must be extremely fast, on Windows too — Electron is disqualified.
- Live preview must never lag noticeably, and the UI must never hang or block, even on 400-500
  page documents. Implies: compilation/rendering off the UI thread, incremental recompilation
  (reuse Typst's `World`/memoization across edits), viewport-only preview rendering — not a full
  re-render per keystroke.
- Cold start must also be fast.
- Every action must be reachable keyboard-only.
- Windows v1 must ship a simple installer (no manual runtime setup, no CLI steps) alongside the
  `.exe`, from day one.

## Out of scope for v1
- No visual/WYSIWYG editing, no toolbar that inserts markup for you.
- No `.docx` import.
- No in-app file browser or multi-file project sidebar — OS file manager handles sibling assets
  (images, `.bib` files); the app just resolves relative paths correctly, same as the Typst
  compiler already does.
- No bespoke lawyer/publishing-house features (citations UI, print/prepress, bleed, CMYK,
  imposition) — served by Typst's own ecosystem + documentation, not app features.
- No macOS build.

## Recommended stack (confirmed direction, to be finalized in spec)
- **Core:** Rust, `typst` crate embedded in-process (not shelling out to the CLI), so Typst's own
  memoization (`comemo`) carries incremental recompilation across edits.
- **App shell:** Tauri 2 — Rust backend + OS-native webview (WebView2 on Windows, WebKitGTK on
  Linux). Not Electron: no bundled Chromium, small binary, fast cold start.
- **Editor pane:** CodeMirror 6, inside the webview. Reused rather than hand-built because it
  already solves large-document virtualized editing + incremental syntax highlighting — building
  that from scratch in a fully native Rust GUI toolkit was assessed and rejected as unnecessary
  engineering risk for this project.
- **Preview pane:** custom canvas rendering from Typst's per-page render output, virtualized to
  only rasterize pages in the current viewport. Compilation runs off the UI thread.
- **Known risk, accepted:** WebView2 has a small first-launch init cost on Windows (preinstalled
  on Win10 1809+/Win11, so this is milliseconds, not Electron-scale). Fallback if this proves
  unacceptable in practice: fully native Rust GUI + custom/rope-based editor — a strictly bigger
  build, only worth it if Tauri demonstrably fails the speed bar under real measurement.

## Platforms
Windows and Linux for v1. No macOS.
