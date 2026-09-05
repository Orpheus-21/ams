import { mountEditor, getContent, setContent } from "./editor/setup";
import { newDocument, openDocument, saveDocument, compileDocument } from "./commands";
import { mountPreview, type PreviewController } from "./preview/viewport";

let preview: PreviewController | null = null;

// Compiles the current content and updates the preview pane. This is a
// one-shot call for Task 10's own verification (new/open trigger it below);
// Task 11 adds the debounced recompile-on-every-edit loop.
async function recompileAndPreview(): Promise<void> {
  const result = await compileDocument(getContent());
  preview?.setPages(result.pages);
}

// Minimal proof that the new/open/save Tauri commands round-trip correctly
// (Task 8). Full shortcut coverage, menus, and focus management are Task 12.
window.addEventListener("keydown", (event) => {
  const mod = event.ctrlKey || event.metaKey;
  if (!mod) return;

  if (event.key === "n") {
    event.preventDefault();
    newDocument().then(() => {
      setContent("");
      return recompileAndPreview();
    });
  } else if (event.key === "o") {
    event.preventDefault();
    openDocument().then((doc) => {
      if (!doc) return;
      setContent(doc.text);
      return recompileAndPreview();
    });
  } else if (event.key === "s") {
    event.preventDefault();
    saveDocument(getContent());
  }
});

window.addEventListener("DOMContentLoaded", () => {
  const editorContainer = document.querySelector<HTMLElement>("#editor");
  if (editorContainer) mountEditor(editorContainer);

  const previewContainer = document.querySelector<HTMLElement>("#preview");
  if (previewContainer) preview = mountPreview(previewContainer);

  recompileAndPreview();
});
