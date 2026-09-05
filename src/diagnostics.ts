// Surfaces compile errors and warnings. Task 11's third acceptance criterion
// is that these are never silently swallowed — a failed compile leaves the
// last good preview on screen, so the bar is the only thing telling the user
// why the preview stopped matching what they typed.

import type { Diagnostic } from "./commands";

let bar: HTMLElement | null = null;

export function mountDiagnostics(element: HTMLElement): void {
  bar = element;
  clearDiagnostics();
}

export function showDiagnostics(diagnostics: Diagnostic[]): void {
  if (!bar) return;

  if (diagnostics.length === 0) {
    clearDiagnostics();
    return;
  }

  bar.hidden = false;
  bar.dataset.severity = diagnostics.some((d) => d.severity === "error") ? "error" : "warning";
  bar.replaceChildren(
    ...diagnostics.map((diagnostic) => {
      const line = document.createElement("div");
      line.className = "diagnostic";
      line.textContent = diagnostic.message;
      return line;
    }),
  );
}

/// Reports a failure of the compile call itself (IPC/backend), as opposed to
/// a Typst diagnostic about the document.
export function showFailure(message: string): void {
  showDiagnostics([{ severity: "error", message }]);
}

export function clearDiagnostics(): void {
  if (!bar) return;
  bar.hidden = true;
  bar.replaceChildren();
}
