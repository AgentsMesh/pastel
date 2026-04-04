// ============================================================
// IR JSON TypeScript Types — matching the Rust compiler output
// ============================================================
// Re-implemented to match the flattened serde output from the Rust IR.

/** Top-level IR document emitted by the Pastel compiler. */
export interface IrDocument {
  version: number;
  canvas: IrCanvas;
  assets: IrAsset[];
  nodes: IrNode[];
}

export interface IrCanvas {
  name: string;
  width: number;
  height: number;
  background?: string;
}

export interface IrAsset {
  id: string;
  type: string;
  path: string;
}

// ---- Dimension ----

export type IrDimension =
  | { type: 'number'; value: number }
  | { type: 'fill' }
  | { type: 'hug' };

// ---- Fill ----

export type IrFill =
  | { type: 'solid'; color: string }
  | { type: 'transparent' };

// ---- Stroke / Shadow ----

export interface IrStroke {
  width: number;
  color: string;
}

export interface IrShadow {
  x: number;
  y: number;
  blur: number;
  color: string;
}

// ---- Layout ----

export interface IrLayout {
  mode: 'horizontal' | 'vertical';
  gap?: number;
  align?: 'start' | 'center' | 'end' | 'stretch';
  justify?: 'start' | 'center' | 'end' | 'space-between' | 'space-around';
}

// ---- Font Weight / Text Align / Image Fit ----

export type FontWeight = 'thin' | 'light' | 'normal' | 'medium' | 'semibold' | 'bold' | 'extrabold' | 'black';
export type TextAlign = 'left' | 'center' | 'right';
export type ImageFit = 'cover' | 'contain' | 'fill' | 'none';
export type ShapeType = 'rectangle' | 'ellipse' | 'line';

// ---- Flattened Node (serde flatten) ----

export type IrNode = IrFrameNode | IrTextNode | IrImageNode | IrShapeNode;

interface IrNodeBase {
  id: string;
  children: IrNode[];
}

export interface IrFrameNode extends IrNodeBase {
  type: 'frame';
  name?: string;
  width?: IrDimension;
  height?: IrDimension;
  padding?: [number, number, number, number];
  layout?: IrLayout;
  fill?: IrFill;
  stroke?: IrStroke;
  corner_radius?: [number, number, number, number];
  shadow?: IrShadow;
  opacity?: number;
}

export interface IrTextNode extends IrNodeBase {
  type: 'text';
  content: string;
  font_size?: number;
  font_weight?: FontWeight;
  font_family?: string;
  color?: string;
  text_align?: TextAlign;
  line_height?: number;
}

export interface IrImageNode extends IrNodeBase {
  type: 'image';
  name?: string;
  asset: string;
  width?: IrDimension;
  height?: IrDimension;
  corner_radius?: [number, number, number, number];
  shadow?: IrShadow;
  opacity?: number;
  fit?: ImageFit;
}

export interface IrShapeNode extends IrNodeBase {
  type: 'shape';
  name?: string;
  shape_type: ShapeType;
  width?: IrDimension;
  height?: IrDimension;
  fill?: IrFill;
  stroke?: IrStroke;
  corner_radius?: [number, number, number, number];
  shadow?: IrShadow;
  opacity?: number;
}
