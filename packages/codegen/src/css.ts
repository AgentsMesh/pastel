import type {
  IrDocument,
  IrNode,
  IrDimension,
  IrLayout,
} from "./types.js";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/** Generate plain CSS from an IR document. */
export function generateCss(ir: IrDocument): string {
  const rules: string[] = [];

  // Canvas root rule
  const canvasCls = sanitizeClassName(ir.canvas.name);
  rules.push(buildRule(`.${canvasCls}`, [
    `width: ${ir.canvas.width}px`,
    `height: ${ir.canvas.height}px`,
    ir.canvas.background ? `background: ${ir.canvas.background}` : undefined,
  ]));

  for (const node of ir.nodes) {
    collectRules(node, rules);
  }

  return rules.join("\n\n") + "\n";
}

// ---------------------------------------------------------------------------
// Rule collection (recursive)
// ---------------------------------------------------------------------------

function collectRules(node: IrNode, rules: string[]): void {
  const cls = sanitizeClassName(node.type !== 'text' ? (node.name ?? node.id) : node.id);
  const declarations = nodeDeclarations(node);

  if (declarations.length > 0) {
    rules.push(buildRule(`.${cls}`, declarations));
  }

  const children = node.children ?? [];
  for (const child of children) {
    collectRules(child, rules);
  }
}

function nodeDeclarations(node: IrNode): string[] {
  switch (node.type) {
    case "frame":
      return frameDeclarations(node);
    case "shape":
      return shapeDeclarations(node);
    case "text":
      return textDeclarations(node);
    case "image":
      return imageDeclarations(node);
    default:
      return [];
  }
}

// ---------------------------------------------------------------------------
// Declaration builders
// ---------------------------------------------------------------------------

function frameDeclarations(node: IrNode & { type: "frame" }): string[] {
  const decls: (string | undefined)[] = [
    ...dimensionDeclarations(node.width, node.height),
    ...paddingDeclarations(node.padding),
    ...layoutDeclarations(node.layout),
    ...fillDeclarations(node.fill),
    ...strokeDeclarations(node.stroke),
    ...cornerRadiusDeclarations(node.corner_radius),
    ...shadowDeclarations(node.shadow),
    ...opacityDeclarations(node.opacity),
  ];
  return decls.filter((d): d is string => d != null);
}

function shapeDeclarations(node: IrNode & { type: "shape" }): string[] {
  const decls: (string | undefined)[] = [
    ...dimensionDeclarations(node.width, node.height),
    ...fillDeclarations(node.fill),
    ...strokeDeclarations(node.stroke),
    ...cornerRadiusDeclarations(node.corner_radius),
    ...shadowDeclarations(node.shadow),
    ...opacityDeclarations(node.opacity),
  ];
  return decls.filter((d): d is string => d != null);
}

function textDeclarations(node: IrNode & { type: "text" }): string[] {
  const decls: (string | undefined)[] = [
    node.font_size != null ? `font-size: ${node.font_size}px` : undefined,
    node.font_weight ? `font-weight: ${node.font_weight}` : undefined,
    node.font_family ? `font-family: '${node.font_family}'` : undefined,
    node.color ? `color: ${node.color}` : undefined,
    node.text_align ? `text-align: ${node.text_align}` : undefined,
  ];
  return decls.filter((d): d is string => d != null);
}

function imageDeclarations(node: IrNode & { type: "image" }): string[] {
  const decls: (string | undefined)[] = [
    ...dimensionDeclarations(node.width, node.height),
    node.fit ? `object-fit: ${node.fit}` : undefined,
  ];
  return decls.filter((d): d is string => d != null);
}

function dimensionDeclarations(w?: IrDimension, h?: IrDimension): (string | undefined)[] {
  return [
    w ? dimensionDecl("width", w) : undefined,
    h ? dimensionDecl("height", h) : undefined,
  ];
}

function dimensionDecl(prop: string, dim: IrDimension): string {
  if (dim.type === 'number') return `${prop}: ${dim.value}px`;
  if (dim.type === 'fill') return `${prop}: 100%`;
  if (dim.type === 'hug') return `${prop}: fit-content`;
  return `${prop}: auto`;
}

function paddingDeclarations(padding?: [number, number, number, number]): (string | undefined)[] {
  if (!padding) return [];
  const [top, right, bottom, left] = padding;

  if (top === right && right === bottom && bottom === left) {
    return top !== 0 ? [`padding: ${top}px`] : [];
  }

  return [`padding: ${top}px ${right}px ${bottom}px ${left}px`];
}

function layoutDeclarations(layout?: IrLayout): (string | undefined)[] {
  if (!layout) return [];
  const decls: (string | undefined)[] = [];

  if (layout.mode === "horizontal" || layout.mode === "vertical") {
    decls.push("display: flex");
    if (layout.mode === "vertical") {
      decls.push("flex-direction: column");
    }
  }

  if (layout.gap != null && layout.gap !== 0) {
    decls.push(`gap: ${layout.gap}px`);
  }

  if (layout.align) {
    decls.push(`align-items: ${layout.align}`);
  }

  if (layout.justify) {
    decls.push(`justify-content: ${layout.justify}`);
  }

  return decls;
}

function fillDeclarations(fill?: { type: string; color?: string }): (string | undefined)[] {
  if (!fill) return [];
  if (fill.type === "solid" && fill.color) {
    return [`background: ${fill.color}`];
  }
  return ["background: transparent"];
}

function strokeDeclarations(stroke?: { width: number; color: string }): (string | undefined)[] {
  if (!stroke) return [];
  return [`border: ${stroke.width}px solid ${stroke.color}`];
}

function cornerRadiusDeclarations(cr?: [number, number, number, number]): (string | undefined)[] {
  if (!cr) return [];
  const [tl, tr, br, bl] = cr;

  if (tl === tr && tr === br && br === bl) {
    return tl !== 0 ? [`border-radius: ${tl}px`] : [];
  }

  return [`border-radius: ${tl}px ${tr}px ${br}px ${bl}px`];
}

function shadowDeclarations(shadow?: { x: number; y: number; blur: number; color: string }): (string | undefined)[] {
  if (!shadow) return [];
  const { x, y, blur, color } = shadow;
  return [`box-shadow: ${x}px ${y}px ${blur}px ${color}`];
}

function opacityDeclarations(opacity?: number): (string | undefined)[] {
  if (opacity == null || opacity === 1) return [];
  return [`opacity: ${opacity}`];
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

function buildRule(selector: string, declarations: (string | undefined)[]): string {
  const filtered = declarations.filter((d): d is string => d != null);
  const body = filtered.map((d) => `  ${d};`).join("\n");
  return `${selector} {\n${body}\n}`;
}

/** Convert arbitrary names to valid CSS class names. */
function sanitizeClassName(name: string): string {
  return name
    .replace(/[^a-zA-Z0-9_-]/g, "-")
    .replace(/^-+/, "")
    .replace(/-+$/, "")
    .replace(/-{2,}/g, "-")
    .toLowerCase();
}
