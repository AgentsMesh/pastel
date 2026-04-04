import type {
  IrDocument,
  IrNode,
  IrDimension,
  IrLayout,
} from "./types.js";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/** Generate a React + Tailwind JSX component from an IR document. */
export function generateJsx(ir: IrDocument): string {
  const componentName = toComponentName(ir.canvas.name);
  const { width, height, background } = ir.canvas;

  const rootClasses = buildClassList([
    dimensionClass("w", { type: "number", value: width }),
    dimensionClass("h", { type: "number", value: height }),
    background ? `bg-[${background}]` : undefined,
  ]);

  const childrenJsx = ir.nodes.map((n) => renderNode(n, 2)).join("\n");

  return [
    `export default function ${componentName}() {`,
    `  return (`,
    `    <div${classAttr(rootClasses)}>`,
    childrenJsx,
    `    </div>`,
    `  );`,
    `}`,
    ``,
  ].join("\n");
}

// ---------------------------------------------------------------------------
// Node rendering
// ---------------------------------------------------------------------------

function renderNode(node: IrNode, depth: number): string {
  const indent = "  ".repeat(depth);

  switch (node.type) {
    case "text":
      return renderText(node, indent);
    case "image":
      return renderImage(node, indent);
    case "frame":
    case "shape":
    default:
      return renderFrame(node, indent, depth);
  }
}

function renderText(node: IrNode & { type: "text" }, indent: string): string {
  const classes = buildClassList(textClasses(node));
  const content = escapeJsx(node.content ?? "");

  const isMultiLine = content.includes("\n");
  const tag = isMultiLine ? "p" : "span";
  return `${indent}<${tag}${classAttr(classes)}>${content}</${tag}>`;
}

function renderImage(node: IrNode & { type: "image" }, indent: string): string {
  const classes = buildClassList([
    ...dimensionClasses(node),
    node.fit ? `object-${node.fit}` : undefined,
  ]);
  const src = node.asset ? `./assets/${node.asset}` : "";
  const alt = node.name ?? "image";
  return `${indent}<img src="${src}"${classAttr(classes)} alt="${alt}" />`;
}

function renderFrame(node: IrNode, indent: string, depth: number): string {
  const classes = buildClassList(frameClasses(node));
  const children = node.children ?? [];

  if (children.length === 0) {
    return `${indent}<div${classAttr(classes)} />`;
  }

  const inner = children.map((c) => renderNode(c, depth + 1)).join("\n");
  return [
    `${indent}<div${classAttr(classes)}>`,
    inner,
    `${indent}</div>`,
  ].join("\n");
}

// ---------------------------------------------------------------------------
// Tailwind class builders
// ---------------------------------------------------------------------------

function frameClasses(node: IrNode): (string | undefined)[] {
  if (node.type === 'text') return [];

  const layout = node.type === 'frame' ? node.layout : undefined;

  return [
    ...dimensionClasses(node),
    ...paddingClasses(node),
    ...layoutClasses(layout),
    ...fillClasses(node),
    ...borderClasses(node),
    ...cornerRadiusClasses(node),
    ...shadowClasses(node),
    ...opacityClasses(node),
  ];
}

function textClasses(node: IrNode & { type: "text" }): (string | undefined)[] {
  return [
    node.font_size ? `text-[${node.font_size}px]` : undefined,
    fontWeightClass(node.font_weight),
    node.color ? `text-[${node.color}]` : undefined,
    node.font_family ? `font-['${node.font_family}']` : undefined,
    textAlignClass(node.text_align),
  ];
}

function dimensionClasses(node: IrNode): (string | undefined)[] {
  if (node.type === 'text') return [];
  const width = node.width;
  const height = node.height;
  return [
    width ? dimensionClass("w", width) : undefined,
    height ? dimensionClass("h", height) : undefined,
  ];
}

function dimensionClass(prefix: "w" | "h", dim: IrDimension): string {
  if (dim.type === 'number') return `${prefix}-[${dim.value}px]`;
  if (dim.type === 'fill') return `${prefix}-full`;
  if (dim.type === 'hug') return `${prefix}-fit`;
  return `${prefix}-auto`;
}

function paddingClasses(node: IrNode): (string | undefined)[] {
  if (node.type !== 'frame' || !node.padding) return [];
  const [top, right, bottom, left] = node.padding;

  if (top === right && right === bottom && bottom === left) {
    return top !== 0 ? [`p-[${top}px]`] : [];
  }

  if (top === bottom && left === right) {
    return [
      top !== 0 ? `py-[${top}px]` : undefined,
      left !== 0 ? `px-[${left}px]` : undefined,
    ];
  }

  return [
    top !== 0 ? `pt-[${top}px]` : undefined,
    right !== 0 ? `pr-[${right}px]` : undefined,
    bottom !== 0 ? `pb-[${bottom}px]` : undefined,
    left !== 0 ? `pl-[${left}px]` : undefined,
  ];
}

