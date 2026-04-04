// ============================================================
// Text Painter — text rendering via Canvas 2D
// ============================================================

import type { IrTextNode } from '../types.js';

/**
 * Draw text content within a bounding box.
 */
export function paintText(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  node: IrTextNode,
): void {
  if (!node.content) return;

  const fontSize = node.font_size ?? 14;
  const fontWeight = node.font_weight ?? 'normal';
  const fontFamily = node.font_family ?? 'sans-serif';
  const color = node.color ?? '#000000';
  const textAlign = node.text_align ?? 'left';

  ctx.font = `${fontWeight} ${fontSize}px ${fontFamily}`;
  ctx.fillStyle = color;
  ctx.textBaseline = 'top';

  let textX = x;
  if (textAlign === 'center') {
    ctx.textAlign = 'center';
    textX = x + w / 2;
  } else if (textAlign === 'right') {
    ctx.textAlign = 'right';
    textX = x + w;
  } else {
    ctx.textAlign = 'left';
  }

  // Vertically center the text within the box
  const textY = y + (h - fontSize) / 2;

  ctx.fillText(node.content, textX, textY);
}

/**
 * Measure text using the Canvas 2D API.
 * Returns approximate width and height.
 */
export function measureText(
  ctx: CanvasRenderingContext2D,
  text: string,
  font: string,
): { width: number; height: number } {
  ctx.font = font;
  const metrics = ctx.measureText(text);
  const height =
    (metrics.actualBoundingBoxAscent ?? 0) +
    (metrics.actualBoundingBoxDescent ?? 0) ||
    parseInt(font) || 14;
  return { width: metrics.width, height };
}
