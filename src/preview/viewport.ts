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
  let observer: IntersectionObserver | null = null;

  function setPages(pages: PageGeometry[]) {
    observer?.disconnect();
    container.innerHTML = "";

    const pixelPerPt = window.devicePixelRatio || 1;
    // ponytail: once a page is rendered it stays rendered — no eviction of
    // off-screen canvases. Fine for the documents this app targets; if very
    // long documents make memory a real problem, evict canvases that scroll
    // far out of view and re-render on return.
    const rendered = new Set<number>();

    observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          const index = Number((entry.target as HTMLElement).dataset.pageIndex);
          if (rendered.has(index)) continue;
          rendered.add(index);

          const canvas = entry.target.querySelector("canvas");
          if (!(canvas instanceof HTMLCanvasElement)) continue;
          renderPageOnto(canvas, index, pixelPerPt).catch((err) => {
            console.error(`page ${index} failed to render`, err);
            rendered.delete(index);
          });
        }
      },
      { root: container, rootMargin: "200px 0px" },
    );

    for (const [index, page] of pages.entries()) {
      const wrapper = document.createElement("div");
      wrapper.className = "preview-page";
      wrapper.dataset.pageIndex = String(index);
      wrapper.style.width = `${page.width_pt}px`;
      wrapper.style.height = `${page.height_pt}px`;

      wrapper.appendChild(document.createElement("canvas"));
      container.appendChild(wrapper);
      observer.observe(wrapper);
    }
  }

  return { setPages };
}
