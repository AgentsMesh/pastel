// ============================================================
// Fill Painter — solid color fills with corner radius
// ============================================================

import type { Fill } from '../types.js';

/**
 * Draw a filled rectangle with optional corner radius.
 * Handles solid and transparent fill types.
 */
export function paintFill(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  fill: Fill | undefined,
  cornerRadius?: [number, number, number, number],
): void {
  if (!fill || fill.type === 'transparent') return;

  ctx.fillStyle = fill.color;

  if (cornerRadius && cornerRadius.some(r => r > 0)) {
    drawRoundedRect(ctx, x, y, w, h, cornerRadius);
    ctx.fill();
  } else {
    ctx.fillRect(x, y, w, h);
  }
}

/** Draw a path for a rounded rectangle */
export function drawRoundedRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  radii: [number, number, number, number],
): void {
  const [tl, tr, br, bl] = radii;
  ctx.beginPath();
  ctx.moveTo(x + tl, y);
  ctx.lineTo(x + w - tr, y);
  ctx.arcTo(x + w, y, x + w, y + tr, tr);
  ctx.lineTo(x + w, y + h - br);
  ctx.arcTo(x + w, y + h, x + w - br, y + h, br);
  ctx.lineTo(x + bl, y + h);
  ctx.arcTo(x, y + h, x, y + h - bl, bl);
  ctx.lineTo(x, y + tl);
  ctx.arcTo(x, y, x + tl, y, tl);
  ctx.closePath();
}
