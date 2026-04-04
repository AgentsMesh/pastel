// ============================================================
// Layout Engine — computes absolute positions from IR tree
// ============================================================

import type {
  IrNode,
  IrDocument,
  LayoutResult,
  Dimension,
  TextMeasurer,
} from './types.js';

const DEFAULT_MEASURER: TextMeasurer = {
  measure(_text: string, _font: string) {
    // Rough fallback: 8px per char, 1.2em line height
    return { width: _text.length * 8, height: 16 };
  },
};

/** Resolve a dimension to a concrete pixel value */
function resolveDimension(
  dim: Dimension | undefined,
  available: number,
  content: number,
): number {
  if (!dim) return content;
  if (dim.type === 'number') return dim.value;
  if (dim.type === 'fill') return available;
  // 'hug'
  return content;
}

/** Get width/height/padding/layout from a flattened node */
function getNodeWidth(node: IrNode): Dimension | undefined {
  if (node.type === 'text') return undefined;
  return node.width;
}

function getNodeHeight(node: IrNode): Dimension | undefined {
  if (node.type === 'text') return undefined;
  return node.height;
}

function getNodePadding(node: IrNode): [number, number, number, number] {
  if (node.type === 'frame' && node.padding) return node.padding;
  return [0, 0, 0, 0];
}

function getNodeLayout(node: IrNode) {
  if (node.type === 'frame') return node.layout;
  return undefined;
}

/** Estimate content size of a single node (text, image placeholder) */
function estimateContentSize(
  node: IrNode,
  measurer: TextMeasurer,
): { width: number; height: number } {
  if (node.type === 'text' && node.content) {
    const fontSize = node.font_size ?? 14;
    const fontWeight = node.font_weight ?? 'normal';
    const fontFamily = node.font_family ?? 'sans-serif';
    const font = `${fontWeight} ${fontSize}px ${fontFamily}`;
    return measurer.measure(node.content, font);
  }
  return { width: 0, height: 0 };
}

/** Compute the content size contributed by children along the layout axis */
function computeChildrenContentSize(
  node: IrNode,
  results: LayoutResult,
): { width: number; height: number } {
  const children = node.children ?? [];
  if (children.length === 0) return { width: 0, height: 0 };

  const layout = getNodeLayout(node);
  const gap = layout?.gap ?? 0;
  const mode = layout?.mode ?? 'vertical';

  let totalMain = 0;
  let maxCross = 0;

  for (const child of children) {
    const rect = results.get(child.id);
    if (!rect) continue;
    if (mode === 'horizontal') {
      totalMain += rect.width;
      maxCross = Math.max(maxCross, rect.height);
    } else {
      totalMain += rect.height;
      maxCross = Math.max(maxCross, rect.width);
    }
  }

  if (children.length > 1) {
    totalMain += gap * (children.length - 1);
  }

  return mode === 'horizontal'
    ? { width: totalMain, height: maxCross }
    : { width: maxCross, height: totalMain };
}

/**
 * Perform layout on the IR document.
 * Returns a map of nodeId -> absolute { x, y, width, height }.
 */
export function computeLayout(
  doc: IrDocument,
  measurer: TextMeasurer = DEFAULT_MEASURER,
): LayoutResult {
  const result: LayoutResult = new Map();
  const canvasW = doc.canvas.width;
  const canvasH = doc.canvas.height;

  for (const node of doc.nodes) {
    layoutNode(node, 0, 0, canvasW, canvasH, result, measurer);
  }
  return result;
}

function layoutNode(
  node: IrNode,
  parentX: number,
  parentY: number,
  availableW: number,
  availableH: number,
  result: LayoutResult,
  measurer: TextMeasurer,
): void {
  const padding = getNodePadding(node);
  const [pt, pr, pb, pl] = padding;

  // First pass: layout children so we can compute content size for 'hug'
  const children = node.children ?? [];
  const innerAvailW = availableW - pl - pr;
  const innerAvailH = availableH - pt - pb;

  // Temporarily layout children with available inner space
  layoutChildren(node, 0, 0, innerAvailW, innerAvailH, result, measurer);

  const contentSize = children.length > 0
    ? computeChildrenContentSize(node, result)
    : estimateContentSize(node, measurer);

  const nodeW = resolveDimension(getNodeWidth(node), availableW, contentSize.width + pl + pr);
  const nodeH = resolveDimension(getNodeHeight(node), availableH, contentSize.height + pt + pb);

  result.set(node.id, { x: parentX, y: parentY, width: nodeW, height: nodeH });

  // Re-layout children with resolved node size
  const resolvedInnerW = nodeW - pl - pr;
  const resolvedInnerH = nodeH - pt - pb;
  layoutChildren(node, parentX + pl, parentY + pt, resolvedInnerW, resolvedInnerH, result, measurer);
}

