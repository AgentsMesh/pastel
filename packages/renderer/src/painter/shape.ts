// ============================================================
// Shape Painter — rectangles and ellipses
// ============================================================

import { drawRoundedRect } from './fill.js';

/**
 * Draw a rectangle path (without filling or stroking).
 */
export function drawRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  cornerRadius?: [number, number, number, number],
): void {
  if (cornerRadius && cornerRadius.some(r => r > 0)) {
    drawRoundedRect(ctx, x, y, w, h, cornerRadius);
  } else {
    ctx.beginPath();
    ctx.rect(x, y, w, h);
    ctx.closePath();
  }
}

/**
 * Draw an ellipse fitting the given bounding box.
 */
export function drawEllipse(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
): void {
  const cx = x + w / 2;
  const cy = y + h / 2;
  ctx.beginPath();
  ctx.ellipse(cx, cy, w / 2, h / 2, 0, 0, Math.PI * 2);
  ctx.closePath();
}
