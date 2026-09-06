import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";

import { mountEditor, getContent, setContent, focusEditor } from "./editor/setup";
import { newDocument, openDocument, saveDocument, compileDocument } from "./commands";
import { mountPreview, type PreviewController } from "./preview/viewport";
import { mountDiagnostics, showDiagnostics, showFailure } from "./diagnostics";
import { mountSplitter } from "./split";
import { mountShortcuts, toggleShortcuts, hideShortcuts } from "./shortcuts";

const COMPILE_DEBOUNCE_MS = 120;
const ZOOM_STEP = 1.25;

let preview: PreviewController | null = null;
let previewPane: HTMLElement | null = null;

let debounceTimer: number | undefined;
let compiling = false;
let compileQueued = false;

// Document identity, tracked here because the frontend owns the editor buffer:
// the backend only learns the text when asked to compile or save.
let currentPath: string | null = null;
let dirty = false;

function fileNameOf(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}

function refreshTitle(): void {
  const name = currentPath ? fileNameOf(currentPath) : "Untitled";
  void getCurrentWindow().setTitle(`${dirty ? "● " : ""}${name} — ams`);
}

function markClean(path: string | null): void {
  currentPath = path;
  dirty = false;
  refreshTitle();
}

// Typing never waits on any of this: edits land in CodeMirror synchronously,
// the compile runs on a background thread in Rust, and an edit arriving mid
// compile just queues the next one instead of blocking or being dropped.
async function compileNow(): Promise<void> {
  if (compiling) {
    compileQueued = true;
    return;
  }

  compiling = true;
  try {
    const result = await compileDocument(getContent());
    showDiagnostics(result.diagnostics);
    // On failure the preview deliberately keeps its last good render — the
    // diagnostics bar is what tells the user it's stale.
    if (result.success) preview?.setPages(result.pages);
  } catch (err) {
    showFailure(String(err));
  } finally {
    compiling = false;
    if (compileQueued) {
      compileQueued = false;
      void compileNow();
    }
  }
}

function onEdit(): void {
  if (!dirty) {
    dirty = true;
    refreshTitle();
  }
  window.clearTimeout(debounceTimer);
  debounceTimer = window.setTimeout(() => void compileNow(), COMPILE_DEBOUNCE_MS);
}

/// Guards the two actions that throw away the buffer. Without this, Ctrl+N on
/// an unsaved document silently destroys it.
async function confirmDiscard(action: string): Promise<boolean> {
  if (!dirty) return true;
  return ask(`You have unsaved changes. ${action} anyway?`, {
    title: "Unsaved changes",
    kind: "warning",
    okLabel: action,
    cancelLabel: "Cancel",
  });
}

async function doNew(): Promise<void> {
  if (!(await confirmDiscard("Discard and start a new document"))) return;
  await newDocument();
  setContent("");
  markClean(null);
  await compileNow();
}

async function doOpen(): Promise<void> {
  if (!(await confirmDiscard("Discard and open another document"))) return;
  const doc = await openDocument();
  if (!doc) return;
  setContent(doc.text);
  markClean(doc.path);
  await compileNow();
}

async function doSave(saveAs: boolean): Promise<void> {
  // The text is captured before the await so that what gets marked clean is
  // exactly what was written, not whatever the buffer holds once the save
  // returns.
  const saved = getContent();
  try {
    const path = await saveDocument(saved, saveAs);
    if (path === null) return; // cancelled, not a failure
    if (getContent() === saved) markClean(path);
    else currentPath = path; // edited mid-save: keep the path, stay dirty
    refreshTitle();
  } catch (err) {
    showFailure(String(err));
  }
}

const MENU_ACTIONS: Record<string, () => void> = {
  new: () => void doNew(),
  open: () => void doOpen(),
  save: () => void doSave(false),
  save_as: () => void doSave(true),
  zoom_in: () => preview?.zoomBy(ZOOM_STEP),
  zoom_out: () => preview?.zoomBy(1 / ZOOM_STEP),
  zoom_reset: () => preview?.resetZoom(),
  focus_editor: () => focusEditor(),
  focus_preview: () => previewPane?.focus(),
  shortcuts: () => toggleShortcuts(),
};

window.addEventListener("DOMContentLoaded", () => {
  const diagnosticsBar = document.querySelector<HTMLElement>("#diagnostics");
  if (diagnosticsBar) mountDiagnostics(diagnosticsBar);

  const overlay = document.querySelector<HTMLElement>("#shortcuts");
  if (overlay) mountShortcuts(overlay);

  previewPane = document.querySelector<HTMLElement>("#preview");
  if (previewPane) preview = mountPreview(previewPane);

  const divider = document.querySelector<HTMLElement>("#divider");
  if (divider) mountSplitter(divider);

  const editorContainer = document.querySelector<HTMLElement>("#editor");
  if (editorContainer) mountEditor(editorContainer, { onChange: onEdit });

  refreshTitle();
  focusEditor();
  void compileNow();
});

// Menu items carry the shortcuts, so the accelerators are registered natively
// rather than here. Escape is the exception: it belongs to the overlay, not
// to any menu command.
void listen<string>("menu", (event) => MENU_ACTIONS[event.payload]?.());

window.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && hideShortcuts()) event.preventDefault();
});
