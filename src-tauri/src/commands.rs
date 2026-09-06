//! Tauri commands bridging the frontend to the compile engine: new/open/save
//! a document, and invoke a compile. Each command is a thin wrapper around a
//! plain function below, so the actual logic is testable without a running
//! Tauri app or a native file dialog.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use typst_layout::PagedDocument;

use typst::diag::SourceDiagnostic;

use crate::compile::{CompileSession, Completions};

/// The single open document's state: which file (if any) it's saved to, the
/// compile session tied to that file's directory (for sibling-asset
/// resolution and incremental recompilation across edits), and the last
/// successfully compiled document (so rendering a page doesn't require
/// recompiling — see `render_page`).
pub struct DocumentState {
    inner: Mutex<Inner>,
}

struct Inner {
    path: Option<PathBuf>,
    session: CompileSession,
    document: Option<PagedDocument>,
}

impl Default for DocumentState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Inner {
                path: None,
                session: CompileSession::detached(String::new()),
                document: None,
            }),
        }
    }
}

#[derive(Serialize)]
pub struct OpenedDocument {
    pub path: String,
    pub text: String,
}

#[derive(Serialize)]
pub struct PageInfo {
    pub width_pt: f64,
    pub height_pt: f64,
}

#[derive(Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub pages: Vec<PageInfo>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize)]
pub struct RenderedPage {
    pub width: u32,
    pub height: u32,
    /// Base64-encoded PNG. Tauri's default IPC serializes `Vec<u8>` as a JSON
    /// array of numbers, which is far heavier over the wire than base64 text
    /// for image-sized payloads.
    pub png_base64: String,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub severity: &'static str,
    pub message: String,
    /// 1-based position in the document, when the diagnostic points at one.
    /// `None` for diagnostics Typst reports without a location, or that point
    /// into a sibling file rather than the open document.
    pub line: Option<usize>,
    pub column: Option<usize>,
    /// Typst's own suggestions for fixing the problem.
    pub hints: Vec<String>,
}

/// Render resolution bounds. 4 px/pt is already well past retina for a page
/// on screen (~2 device pixels per point at 2x zoom); beyond that the pixmap
/// grows quadratically for no visible gain.
const MIN_PIXEL_PER_PT: f32 = 0.1;
const MAX_PIXEL_PER_PT: f32 = 4.0;

fn diagnostics_of(
    session: &CompileSession,
    diagnostics: impl IntoIterator<Item = SourceDiagnostic>,
    severity: &'static str,
) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|d| {
            let (line, column) = match session.location_of(d.span) {
                Some((line, column)) => (Some(line), Some(column)),
                None => (None, None),
            };
            Diagnostic {
                severity,
                message: d.message.to_string(),
                line,
                column,
                // Typst's hints are the genuinely instructive half of a
                // diagnostic ("hint: use #set before the first content") and
                // are exactly what a user new to the language needs.
                hints: d.hints.iter().map(|hint| hint.v.to_string()).collect(),
            }
        })
        .collect()
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
    inner.document = None;
}

fn open_at(state: &DocumentState, path: PathBuf) -> Result<OpenedDocument, String> {
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut inner = state.inner.lock().unwrap();
    inner.session = CompileSession::new(root_of(&path), &file_name_of(&path), text.clone());
    inner.path = Some(path.clone());
    inner.document = None;
    Ok(OpenedDocument { path: path.display().to_string(), text })
}

fn save_at(state: &DocumentState, path: PathBuf, text: String) -> Result<String, String> {
    fs::write(&path, &text).map_err(|e| e.to_string())?;
    let mut inner = state.inner.lock().unwrap();
    inner.session = CompileSession::new(root_of(&path), &file_name_of(&path), text);
    inner.path = Some(path.clone());
    inner.document = None;
    Ok(path.display().to_string())
}

fn current_path(state: &DocumentState) -> Option<PathBuf> {
    state.inner.lock().unwrap().path.clone()
}

fn compile_at(state: &DocumentState, text: &str) -> CompileResult {
    let mut inner = state.inner.lock().unwrap();
    match inner.session.recompile(text) {
        Ok(output) => {
            let pages = output
                .document
                .pages()
                .iter()
                .map(|page| PageInfo {
                    width_pt: page.frame.width().to_pt(),
                    height_pt: page.frame.height().to_pt(),
                })
                .collect();
            let diagnostics = diagnostics_of(&inner.session, output.warnings, "warning");
            inner.document = Some(output.document);
            CompileResult { success: true, pages, diagnostics }
        }
        // A failed compile keeps the last good document renderable on purpose:
        // half-typed markup errors constantly while editing, and blanking the
        // preview on each one is worse than showing the last good render with
        // the error surfaced in the UI. `reset`/`open_at`/`save_at` do clear
        // it, since those genuinely change which document is open.
        Err(err) => {
            let diagnostics = diagnostics_of(&inner.session, err.0, "error");
            CompileResult { success: false, pages: Vec::new(), diagnostics }
        }
    }
}

fn complete_at(state: &DocumentState, text: &str, cursor: usize) -> Completions {
    let mut inner = state.inner.lock().unwrap();
    let Inner { session, document, .. } = &mut *inner;
    session.completions(text, cursor, document.as_ref())
}

