// ============================================================
// @pastel/renderer — Public API
// ============================================================

export { render } from './renderer.js';
export { computeLayout } from './layout.js';
export { exportSvg } from './export/svg.js';
export { exportPng } from './export/png.js';

// Re-export types
export type {
  IrDocument,
  IrCanvas,
  IrAsset,
  IrNode,
  IrFrameNode,
  IrTextNode,
  IrImageNode,
  IrShapeNode,
  IrLayout,
  IrStroke,
  IrShadow,
  Dimension,
  NumberDimension,
  FillDimension,
  HugDimension,
  Fill,
  SolidFill,
  TransparentFill,
  LayoutMode,
  Align,
  Justify,
  FontWeight,
  TextAlign,
  ImageFit,
  ShapeType,
  LayoutRect,
  LayoutResult,
  TextMeasurer,
} from './types.js';
