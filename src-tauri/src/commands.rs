//! Tauri commands bridging the frontend to the compile engine: new/open/save
//! a document, and invoke a compile. Each command is a thin wrapper around a
//! plain function below, so the actual logic is testable without a running
//! Tauri app or a native file dialog.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::compile::{CompileError, CompileSession};

/// The single open document's state: which file (if any) it's saved to, and
/// the compile session tied to that file's directory (for sibling-asset
/// resolution and incremental recompilation across edits).
pub struct DocumentState {
    inner: Mutex<Inner>,
}

struct Inner {
    path: Option<PathBuf>,
    session: CompileSession,
}

impl Default for DocumentState {
    fn default() -> Self {
        Self { inner: Mutex::new(Inner { path: None, session: CompileSession::detached(String::new()) }) }
    }
}

#[derive(Serialize)]
pub struct OpenedDocument {
    pub path: String,
    pub text: String,
}

#[derive(Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub page_count: usize,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub severity: &'static str,
    pub message: String,
}

fn error_diagnostics(err: &CompileError) -> Vec<Diagnostic> {
    err.0.iter().map(|d| Diagnostic { severity: "error", message: d.message.to_string() }).collect()
}

fn file_name_of(path: &Path) -> String {
    path.file_name().and_then(|n| n.to_str()).unwrap_or("main.typ").to_string()
}

fn root_of(path: &Path) -> PathBuf {
    path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
}

// -- Plain, testable logic (no Tauri app needed) ----------------------------

fn reset(state: &DocumentState) {
    let mut inner = state.inner.lock().unwrap();
    inner.path = None;
    inner.session = CompileSession::detached(String::new());
}

fn open_at(state: &DocumentState, path: PathBuf) -> Result<OpenedDocument, String> {
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut inner = state.inner.lock().unwrap();
    inner.session = CompileSession::new(root_of(&path), &file_name_of(&path), text.clone());
    inner.path = Some(path.clone());
    Ok(OpenedDocument { path: path.display().to_string(), text })
}

fn save_at(state: &DocumentState, path: PathBuf, text: String) -> Result<String, String> {
    fs::write(&path, &text).map_err(|e| e.to_string())?;
    let mut inner = state.inner.lock().unwrap();
    inner.session = CompileSession::new(root_of(&path), &file_name_of(&path), text);
    inner.path = Some(path.clone());
    Ok(path.display().to_string())
}

fn current_path(state: &DocumentState) -> Option<PathBuf> {
    state.inner.lock().unwrap().path.clone()
}

fn compile_at(state: &DocumentState, text: &str) -> CompileResult {
    let mut inner = state.inner.lock().unwrap();
    match inner.session.recompile(text) {
        Ok(output) => CompileResult {
            success: true,
            page_count: output.document.pages().len(),
            diagnostics: output
                .warnings
                .iter()
                .map(|w| Diagnostic { severity: "warning", message: w.message.to_string() })
                .collect(),
        },
        Err(err) => CompileResult { success: false, page_count: 0, diagnostics: error_diagnostics(&err) },
    }
}

// -- Tauri commands -----------------------------------------------------

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
/// first if it doesn't have one yet. Returns the path saved to.
#[tauri::command]
pub async fn save_document(app: AppHandle, text: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DocumentState>();
        let path = match current_path(&state) {
            Some(p) => p,
            None => {
                let picked = app.dialog().file().add_filter("Typst", &["typ"]).blocking_save_file();
                let picked = picked.ok_or_else(|| "save cancelled".to_string())?;
                picked.into_path().map_err(|e| e.to_string())?
            }
        };
        save_at(&state, path, text)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_reads_file_and_starts_a_rooted_session() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.typ");
        fs::write(&path, "= Hello").unwrap();

        let state = DocumentState::default();
        let opened = open_at(&state, path.clone()).expect("open should succeed");

        assert_eq!(opened.text, "= Hello");
        assert_eq!(current_path(&state), Some(path));
    }

    #[test]
    fn open_missing_file_is_an_error_not_a_panic() {
        let state = DocumentState::default();
        let result = open_at(&state, PathBuf::from("/nonexistent/doc.typ"));
        assert!(result.is_err());
    }

    #[test]
    fn save_writes_file_and_updates_current_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.typ");

        let state = DocumentState::default();
        let saved_path = save_at(&state, path.clone(), "= Hi".into()).expect("save should succeed");

        assert_eq!(saved_path, path.display().to_string());
        assert_eq!(fs::read_to_string(&path).unwrap(), "= Hi");
        assert_eq!(current_path(&state), Some(path));
    }

    #[test]
    fn new_document_clears_current_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.typ");
        let state = DocumentState::default();
        save_at(&state, path, "= Hi".into()).unwrap();

        reset(&state);

        assert_eq!(current_path(&state), None);
    }

    #[test]
    fn compile_success_reports_page_count() {
        let state = DocumentState::default();
        let result = compile_at(&state, "= Hello\nBody text.");
        assert!(result.success);
        assert_eq!(result.page_count, 1);
    }

    #[test]
    fn compile_error_surfaces_diagnostics_not_a_panic() {
        let state = DocumentState::default();
        let result = compile_at(&state, "#unknown_function()");
        assert!(!result.success);
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn saving_after_opening_a_different_file_resolves_siblings_from_the_new_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("data.txt"), "sibling content").unwrap();
        let path = dir.path().join("doc.typ");

        let state = DocumentState::default();
        save_at(&state, path, "#read(\"data.txt\")".into()).unwrap();

        let result = compile_at(&state, "#read(\"data.txt\")");
        assert!(result.success, "diagnostics: {:?}", result.diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>());
    }
}
