// ============================================================
// PNG Export — render to off-screen canvas and export buffer
// ============================================================

import type { IrDocument, TextMeasurer } from '../types.js';

/**
 * Export an IR document to PNG.
 *
 * This is a placeholder — actual implementation requires a Canvas
 * backend (e.g. node-canvas or OffscreenCanvas in browsers).
 * Returns the raw pixel buffer and dimensions.
 */
export interface PngExportResult {
  width: number;
  height: number;
  buffer: Uint8Array | null;
}

export function exportPng(
  _doc: IrDocument,
  _measurer?: TextMeasurer,
): PngExportResult {
  // In a real implementation, we would:
  // 1. Create an OffscreenCanvas / node-canvas
  // 2. Call render(ctx, doc, measurer)
  // 3. Export via canvas.toBuffer('image/png')
  return {
    width: _doc.canvas.width,
    height: _doc.canvas.height,
    buffer: null,
  };
}
