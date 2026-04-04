// ============================================================
// Effects Painter — shadow and opacity
// ============================================================

import type { IrShadow } from '../types.js';

/**
 * Apply shadow settings to the canvas context.
 * Returns a cleanup function to restore previous state.
 */
export function applyShadow(
  ctx: CanvasRenderingContext2D,
  shadow: IrShadow | undefined,
): () => void {
  if (!shadow) return () => {};

  const prev = {
    shadowColor: ctx.shadowColor,
    shadowBlur: ctx.shadowBlur,
    shadowOffsetX: ctx.shadowOffsetX,
    shadowOffsetY: ctx.shadowOffsetY,
  };

  ctx.shadowColor = shadow.color;
  ctx.shadowBlur = shadow.blur;
  ctx.shadowOffsetX = shadow.x;
  ctx.shadowOffsetY = shadow.y;

  return () => {
    ctx.shadowColor = prev.shadowColor;
    ctx.shadowBlur = prev.shadowBlur;
    ctx.shadowOffsetX = prev.shadowOffsetX;
    ctx.shadowOffsetY = prev.shadowOffsetY;
  };
}

/**
 * Apply opacity to the canvas context.
 * Returns a cleanup function to restore previous state.
 */
export function applyOpacity(
  ctx: CanvasRenderingContext2D,
  opacity: number | undefined,
): () => void {
  if (opacity === undefined || opacity >= 1.0) return () => {};

  const prev = ctx.globalAlpha;
  ctx.globalAlpha = opacity;

  return () => {
    ctx.globalAlpha = prev;
  };
}