fn jump_at(state: &DocumentState, page: usize, x_pt: f64, y_pt: f64) -> Option<usize> {
    let inner = state.inner.lock().unwrap();
    let document = inner.document.as_ref()?;
    inner.session.jump_from_preview(document, page, x_pt, y_pt)
}

/// Rasterizes one page of the last successfully compiled document to PNG, at
/// `pixel_per_pt` resolution. Never re-renders the whole document — the
/// caller (the preview pane's viewport virtualization) decides which single
/// page it currently needs.
fn render_page_at(state: &DocumentState, index: usize, pixel_per_pt: f32) -> Result<RenderedPage, String> {
    // The resolution comes from the frontend (zoom x device pixel ratio), so
    // it gets clamped here rather than trusted: an A4 page at 8 px/pt is a
    // ~32 megapixel, ~128 MB pixmap, and zooming is a held keystroke away.
    let pixel_per_pt = pixel_per_pt.clamp(MIN_PIXEL_PER_PT, MAX_PIXEL_PER_PT);

    let inner = state.inner.lock().unwrap();
    let document = inner.document.as_ref().ok_or("no compiled document to render")?;
    let page = document.pages().get(index).ok_or_else(|| format!("page index {index} out of range"))?;

    let options =
        typst_render::RenderOptions { pixel_per_pt: (pixel_per_pt as f64).into(), ..Default::default() };
    let pixmap = typst_render::render(page, &options);
    let png_bytes = pixmap.encode_png().map_err(|e| e.to_string())?;

    Ok(RenderedPage {
        width: pixmap.width(),
        height: pixmap.height(),
        png_base64: base64::engine::general_purpose::STANDARD.encode(png_bytes),
    })
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
    fn compile_success_reports_page_geometry() {
        let state = DocumentState::default();
        let result = compile_at(&state, "= Hello\nBody text.");
        assert!(result.success);
        assert_eq!(result.pages.len(), 1);
        assert!(result.pages[0].width_pt > 0.0);
        assert!(result.pages[0].height_pt > 0.0);
    }

    #[test]
    fn compile_error_surfaces_diagnostics_not_a_panic() {
        let state = DocumentState::default();
        let result = compile_at(&state, "#unknown_function()");
        assert!(!result.success);
        assert!(result.pages.is_empty());
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn a_diagnostic_reports_the_line_it_is_on() {
        let state = DocumentState::default();
        let result = compile_at(&state, "= Fine\n\nAlso fine.\n\n#unknown_function()\n");

        let diagnostic = result.diagnostics.first().expect("an error was reported");
        assert_eq!(diagnostic.line, Some(5), "the bad call is on line 5");
        assert!(diagnostic.column.is_some());
    }

    #[test]
    fn render_page_rasterizes_the_last_compiled_document() {
        let state = DocumentState::default();
        compile_at(&state, "= Hello\nBody text.");

        let rendered = render_page_at(&state, 0, 2.0).expect("render should succeed");

        assert!(rendered.width > 0 && rendered.height > 0);
        assert!(!rendered.png_base64.is_empty());
        let png_bytes = base64::engine::general_purpose::STANDARD.decode(&rendered.png_base64).unwrap();
        assert_eq!(&png_bytes[..8], b"\x89PNG\r\n\x1a\n", "should be a real PNG");
    }

    #[test]
    fn render_resolution_is_clamped_so_the_frontend_cannot_ask_for_a_huge_pixmap() {
        let state = DocumentState::default();
        compile_at(&state, "= Hello");

        let sane = render_page_at(&state, 0, MAX_PIXEL_PER_PT).expect("render at the cap");
        let absurd = render_page_at(&state, 0, 100.0).expect("render above the cap");

        assert_eq!(
            (absurd.width, absurd.height),
            (sane.width, sane.height),
            "a request above the cap must render at the cap, not allocate a huge pixmap"
        );
    }

    #[test]
    fn render_page_out_of_range_is_an_error_not_a_panic() {
        let state = DocumentState::default();
        compile_at(&state, "= Hello");
        assert!(render_page_at(&state, 5, 1.0).is_err());
    }

    #[test]
    fn render_page_before_any_compile_is_an_error_not_a_panic() {
        let state = DocumentState::default();
        assert!(render_page_at(&state, 0, 1.0).is_err());
    }

    #[test]
    fn a_failed_compile_keeps_the_last_good_document_renderable() {
        let state = DocumentState::default();
        compile_at(&state, "= Hello\nBody text.");
        compile_at(&state, "#unknown_function()");
        assert!(
            render_page_at(&state, 0, 1.0).is_ok(),
            "the preview should keep showing the last good render while the source has an error"
        );
    }

    #[test]
    fn opening_a_different_document_drops_the_previous_render() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.typ");
        fs::write(&path, "= Other").unwrap();

        let state = DocumentState::default();
        compile_at(&state, "= Hello");
        open_at(&state, path).unwrap();

        assert!(render_page_at(&state, 0, 1.0).is_err(), "a newly opened document has nothing rendered yet");
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
