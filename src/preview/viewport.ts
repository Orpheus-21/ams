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
  /// Multiplies page size on screen. Pages re-render at the matching
  /// resolution so zooming in sharpens rather than upscales a blurry bitmap.
  setZoom(zoom: number): void;
  zoomBy(factor: number): void;
  resetZoom(): void;
}

const MIN_ZOOM = 0.25;
const MAX_ZOOM = 4;

export function mountPreview(
  container: HTMLElement,
  onRenderError?: (message: string) => void,
): PreviewController {
  const wrappers: HTMLDivElement[] = [];
  const visible = new Set<number>();
  const rendered = new Set<number>();
  const devicePixels = window.devicePixelRatio || 1;

  let geometry: PageGeometry[] = [];
  let zoom = 1;
  let pixelPerPt = devicePixels;
  // Pages are fitted to the pane until the user zooms deliberately. Without
  // this, an A4 page (595pt) simply overflows any pane narrower than that and
  // there is no way to scroll left to see what was cut off.
  let fitToWidth = true;

  const HORIZONTAL_PADDING = 48;

  function widestPagePt(): number {
    return geometry.reduce((widest, page) => Math.max(widest, page.width_pt), 0);
  }

  function fittedZoom(): number {
    const widest = widestPagePt();
    if (widest === 0) return zoom;
    const available = container.clientWidth - HORIZONTAL_PADDING;
    return clampZoom(available / widest);
  }

  function clampZoom(value: number): number {
    return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, value));
  }

  // Refit when the pane is resized by the splitter or the window.
  new ResizeObserver(() => {
    if (!fitToWidth) return;
    applyZoom(fittedZoom());
  }).observe(container);

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
      // A page that silently fails to draw looks identical to a blank page,
      // so this surfaces rather than only reaching the console.
      onRenderError?.(`page ${index + 1} failed to render: ${err}`);
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
      const index = wrappers.length;
      const wrapper = document.createElement("div");
      wrapper.className = "preview-page";
      wrapper.dataset.pageIndex = String(index);
      wrapper.appendChild(document.createElement("canvas"));
      container.appendChild(wrapper);
      observer.observe(wrapper);
      wrappers.push(wrapper);
    }

    // Page numbers are only worth showing once there's more than one page.
    wrappers.forEach((wrapper, index) => {
      wrapper.dataset.pageLabel = pages.length > 1 ? String(index + 1) : "";
    });

    geometry = pages;
    if (fitToWidth) zoom = fittedZoom();
    pixelPerPt = devicePixels * zoom;
    applyGeometry();

    // The document changed, so every page's raster is stale — but only the
    // ones on screen are redrawn now. The rest re-render when scrolled to.
    rendered.clear();
    for (const index of visible) renderIfNeeded(index);
  }

  function applyGeometry() {
    geometry.forEach((page, index) => {
      const wrapper = wrappers[index];
      if (!wrapper) return;
      wrapper.style.width = `${page.width_pt * zoom}px`;
      wrapper.style.height = `${page.height_pt * zoom}px`;
    });
  }

  function applyZoom(next: number) {
    const clamped = clampZoom(next);
    if (clamped === zoom) return;

    zoom = clamped;
    pixelPerPt = devicePixels * zoom;
    applyGeometry();

    // Existing bitmaps are now the wrong resolution for their box, so they get
    // redrawn — but again only where the user can actually see them.
    rendered.clear();
    for (const index of visible) renderIfNeeded(index);
  }

  return {
    setPages,
    setZoom(next: number) {
      fitToWidth = false;
      applyZoom(next);
    },
    zoomBy(factor: number) {
      fitToWidth = false;
      applyZoom(zoom * factor);
    },
    // Reset goes back to fitting the pane rather than to a literal 100%: at
    // this size "fits" is what a reader actually wants from a reset.
    resetZoom() {
      fitToWidth = true;
      applyZoom(fittedZoom());
    },
  };
}
