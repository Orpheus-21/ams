import { mountEditor } from "./editor/setup";

window.addEventListener("DOMContentLoaded", () => {
  const container = document.querySelector<HTMLElement>("#editor");
  if (container) mountEditor(container);
});
