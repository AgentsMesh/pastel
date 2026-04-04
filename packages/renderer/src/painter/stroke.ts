// ============================================================
// Stroke Painter — border rendering
// ============================================================

import type { IrStroke } from '../types.js';
import { drawRoundedRect } from './fill.js';

/**
 * Draw a stroke around a rectangle with optional corner radius.
 */
export function paintStroke(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  stroke: IrStroke | undefined,
  cornerRadius?: [number, number, number, number],
): void {
  if (!stroke || stroke.width <= 0) return;

  ctx.strokeStyle = stroke.color;
  ctx.lineWidth = stroke.width;

  if (cornerRadius && cornerRadius.some(r => r > 0)) {
    drawRoundedRect(ctx, x, y, w, h, cornerRadius);
    ctx.stroke();
  } else {
    ctx.strokeRect(x, y, w, h);
  }
}
