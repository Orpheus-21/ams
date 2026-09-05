import { EditorView, basicSetup } from "codemirror";
import { Compartment, EditorState } from "@codemirror/state";
import { typst_lezer } from "codemirror-lang-typst/lezer";

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
  options: { initialContent?: string; onChange?: () => void } = {},
): EditorView {
  const { initialContent = "", onChange } = options;

  const changeNotifier = EditorView.updateListener.of((update) => {
    if (update.docChanged) onChange?.();
  });

  view = new EditorView({
    state: EditorState.create({
      doc: initialContent,
      extensions: [
        basicSetup,
        language.of(languageExtension(countLines(initialContent))),
        languageSizeGuard,
        ...(onChange ? [changeNotifier] : []),
      ],
    }),
    parent,
  });
  return view;
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
