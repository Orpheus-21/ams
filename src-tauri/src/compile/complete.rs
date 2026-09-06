//! Completions from Typst's own compiler.
//!
//! This is the answer to "how does someone who doesn't know Typst discover
//! what it can do" that doesn't involve hiding Typst behind a toolbar: the
//! language describes itself as you type. Suggestions come from
//! `typst-ide`, so they're whatever the compiler actually knows — functions,
//! parameters, symbols, packages — not a list we maintain by hand.

use serde::Serialize;
use typst::World;
use typst_ide::{Completion, CompletionKind, IdeWorld};
use typst_layout::PagedDocument;

use super::world::AmsWorld;

impl IdeWorld for AmsWorld {
    fn upcast(&self) -> &dyn World {
        self
    }
}

#[derive(Serialize)]
pub struct CompletionItem {
    pub label: String,
    /// What to insert; falls back to the label. May contain Typst's `${...}`
    /// snippet placeholders, which the frontend strips.
    pub apply: Option<String>,
    pub detail: Option<String>,
    /// Kind name, so the UI can group or icon them.
    pub kind: &'static str,
}

#[derive(Serialize)]
pub struct Completions {
    /// Where the completion replaces from, as a UTF-16 offset so it can be
    /// used directly by the editor.
    pub from: usize,
    pub items: Vec<CompletionItem>,
}

fn kind_name(kind: &CompletionKind) -> &'static str {
    match kind {
        CompletionKind::Syntax => "syntax",
        CompletionKind::Func => "function",
        CompletionKind::Type => "type",
        CompletionKind::Param => "parameter",
        CompletionKind::Constant => "constant",
        CompletionKind::Path => "path",
        CompletionKind::Package => "package",
        CompletionKind::Label => "label",
        CompletionKind::Font => "font",
        _ => "value",
    }
}

/// Typst indexes source by byte offset; editors count UTF-16 code units. The
/// two agree only while the document is pure ASCII, which for prose it never
/// stays — one em dash or accented name and every offset after it is wrong.
pub fn utf16_to_byte(text: &str, utf16_offset: usize) -> usize {
    let mut utf16 = 0;
    for (byte, ch) in text.char_indices() {
        if utf16 >= utf16_offset {
            return byte;
        }
        utf16 += ch.len_utf16();
    }
    text.len()
}

pub fn byte_to_utf16(text: &str, byte_offset: usize) -> usize {
    text.get(..byte_offset).unwrap_or(text).encode_utf16().count()
}

/// Completions for `cursor` (a UTF-16 offset) in the world's current text.
///
/// `document` is the last successful compile, which lets Typst suggest things
/// it can only know from a laid-out document, such as existing labels.
pub fn completions_at(
    world: &AmsWorld,
    document: Option<&PagedDocument>,
    cursor: usize,
) -> Completions {
    let source = world.main_source();
    let text = source.text();
    let byte_cursor = utf16_to_byte(text, cursor);

    let Some((from, completions)) =
        typst_ide::autocomplete(world, document, source, byte_cursor, true)
    else {
        return Completions { from: cursor, items: Vec::new() };
    };

    Completions {
        from: byte_to_utf16(text, from),
        items: completions.iter().map(to_item).collect(),
    }
}

fn to_item(completion: &Completion) -> CompletionItem {
    CompletionItem {
        label: completion.label.to_string(),
        apply: completion.apply.as_ref().map(|a| a.to_string()),
        detail: completion.detail.as_ref().map(|d| d.to_string()),
        kind: kind_name(&completion.kind),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_agree_on_ascii() {
        let text = "hello world";
        assert_eq!(utf16_to_byte(text, 6), 6);
        assert_eq!(byte_to_utf16(text, 6), 6);
    }

    #[test]
    fn offsets_survive_non_ascii_prose() {
        // "café — " is where byte and UTF-16 offsets part ways: é is 2 bytes
        // and 1 unit, the em dash is 3 bytes and 1 unit.
        let text = "café — naïve";
        let utf16_of_n = text.encode_utf16().count() - "naïve".encode_utf16().count();

        let byte_of_n = utf16_to_byte(text, utf16_of_n);

        assert_eq!(&text[byte_of_n..], "naïve");
        assert_eq!(byte_to_utf16(text, byte_of_n), utf16_of_n);
    }

    #[test]
    fn offsets_past_the_end_clamp_instead_of_panicking() {
        let text = "short";
        assert_eq!(utf16_to_byte(text, 999), text.len());
        assert_eq!(byte_to_utf16(text, 999), text.encode_utf16().count());
    }

    #[test]
    fn completing_a_hash_offers_typst_functions() {
        let world = AmsWorld::detached("#");
        let completions = completions_at(&world, None, 1);

        assert!(!completions.items.is_empty(), "typing # should offer the language's functions");
        assert!(
            completions.items.iter().any(|item| item.label == "text"),
            "expected the `text` function among completions"
        );
    }
}
