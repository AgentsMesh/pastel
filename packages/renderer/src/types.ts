// ============================================================
// IR JSON TypeScript Types — matching the Rust compiler output
// ============================================================
// Node data is flattened via serde: {id, type, name?, ...data, children?}

/** Top-level IR document */
export interface IrDocument {
  version: number;
  canvas: IrCanvas;
  assets: IrAsset[];
  nodes: IrNode[];
}

/** Canvas metadata */
export interface IrCanvas {
  name: string;
  width: number;
  height: number;
  background?: string;
}

/** External asset reference */
export interface IrAsset {
  id: string;
  type: string;
  path: string;
}

// ---- Dimension ----

export interface NumberDimension {
  type: 'number';
  value: number;
}

export interface FillDimension {
  type: 'fill';
}

export interface HugDimension {
  type: 'hug';
}

export type Dimension = NumberDimension | FillDimension | HugDimension;

// ---- Fill ----

export interface SolidFill {
  type: 'solid';
  color: string;
}

export interface TransparentFill {
  type: 'transparent';
}

export type Fill = SolidFill | TransparentFill;

// ---- Stroke ----

export interface IrStroke {
  width: number;
  color: string;
}

// ---- Shadow ----

export interface IrShadow {
  x: number;
  y: number;
  blur: number;
  color: string;
}

// ---- Layout ----

export type LayoutMode = 'horizontal' | 'vertical';
export type Align = 'start' | 'center' | 'end' | 'stretch';
export type Justify = 'start' | 'center' | 'end' | 'space-between' | 'space-around';

export interface IrLayout {
  mode: LayoutMode;
  gap?: number;
  align?: Align;
  justify?: Justify;
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
  children?: IrNode[];
}

export interface IrFrameNode extends IrNodeBase {
  type: 'frame';
  name?: string;
  width?: Dimension;
  height?: Dimension;
  padding?: [number, number, number, number];
  layout?: IrLayout;
  // visual props (flattened)
  fill?: Fill;
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
  width?: Dimension;
  height?: Dimension;
  corner_radius?: [number, number, number, number];
  shadow?: IrShadow;
  opacity?: number;
  fit?: ImageFit;
}

export interface IrShapeNode extends IrNodeBase {
  type: 'shape';
  name?: string;
  shape_type: ShapeType;
  width?: Dimension;
  height?: Dimension;
  // visual props (flattened)
  fill?: Fill;
  stroke?: IrStroke;
  corner_radius?: [number, number, number, number];
  shadow?: IrShadow;
  opacity?: number;
}

// ---- Layout computation result ----

export interface LayoutRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type LayoutResult = Map<string, LayoutRect>;

// ---- Text measurement abstraction ----

/** Abstraction for measuring text — allows injection in tests */
export interface TextMeasurer {
  measure(text: string, font: string): { width: number; height: number };
}
