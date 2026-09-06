# ams

Write [Typst](https://typst.app) documents in a desktop app. Markup on the left, rendered
pages on the right, updating as you type. No terminal, no editor extension, no config.

Typst is excellent and its tooling assumes you're a programmer. ams is for people who
aren't.

## install

Windows and Linux. No macOS build.

Grab an installer from the [latest release](https://github.com/Orpheus-21/ams/releases).

(Untagged builds are also on the [Actions](https://github.com/Orpheus-21/ams/actions) tab, but
those artifacts expire after 90 days and need a GitHub login.)

- **Windows**: unzip, run `ams_x.y.z_x64-setup.exe`. Windows will say *"Windows protected
  your PC"* because the installer isn't code-signed — More info → Run anyway. Installs
  per-user, no admin prompt.
- **Linux**: unzip, `chmod +x ams_x.y.z_amd64.AppImage`, run it.

## use

Type Typst markup. The preview updates about 120ms after you stop.

    = A heading

    Some text, with *bold* and _italic_.

Everything is on the menu bar, with its shortcut next to it. `F1` lists them.

Typing `#` offers completions from the compiler itself, with descriptions. Clicking anywhere
on a page jumps the cursor to the source that produced it, and the preview follows the cursor
as you move around.

Documents are plain `.typ` files. Images and `.bib` files are resolved relative to the
document, so put them in the same folder — use your file manager, ams has no file browser.

New to Typst? The [official docs](https://typst.app/docs) are genuinely good, and its
[package registry](https://typst.app/universe) has templates for most document types.

## what it doesn't do

- No visual/WYSIWYG editing. You write markup. That's the point of Typst.
- No importing `.docx`. Start your next document here instead.
- No file tree, no tabs, no project management. One document at a time.
- Syntax highlighting switches off above 2000 lines. The available Typst grammar reparses
  the whole document on every keystroke, which stalls long documents; editing stays fast
  without it. Fixable upstream, not fixed here.
- Packages (`#import "@preview/..."`) need internet the first time — Typst caches them
  after that. Making a fresh install work fully offline is a later job.

## build

Needs Rust, Node, pnpm, and the [Tauri Linux
deps](https://tauri.app/start/prerequisites/) if you're on Linux.

    pnpm install
    pnpm tauri dev

    cargo test --workspace --manifest-path src-tauri/Cargo.toml
    cargo clippy --workspace --manifest-path src-tauri/Cargo.toml -- -D warnings
    pnpm exec tsc --noEmit

`pnpm tauri build` produces installers. AppImage bundling fails on Arch — `linuxdeploy`
expects a Debian-style gdk-pixbuf layout — so Linux releases are built in CI on Ubuntu.
Use `--no-bundle` locally.

`SPEC.md` is what ams is meant to be; `tasks/plan.md` is how it got here and what's left.

## license

GPL-3.0-or-later. Copyright (C) 2026 Orpheus-21. See `LICENSE`.
