import { mountEditor, getContent, setContent } from "./editor/setup";
import { newDocument, openDocument, saveDocument } from "./commands";

// Minimal proof that the new/open/save Tauri commands round-trip correctly
// (Task 8). Full shortcut coverage, menus, and focus management are Task 12.
window.addEventListener("keydown", (event) => {
  const mod = event.ctrlKey || event.metaKey;
  if (!mod) return;

  if (event.key === "n") {
    event.preventDefault();
    newDocument().then(() => setContent(""));
  } else if (event.key === "o") {
    event.preventDefault();
    openDocument().then((doc) => {
      if (doc) setContent(doc.text);
    });
  } else if (event.key === "s") {
    event.preventDefault();
    saveDocument(getContent());
  }
});

window.addEventListener("DOMContentLoaded", () => {
  const container = document.querySelector<HTMLElement>("#editor");
  if (container) mountEditor(container);
});
