// Renders one compiled page onto a canvas. Task 11 decides *when* to call
// this (on compile) and *which* pages (viewport.ts decides that visibility).

import { invoke } from "@tauri-apps/api/core";

interface RenderedPage {
  width: number;
  height: number;
  png_base64: string;
}

function base64ToBlob(base64: string): Blob {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return new Blob([bytes], { type: "image/png" });
}

export async function renderPageOnto(canvas: HTMLCanvasElement, index: number, pixelPerPt: number): Promise<void> {
  const result = await invoke<RenderedPage>("render_page", { index, pixelPerPt });
  const bitmap = await createImageBitmap(base64ToBlob(result.png_base64));
  canvas.width = result.width;
  canvas.height = result.height;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("2D canvas context unavailable");
  ctx.drawImage(bitmap, 0, 0);
}
