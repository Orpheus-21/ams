//! The default, redistributable font set bundled into the app so a document
//! renders correctly with zero system fonts and zero network access.
//!
//! The fonts are the ones the Typst CLI itself ships (`typst-assets`,
//! embedded at compile time via `typst-kit`'s `embedded-fonts` feature):
//! Libertinus Serif (default text font), New Computer Modern + New Computer
//! Modern Math (default math font), and DejaVu Sans Mono (default code
//! font) — all under redistributable open licenses. See
//! `assets/fonts/THIRD_PARTY_LICENSES.md` for the exact licenses.

use typst_kit::fonts::FontStore;

/// Builds a font store seeded with the bundled default font set.
pub fn default_fonts() -> FontStore {
    let mut fonts = FontStore::new();
    fonts.extend(typst_kit::fonts::embedded());
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_text_and_math_fonts() {
        let fonts = default_fonts();
        let book = fonts.book();

        // `FontBook::select` expects an all-lowercase family name.
        let text = book.select("libertinus serif", Default::default());
        assert!(text.is_some(), "default text font should be embedded");

        let math = book.select("new computer modern math", Default::default());
        assert!(math.is_some(), "default math font should be embedded");
    }
}
