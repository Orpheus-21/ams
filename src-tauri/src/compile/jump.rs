//! Clicking the preview to reach the source that produced it.
//!
//! Without this the preview is a picture: you notice a problem on page 200
//! and then go hunting for it in the text. Typst can map a point on a laid-out
//! page back to the span that produced it, so the preview becomes a way to
//! navigate the document rather than only to look at it.

use typst::layout::Point;
use typst_ide::Jump;
use typst_layout::PagedDocument;

use super::complete::byte_to_utf16;
use super::world::AmsWorld;

/// Where in the source a click on the preview landed, as a UTF-16 offset.
///
/// `x_pt`/`y_pt` are in Typst points relative to the page's top-left corner.
/// Returns `None` when the click doesn't correspond to source in this document
/// — empty margin, a link out to a URL, or content from a package.
pub fn jump_from_preview(
    world: &AmsWorld,
    document: &PagedDocument,
    page_index: usize,
    x_pt: f64,
    y_pt: f64,
) -> Option<usize> {
    let page = document.pages().get(page_index)?;
    let click = Point::new(typst::layout::Abs::pt(x_pt), typst::layout::Abs::pt(y_pt));

    match typst_ide::jump_from_click_in_frame(world, document, &page.frame, click)? {
        Jump::File(id, byte) if id == world.main_id() => {
            Some(byte_to_utf16(world.main_source().text(), byte))
        }
        // A URL, or a span inside an imported package: nothing to move the
        // cursor to in the document the user is editing.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::compile;

    #[test]
    fn clicking_the_second_paragraph_lands_in_the_second_paragraph() {
        // Two well-separated blocks, so a hit can be attributed to one of them
        // rather than being "somewhere in the document".
        let source = "First paragraph here.\n\n#v(6cm)\n\nSecond paragraph here.";
        let needle = source.find("Second").expect("marker exists");

        let world = AmsWorld::detached(source);
        let document = compile(&world).expect("compiles").document;

        // Scan down the page for a point that resolves, rather than guessing a
        // single coordinate and accepting `None` as a pass — a test that can
        // succeed without hitting anything proves nothing.
        let hit = (200..700)
            .step_by(4)
            .filter_map(|y| jump_from_preview(&world, &document, 0, 90.0, y as f64).map(|o| (y, o)))
            .find(|(_, offset)| *offset >= needle);

        let (_, offset) = hit.expect("some point down the page maps into the second paragraph");

        // A click resolves to the glyph under it, so landing mid-word is
        // correct — what matters is that it's inside the right paragraph.
        let paragraph = needle..needle + "Second paragraph here.".len();
        assert!(
            paragraph.contains(&offset),
            "expected an offset within {paragraph:?}, got {offset} ({:?})",
            &source[offset..(offset + 10).min(source.len())]
        );
    }

    #[test]
    fn clicking_far_outside_the_page_reports_nothing() {
        let world = AmsWorld::detached("= Heading");
        let document = compile(&world).expect("compiles").document;

        assert_eq!(jump_from_preview(&world, &document, 0, 5000.0, 5000.0), None);
    }

    #[test]
    fn a_page_index_past_the_end_reports_nothing_rather_than_panicking() {
        let world = AmsWorld::detached("= Heading");
        let document = compile(&world).expect("compiles").document;

        assert_eq!(jump_from_preview(&world, &document, 99, 80.0, 90.0), None);
    }
}