function layoutClasses(layout?: IrLayout): (string | undefined)[] {
  if (!layout) return [];

  const classes: (string | undefined)[] = [];

  if (layout.mode === "horizontal") {
    classes.push("flex");
  } else if (layout.mode === "vertical") {
    classes.push("flex", "flex-col");
  }

  if (layout.gap != null && layout.gap !== 0) {
    classes.push(`gap-[${layout.gap}px]`);
  }

  if (layout.align) {
    classes.push(alignClass(layout.align));
  }

  if (layout.justify) {
    classes.push(justifyClass(layout.justify));
  }

  return classes;
}

function alignClass(align: string): string {
  const map: Record<string, string> = {
    start: "items-start",
    center: "items-center",
    end: "items-end",
    stretch: "items-stretch",
  };
  return map[align] ?? `items-${align}`;
}

function justifyClass(justify: string): string {
  const map: Record<string, string> = {
    start: "justify-start",
    center: "justify-center",
    end: "justify-end",
    "space-between": "justify-between",
    "space-around": "justify-around",
    "space-evenly": "justify-evenly",
  };
  return map[justify] ?? `justify-${justify}`;
}

function fillClasses(node: IrNode): (string | undefined)[] {
  if (node.type === 'text') return [];
  const fill = (node.type === 'frame' || node.type === 'shape') ? node.fill : undefined;
  if (!fill) return [];
  if (fill.type === "solid") {
    return [`bg-[${fill.color}]`];
  }
  return [];
}

function borderClasses(node: IrNode): (string | undefined)[] {
  if (node.type === 'text') return [];
  const stroke = (node.type === 'frame' || node.type === 'shape') ? node.stroke : undefined;
  if (!stroke) return [];
  return [
    `border-[${stroke.width}px]`,
    `border-[${stroke.color}]`,
  ];
}

function cornerRadiusClasses(node: IrNode): (string | undefined)[] {
  if (node.type === 'text') return [];
  const cr = node.corner_radius;
  if (!cr) return [];
  const [tl, tr, br, bl] = cr;

  if (tl === tr && tr === br && br === bl) {
    return tl !== 0 ? [`rounded-[${tl}px]`] : [];
  }

  return [
    tl !== 0 ? `rounded-tl-[${tl}px]` : undefined,
    tr !== 0 ? `rounded-tr-[${tr}px]` : undefined,
    br !== 0 ? `rounded-br-[${br}px]` : undefined,
    bl !== 0 ? `rounded-bl-[${bl}px]` : undefined,
  ];
}

function shadowClasses(node: IrNode): (string | undefined)[] {
  if (node.type === 'text') return [];
  const shadow = (node.type === 'frame' || node.type === 'shape' || node.type === 'image') ? node.shadow : undefined;
  if (!shadow) return [];
  const { x, y, blur, color } = shadow;
  return [`shadow-[${x}px_${y}px_${blur}px_${color}]`];
}

function opacityClasses(node: IrNode): (string | undefined)[] {
  if (node.type === 'text') return [];
  const opacity = (node.type === 'frame' || node.type === 'shape' || node.type === 'image') ? node.opacity : undefined;
  if (opacity == null || opacity === 1) return [];
  return [`opacity-[${opacity}]`];
}

function fontWeightClass(weight?: string): string | undefined {
  if (!weight) return undefined;
  const map: Record<string, string> = {
    thin: "font-thin",
    extralight: "font-extralight",
    light: "font-light",
    normal: "font-normal",
    medium: "font-medium",
    semibold: "font-semibold",
    bold: "font-bold",
    extrabold: "font-extrabold",
    black: "font-black",
  };
  return map[weight.toLowerCase()] ?? `font-[${weight}]`;
}

function textAlignClass(align?: string): string | undefined {
  if (!align) return undefined;
  return `text-${align}`;
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

function buildClassList(classes: (string | undefined)[]): string {
  return classes.filter(Boolean).join(" ");
}

function classAttr(classes: string): string {
  return classes ? ` className="${classes}"` : "";
}

/** Convert a kebab/snake-case name to PascalCase. */
export function toComponentName(name: string): string {
  return name
    .split(/[-_\s]+/)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1).toLowerCase())
    .join("");
}

function escapeJsx(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/{/g, "&#123;")
    .replace(/}/g, "&#125;");
}
