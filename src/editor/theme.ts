// Editor appearance: the chrome (Task 16) and the syntax colours (Task 17).
//
// The palette deliberately stays quiet. This is a window onto prose, not a
// codebase — most of what's on screen is the text the person is writing, so
// markup gets colour only where it distinguishes structure from content, and
// nothing competes with the words themselves. Values mirror the tokens in
// styles.css; CodeMirror needs them in JS, so they're duplicated here rather
// than read from CSS at runtime.

import { EditorView } from "codemirror";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";

const INK = "#23201c";
const INK_MUTED = "#6d655a";
const INK_FAINT = "#9a9184";
const SURFACE = "#faf7f1";
const RULE = "#e3dbcd";
const ACCENT = "#7a4a2b";
const SELECTION = "#e6ddc9";
const ACTIVE_LINE = "#f2ece1";

export const editorTheme = EditorView.theme(
  {
    "&": {
      color: INK,
      backgroundColor: SURFACE,
      fontSize: "14px",
    },
    ".cm-scroller": {
      fontFamily: '"Cascadia Code", "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace',
      lineHeight: "1.7",
      padding: "18px 0",
    },
    ".cm-content": {
      caretColor: ACCENT,
      // Room to breathe on the left, and a right margin so long lines don't
      // run into the divider.
      padding: "0 20px 0 8px",
    },
    ".cm-cursor, .cm-dropCursor": {
      borderLeftColor: ACCENT,
      borderLeftWidth: "2px",
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": {
      backgroundColor: SELECTION,
    },
    ".cm-activeLine": { backgroundColor: ACTIVE_LINE },
    ".cm-gutters": {
      backgroundColor: SURFACE,
      color: INK_FAINT,
      border: "none",
      paddingRight: "4px",
    },
    ".cm-activeLineGutter": {
      backgroundColor: ACTIVE_LINE,
      color: INK_MUTED,
    },
    ".cm-foldPlaceholder": {
      backgroundColor: RULE,
      border: "none",
      color: INK_MUTED,
    },
    ".cm-panels": {
      backgroundColor: SURFACE,
      color: INK,
      borderTop: `1px solid ${RULE}`,
    },
    ".cm-searchMatch": { backgroundColor: "#f3e2c7" },
    ".cm-searchMatch.cm-searchMatch-selected": { backgroundColor: "#e8c98f" },
    ".cm-selectionMatch": { backgroundColor: "#eee6d6" },
    ".cm-matchingBracket, &.cm-focused .cm-matchingBracket": {
      backgroundColor: "#e8dcc2",
      outline: "none",
    },
    ".cm-tooltip": {
      backgroundColor: SURFACE,
      border: `1px solid ${RULE}`,
    },
  },
  { dark: false },
);

/// Structure gets weight, code gets colour, prose is left alone.
export const typstHighlighting = HighlightStyle.define([
  // textDecoration is reset explicitly: the Typst grammar package underlines
  // headings by default, which is loud next to body text.
  { tag: tags.heading, color: INK, fontWeight: "600", textDecoration: "none" },
  { tag: tags.strong, color: INK, fontWeight: "600" },
  { tag: tags.emphasis, color: INK, fontStyle: "italic" },
  { tag: tags.link, color: ACCENT, textDecoration: "underline" },
  { tag: tags.list, color: ACCENT },
  { tag: tags.quote, color: INK_MUTED, fontStyle: "italic" },

  // Everything below is Typst's code side: markers, functions, values.
  { tag: tags.keyword, color: "#7a3b6b" },
  { tag: tags.controlKeyword, color: "#7a3b6b" },
  { tag: tags.function(tags.variableName), color: "#2f5d8a" },
  { tag: tags.variableName, color: "#2f5d8a" },
  { tag: tags.propertyName, color: "#2f5d8a" },
  { tag: tags.string, color: "#4a6b3a" },
  { tag: tags.number, color: "#8a5a1f" },
  { tag: tags.bool, color: "#8a5a1f" },
  { tag: tags.unit, color: "#8a5a1f" },
  { tag: tags.escape, color: "#8a5a1f" },
  { tag: tags.operator, color: INK_MUTED },
  { tag: tags.punctuation, color: INK_MUTED },
  { tag: tags.bracket, color: INK_MUTED },
  { tag: tags.comment, color: INK_FAINT, fontStyle: "italic" },
  { tag: tags.invalid, color: "#8c2f2f" },
]);

export const editorAppearance = [editorTheme, syntaxHighlighting(typstHighlighting)];
