import { EditorView, basicSetup } from "codemirror";
import { Compartment, EditorState } from "@codemirror/state";
import { typst_lezer } from "codemirror-lang-typst/lezer";

import { editorAppearance } from "./theme";
import { typstAutocomplete } from "./complete";

// ponytail: codemirror-lang-typst's Lezer parser re-parses the *entire*
// document on every edit (its fragment-reuse path only fires when nothing at
// all changed — see TypstPartialParse.advance() in its dist/lezer.js). On a
// ~20,000-line synthetic doc that's ~300-450ms of blocked main thread per
// keystroke, which breaks the "must not hang on huge documents" requirement.
// Until upstream ships real incremental reparsing (or this gets replaced with
// a proper @lezer/lr grammar), drop syntax highlighting above this size so
// large documents stay editable without lag.
const HIGHLIGHT_LINE_LIMIT = 2000;

function countLines(text: string): number {
  let lines = 1;
  for (let i = 0; i < text.length; i++) if (text.charCodeAt(i) === 10) lines++;
  return lines;
}

function languageExtension(lineCount: number) {
  return lineCount > HIGHLIGHT_LINE_LIMIT ? [] : [typst_lezer()];
}

const language = new Compartment();

const languageSizeGuard = EditorView.updateListener.of((update) => {
  if (!update.docChanged) return;
  const wasHighlighted = update.startState.doc.lines <= HIGHLIGHT_LINE_LIMIT;
  const shouldHighlight = update.state.doc.lines <= HIGHLIGHT_LINE_LIMIT;
  if (wasHighlighted !== shouldHighlight) {
    update.view.dispatch({
      effects: language.reconfigure(languageExtension(update.state.doc.lines)),
    });
  }
});

let view: EditorView | null = null;

export function mountEditor(
  parent: HTMLElement,
  options: {
    initialContent?: string;
    onChange?: () => void;
    onCursorMove?: (cursor: number) => void;
  } = {},
): EditorView {
  const { initialContent = "", onChange, onCursorMove } = options;

  const changeNotifier = EditorView.updateListener.of((update) => {
    if (update.docChanged) onChange?.();
    if (update.docChanged || update.selectionSet) {
      onCursorMove?.(update.state.selection.main.head);
    }
  });

  view = new EditorView({
    state: EditorState.create({
      doc: initialContent,
      extensions: [
        basicSetup,
        // After basicSetup so it overrides the default highlight style.
        editorAppearance,
        typstAutocomplete,
        language.of(languageExtension(countLines(initialContent))),
        languageSizeGuard,
        ...(onChange || onCursorMove ? [changeNotifier] : []),
      ],
    }),
    parent,
  });
  return view;
}

export function focusEditor(): void {
  view?.focus();
}

/// Puts the cursor at a UTF-16 offset and scrolls it into view.
export function revealOffset(offset: number): void {
  if (!view) return;
  const position = Math.min(Math.max(offset, 0), view.state.doc.length);
  view.dispatch({
    selection: { anchor: position },
    effects: EditorView.scrollIntoView(position, { y: "center" }),
  });
  view.focus();
}

export function getContent(): string {
  if (!view) throw new Error("Editor not mounted");
  return view.state.doc.toString();
}

export function setContent(content: string): void {
  if (!view) throw new Error("Editor not mounted");
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: content },
    effects: language.reconfigure(languageExtension(countLines(content))),
  });
}