function layoutChildren(
  parent: IrNode,
  offsetX: number,
  offsetY: number,
  availableW: number,
  availableH: number,
  result: LayoutResult,
  measurer: TextMeasurer,
): void {
  const children = parent.children ?? [];
  if (children.length === 0) return;

  const layout = getNodeLayout(parent);
  const mode = layout?.mode ?? 'vertical';
  const gap = layout?.gap ?? 0;
  const align = layout?.align ?? 'start';
  const justify = layout?.justify ?? 'start';

  // First: compute each child's intrinsic / resolved size
  const childSizes: { w: number; h: number }[] = [];
  let fillCount = 0;
  let fixedMain = 0;

  for (const child of children) {
    const childContentSize = estimateContentSize(child, measurer);
    const childW = getNodeWidth(child);
    const childH = getNodeHeight(child);
    const cw = resolveDimensionForChild(childW, mode === 'horizontal' ? 0 : availableW, childContentSize.width);
    const ch = resolveDimensionForChild(childH, mode === 'vertical' ? 0 : availableH, childContentSize.height);

    const isFillMain = mode === 'horizontal'
      ? (childW?.type === 'fill')
      : (childH?.type === 'fill');

    if (isFillMain) {
      fillCount++;
    } else {
      fixedMain += mode === 'horizontal' ? cw : ch;
    }
    childSizes.push({ w: cw, h: ch });
  }

  const totalGap = gap * (children.length - 1);
  const remainingMain = Math.max(0, (mode === 'horizontal' ? availableW : availableH) - fixedMain - totalGap);
  const fillSize = fillCount > 0 ? remainingMain / fillCount : 0;

  // Resolve fill sizes
  for (let i = 0; i < children.length; i++) {
    const child = children[i];
    const childW = getNodeWidth(child);
    const childH = getNodeHeight(child);
    const isFillMain = mode === 'horizontal'
      ? (childW?.type === 'fill')
      : (childH?.type === 'fill');

    if (isFillMain) {
      if (mode === 'horizontal') childSizes[i].w = fillSize;
      else childSizes[i].h = fillSize;
    }

    // Cross-axis fill
    const isFillCross = mode === 'horizontal'
      ? (childH?.type === 'fill')
      : (childW?.type === 'fill');

    if (isFillCross || align === 'stretch') {
      if (mode === 'horizontal') childSizes[i].h = availableH;
      else childSizes[i].w = availableW;
    }
  }

  // Compute total main axis size
  let totalChildMain = 0;
  for (let i = 0; i < childSizes.length; i++) {
    totalChildMain += mode === 'horizontal' ? childSizes[i].w : childSizes[i].h;
  }
  totalChildMain += totalGap;

  const mainSpace = (mode === 'horizontal' ? availableW : availableH) - totalChildMain;

  // Justify: compute starting offset and per-gap extra
  let mainCursor = 0;
  let extraGap = 0;

  switch (justify) {
    case 'start':
      mainCursor = 0;
      break;
    case 'center':
      mainCursor = mainSpace / 2;
      break;
    case 'end':
      mainCursor = mainSpace;
      break;
    case 'space-between':
      mainCursor = 0;
      extraGap = children.length > 1 ? mainSpace / (children.length - 1) : 0;
      break;
    case 'space-around':
      extraGap = children.length > 0 ? mainSpace / children.length : 0;
      mainCursor = extraGap / 2;
      break;
  }

  for (let i = 0; i < children.length; i++) {
    const child = children[i];
    const size = childSizes[i];
    const crossTotal = mode === 'horizontal' ? availableH : availableW;
    const childCross = mode === 'horizontal' ? size.h : size.w;

    let crossOffset = 0;
    switch (align) {
      case 'start':
        crossOffset = 0;
        break;
      case 'center':
        crossOffset = (crossTotal - childCross) / 2;
        break;
      case 'end':
        crossOffset = crossTotal - childCross;
        break;
      case 'stretch':
        crossOffset = 0;
        break;
    }

    const x = mode === 'horizontal' ? offsetX + mainCursor : offsetX + crossOffset;
    const y = mode === 'horizontal' ? offsetY + crossOffset : offsetY + mainCursor;

    // Layout the child (recursively handles its own children)
    layoutNode(child, x, y, size.w, size.h, result, measurer);

    // Advance cursor
    mainCursor += (mode === 'horizontal' ? size.w : size.h) + gap + extraGap;
  }
}

function resolveDimensionForChild(
  dim: Dimension | undefined,
  available: number,
  content: number,
): number {
  if (!dim) return content;
  if (dim.type === 'number') return dim.value;
  if (dim.type === 'fill') return available; // will be recalculated
  return content; // hug
}
