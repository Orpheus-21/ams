// Completions sourced from the Typst compiler itself.
//
// The point is discovery without hiding anything: someone who doesn't know
// the language types `#` and sees what it can do, with one-line descriptions,
// instead of having to already know. Nothing here decides *what* to suggest —
// that's the compiler's own answer, so it can't drift out of date.

import { autocompletion, type CompletionContext, type CompletionResult } from "@codemirror/autocomplete";
import { tooltips } from "@codemirror/view";
import { invoke } from "@tauri-apps/api/core";

interface CompletionItem {
  label: string;
  apply: string | null;
  detail: string | null;
  kind: string;
}

interface Completions {
  from: number;
  items: CompletionItem[];
}

/// Typst describes insertions with `${...}` placeholders (`text(${})`). We
/// don't implement snippet tab-stops, so they're stripped rather than
/// inserted literally, which would leave `${}` sitting in the document.
function plainInsertion(item: CompletionItem): string {
  const template = item.apply ?? item.label;
  return template.replace(/\$\{([^}]*)\}/g, "$1");
}

async function typstCompletions(context: CompletionContext): Promise<CompletionResult | null> {
  // Without this, every keystroke in ordinary prose would hit the backend.
  // Typst's own completion logic decides the rest.
  if (!context.explicit && !context.matchBefore(/[#@][\w.-]*$|\w{2,}$/)) return null;

  const result = await invoke<Completions>("complete", {
    text: context.state.doc.toString(),
    cursor: context.pos,
  });

  if (result.items.length === 0) return null;

  return {
    from: result.from,
    options: result.items.map((item) => ({
      label: item.label,
      detail: item.detail ?? undefined,
      type: item.kind,
      apply: plainInsertion(item),
    })),
  };
}

export const typstAutocomplete = [
  autocompletion({
    override: [typstCompletions],
    icons: false,
    activateOnTyping: true,
  }),
  // The editor pane clips overflow, which cut the popup off at the divider.
  // Rendering into the body lets it size to its content instead.
  tooltips({ parent: document.body, position: "absolute" }),
];
