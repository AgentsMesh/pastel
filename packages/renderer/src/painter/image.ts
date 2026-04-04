// ============================================================
// Image Painter — placeholder image rendering
// ============================================================

/**
 * Draw an image placeholder (gray box with an "X").
 * Actual async image loading is out of scope for the core renderer.
 */
export function paintImagePlaceholder(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
): void {
  // Background
  ctx.fillStyle = '#E0E0E0';
  ctx.fillRect(x, y, w, h);

  // Border
  ctx.strokeStyle = '#BDBDBD';
  ctx.lineWidth = 1;
  ctx.strokeRect(x, y, w, h);

  // "X" indicator
  ctx.strokeStyle = '#9E9E9E';
  ctx.lineWidth = 2;
  ctx.beginPath();
  const inset = Math.min(w, h) * 0.2;
  ctx.moveTo(x + inset, y + inset);
  ctx.lineTo(x + w - inset, y + h - inset);
  ctx.moveTo(x + w - inset, y + inset);
  ctx.lineTo(x + inset, y + h - inset);
  ctx.stroke();
}
