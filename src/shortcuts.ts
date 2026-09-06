// The Help > Keyboard Shortcuts overlay.
//
// The menu already shows each shortcut next to its item, so this exists for
// the one thing a menu can't show: the editing keys that belong to the editor
// itself rather than to a menu command.

const SHORTCUTS: Array<[string, string]> = [
  ["Ctrl+N", "New document"],
  ["Ctrl+O", "Open document"],
  ["Ctrl+S", "Save"],
  ["Ctrl+Shift+S", "Save as"],
  ["Ctrl+=  /  Ctrl+-", "Zoom preview in / out"],
  ["Ctrl+0", "Reset preview zoom"],
  ["Ctrl+1  /  Ctrl+2", "Focus editor / preview"],
  ["Ctrl+Z  /  Ctrl+Y", "Undo / redo"],
  ["Ctrl+F", "Find in document"],
  ["Click a page", "Jump to the source that produced it"],
  ["F1", "Show this list"],
  ["Esc", "Close this list"],
];

let overlay: HTMLElement | null = null;

export function mountShortcuts(element: HTMLElement): void {
  overlay = element;

  const table = document.createElement("dl");
  table.className = "shortcut-list";
  for (const [keys, description] of SHORTCUTS) {
    const key = document.createElement("dt");
    key.textContent = keys;
    const label = document.createElement("dd");
    label.textContent = description;
    table.append(key, label);
  }

  const title = document.createElement("h2");
  title.textContent = "Keyboard shortcuts";

  const panel = document.createElement("div");
  panel.className = "shortcut-panel";
  panel.append(title, table);

  overlay.replaceChildren(panel);
  overlay.hidden = true;

  // Clicking anywhere outside the panel dismisses it.
  overlay.addEventListener("click", (event) => {
    if (event.target === overlay) hideShortcuts();
  });
}

export function toggleShortcuts(): void {
  if (!overlay) return;
  overlay.hidden = !overlay.hidden;
}

export function hideShortcuts(): boolean {
  if (!overlay || overlay.hidden) return false;
  overlay.hidden = true;
  return true;
}
