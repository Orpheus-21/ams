//! The native application menu.
//!
//! Its real job is discoverability. SPEC.md's success criterion is a friend
//! being productive without anyone sitting next to them, and until this
//! existed the only way to open or save a document was to already know that
//! Ctrl+O exists. The menu is how a desktop app tells a first-time user what
//! it can do — and each item displays its shortcut, so it teaches the
//! keyboard path rather than replacing it.
//!
//! Menu items only *signal*: the id is forwarded to the frontend, which owns
//! the document state and decides what to do. Keeping the action in one place
//! means the menu and the keyboard can't drift apart.

use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Runtime};

/// Event name carrying a menu item id to the frontend.
pub const MENU_EVENT: &str = "menu";

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    // Accelerator strings are parsed at startup and a bad one fails the whole
    // app, so these use key names verified against muda's parser ("Equal" and
    // "Minus" parse; "Plus" does not).
    let file = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &MenuItem::with_id(app, "new", "New", true, Some("CmdOrCtrl+N"))?,
            &MenuItem::with_id(app, "open", "Open...", true, Some("CmdOrCtrl+O"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "save", "Save", true, Some("CmdOrCtrl+S"))?,
            &MenuItem::with_id(app, "save_as", "Save As...", true, Some("CmdOrCtrl+Shift+S"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, None)?,
        ],
    )?;

    let view = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &MenuItem::with_id(app, "zoom_in", "Zoom In", true, Some("CmdOrCtrl+Equal"))?,
            &MenuItem::with_id(app, "zoom_out", "Zoom Out", true, Some("CmdOrCtrl+Minus"))?,
            &MenuItem::with_id(app, "zoom_reset", "Reset Zoom", true, Some("CmdOrCtrl+0"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "focus_editor", "Focus Editor", true, Some("CmdOrCtrl+1"))?,
            &MenuItem::with_id(app, "focus_preview", "Focus Preview", true, Some("CmdOrCtrl+2"))?,
        ],
    )?;

    let help = Submenu::with_items(
        app,
        "Help",
        true,
        &[&MenuItem::with_id(app, "shortcuts", "Keyboard Shortcuts", true, Some("F1"))?],
    )?;

    Menu::with_items(app, &[&file, &view, &help])
}

pub fn forward<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let _ = app.emit(MENU_EVENT, event.id().as_ref());
}
