import { mountEditor, getContent, setContent } from "./editor/setup";
import { newDocument, openDocument, saveDocument, compileDocument } from "./commands";
import { mountPreview, type PreviewController } from "./preview/viewport";
import { mountDiagnostics, showDiagnostics, showFailure } from "./diagnostics";

const COMPILE_DEBOUNCE_MS = 120;

let preview: PreviewController | null = null;
let debounceTimer: number | undefined;
let compiling = false;
let compileQueued = false;

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

function scheduleCompile(): void {
  window.clearTimeout(debounceTimer);
  debounceTimer = window.setTimeout(() => void compileNow(), COMPILE_DEBOUNCE_MS);
}

// Full shortcut coverage, menus, and focus management are Task 12.
window.addEventListener("keydown", (event) => {
  const mod = event.ctrlKey || event.metaKey;
  if (!mod) return;

  if (event.key === "n") {
    event.preventDefault();
    // setContent triggers the editor's change listener, which schedules the
    // compile — so these only need to get the backend and editor in sync.
    newDocument().then(() => setContent(""));
  } else if (event.key === "o") {
    event.preventDefault();
    openDocument().then((doc) => {
      if (doc) setContent(doc.text);
    });
  } else if (event.key === "s") {
    event.preventDefault();
    saveDocument(getContent()).catch((err) => showFailure(String(err)));
  }
});

window.addEventListener("DOMContentLoaded", () => {
  const diagnosticsBar = document.querySelector<HTMLElement>("#diagnostics");
  if (diagnosticsBar) mountDiagnostics(diagnosticsBar);

  const previewContainer = document.querySelector<HTMLElement>("#preview");
  if (previewContainer) preview = mountPreview(previewContainer);

  const editorContainer = document.querySelector<HTMLElement>("#editor");
  if (editorContainer) mountEditor(editorContainer, { onChange: scheduleCompile });

  void compileNow();
});
