// Virtualized preview pane: lays out one placeholder per page (sized from
// compile-time geometry, cheap even for hundreds of pages), and rasterizes a
// page only once it scrolls into view. This is what satisfies "does not
// rasterize all 500 pages on load" and "scrolling only renders newly-visible
// pages" from tasks/plan.md's Task 10.

import { renderPageOnto } from "./render";

export interface PageGeometry {
  width_pt: number;
  height_pt: number;
}

export interface PreviewController {
  setPages(pages: PageGeometry[]): void;
}

export function mountPreview(container: HTMLElement): PreviewController {
  const wrappers: HTMLDivElement[] = [];
  const visible = new Set<number>();
  const rendered = new Set<number>();
  const pixelPerPt = window.devicePixelRatio || 1;

  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        const index = Number((entry.target as HTMLElement).dataset.pageIndex);
        if (entry.isIntersecting) {
          visible.add(index);
          renderIfNeeded(index);
        } else {
          visible.delete(index);
        }
      }
    },
    { root: container, rootMargin: "200px 0px" },
  );

  // ponytail: once a page is rendered it stays rendered until the document
  // recompiles — no eviction of off-screen canvases. Fine for the documents
  // this app targets; if very long documents make memory a real problem,
  // evict canvases that scroll far out of view and re-render on return.
  function renderIfNeeded(index: number) {
    if (rendered.has(index)) return;
    const canvas = wrappers[index]?.querySelector("canvas");
    if (!(canvas instanceof HTMLCanvasElement)) return;

    rendered.add(index);
    renderPageOnto(canvas, index, pixelPerPt).catch((err) => {
      rendered.delete(index);
      console.error(`page ${index} failed to render`, err);
    });
  }

  function setPages(pages: PageGeometry[]) {
    // Grow/shrink the placeholder list in place rather than rebuilding it.
    // Reusing nodes means an already-rendered canvas keeps showing its last
    // content until the new render lands (no flash on every keystroke), and
    // the pane doesn't jump back to the top mid-edit.
    while (wrappers.length > pages.length) {
      const wrapper = wrappers.pop();
      if (!wrapper) break;
      observer.unobserve(wrapper);
      visible.delete(wrappers.length);
      wrapper.remove();
    }

    while (wrappers.length < pages.length) {
      const wrapper = document.createElement("div");
      wrapper.className = "preview-page";
      wrapper.dataset.pageIndex = String(wrappers.length);
      wrapper.appendChild(document.createElement("canvas"));
      container.appendChild(wrapper);
      observer.observe(wrapper);
      wrappers.push(wrapper);
    }

    pages.forEach((page, index) => {
      wrappers[index].style.width = `${page.width_pt}px`;
      wrappers[index].style.height = `${page.height_pt}px`;
    });

    // The document changed, so every page's raster is stale — but only the
    // ones on screen are redrawn now. The rest re-render when scrolled to.
    rendered.clear();
    for (const index of visible) renderIfNeeded(index);
  }

  return { setPages };
}
