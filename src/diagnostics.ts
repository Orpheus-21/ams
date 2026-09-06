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
      const row = document.createElement("div");
      row.className = "diagnostic";

      // "line 12" first, because the message alone rarely tells you where to
      // look in a document of any length.
      if (diagnostic.line !== null) {
        const where = document.createElement("span");
        where.className = "diagnostic-location";
        where.textContent = `line ${diagnostic.line}`;
        row.append(where, " ");
      }

      row.append(diagnostic.message);

      // Typst's hints are usually the part that actually tells someone new to
      // the language what to do about the error.
      for (const hint of diagnostic.hints) {
        const line = document.createElement("div");
        line.className = "diagnostic-hint";
        line.textContent = `hint: ${hint}`;
        row.append(line);
      }

      return row;
    }),
  );
}

/// Reports a failure of the compile call itself (IPC/backend), as opposed to
/// a Typst diagnostic about the document.
export function showFailure(message: string): void {
  showDiagnostics([{ severity: "error", message, line: null, column: null, hints: [] }]);
}

export function clearDiagnostics(): void {
  if (!bar) return;
  bar.hidden = true;
  bar.replaceChildren();
}
