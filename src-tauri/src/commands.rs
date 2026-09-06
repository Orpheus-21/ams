//! Tauri commands: thin wrappers that move work off the UI-relevant path and
//! translate native dialogs into plain values. The logic they call lives in
//! `document.rs`.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::compile::Completions;
use crate::document::{
    compile_at, complete_at, current_path, jump_at, open_at, preview_position_at, render_page_at,
    reset, save_at, CompileResult, DocumentState, OpenedDocument, PreviewPosition, RenderedPage,
};

#[tauri::command]
pub fn new_document(state: State<'_, DocumentState>) {
    reset(&state);
}

/// Shows a native "Open" dialog and reads the chosen file. Returns `None` if
/// the user cancelled, never a Tauri error, so cancelling isn't treated as a
/// failure by the frontend.
#[tauri::command]
pub async fn open_document(app: AppHandle) -> Result<Option<OpenedDocument>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(picked) = app.dialog().file().add_filter("Typst", &["typ"]).blocking_pick_file() else {
            return Ok(None);
        };
        let path = picked.into_path().map_err(|e| e.to_string())?;
        let state = app.state::<DocumentState>();
        open_at(&state, path).map(Some)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Saves to the document's current path, or shows a native "Save As" dialog
/// first if it doesn't have one yet. `save_as` forces the dialog even when the
/// document already has a path.
///
/// Returns the path saved to, or `None` if the user cancelled the dialog —
/// matching [`open_document`], so cancelling is never an error the frontend
/// has to recognise by its message text.
#[tauri::command]
pub async fn save_document(
    app: AppHandle,
    text: String,
    save_as: bool,
) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DocumentState>();
        let path = match current_path(&state).filter(|_| !save_as) {
            Some(p) => p,
            None => {
                let Some(picked) =
                    app.dialog().file().add_filter("Typst", &["typ"]).blocking_save_file()
                else {
                    return Ok(None);
                };
                picked.into_path().map_err(|e| e.to_string())?
            }
        };
        save_at(&state, path, text).map(Some)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Recompiles the document's current text. Runs on a blocking-task thread —
/// never the UI-relevant path — per `SPEC.md`'s must-not-hang requirement.
#[tauri::command]
pub async fn compile_document(app: AppHandle, text: String) -> Result<CompileResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DocumentState>();
        compile_at(&state, &text)
    })
    .await
    .map_err(|e| e.to_string())
}

/// Completions at the cursor, from Typst's own compiler.
#[tauri::command]
pub async fn complete(app: AppHandle, text: String, cursor: usize) -> Result<Completions, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DocumentState>();
        complete_at(&state, &text, cursor)
    })
    .await
    .map_err(|e| e.to_string())
}

/// Where a click on the preview maps to in the source, as a UTF-16 offset.
/// `None` when the click hits nothing that came from this document.
#[tauri::command]
pub async fn jump_to_source(
    app: AppHandle,
    page: usize,
    x_pt: f64,
    y_pt: f64,
) -> Result<Option<usize>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DocumentState>();
        jump_at(&state, page, x_pt, y_pt)
    })
    .await
    .map_err(|e| e.to_string())
}

/// Where the cursor's text sits in the preview, so the preview can follow it.
#[tauri::command]
pub async fn preview_position(
    app: AppHandle,
    cursor: usize,
) -> Result<Option<PreviewPosition>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DocumentState>();
        preview_position_at(&state, cursor)
    })
    .await
    .map_err(|e| e.to_string())
}

/// Renders one page of the last compiled document. Also runs on a
/// blocking-task thread: rasterizing is CPU-bound like compilation is.
#[tauri::command]
pub async fn render_page(app: AppHandle, index: usize, pixel_per_pt: f32) -> Result<RenderedPage, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DocumentState>();
        render_page_at(&state, index, pixel_per_pt)
    })
    .await
    .map_err(|e| e.to_string())?
}
