// Draggable divider between the editor and preview panes. Kept deliberately
// small: a pointer-events drag that rewrites one CSS custom property, no
// library and no layout framework.

const MIN_FRACTION = 0.2;
const MAX_FRACTION = 0.8;

export function mountSplitter(divider: HTMLElement, container: HTMLElement = document.body): void {
  divider.addEventListener("pointerdown", (event) => {
    event.preventDefault();
    divider.setPointerCapture(event.pointerId);

    const onMove = (move: PointerEvent) => {
      const bounds = container.getBoundingClientRect();
      const fraction = (move.clientX - bounds.left) / bounds.width;
      const clamped = Math.min(MAX_FRACTION, Math.max(MIN_FRACTION, fraction));
      container.style.setProperty("--editor-fraction", String(clamped));
    };

    const onUp = () => {
      divider.releasePointerCapture(event.pointerId);
      divider.removeEventListener("pointermove", onMove);
      divider.removeEventListener("pointerup", onUp);
    };

    divider.addEventListener("pointermove", onMove);
    divider.addEventListener("pointerup", onUp);
  });
}
