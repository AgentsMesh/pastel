// ============================================================
// Main Renderer — traverses IR tree and draws to Canvas 2D
// ============================================================

import type { IrDocument, IrNode, LayoutResult, TextMeasurer } from './types.js';
import { computeLayout } from './layout.js';
import { paintFill } from './painter/fill.js';
import { paintStroke } from './painter/stroke.js';
import { paintText } from './painter/text.js';
import { drawEllipse } from './painter/shape.js';
import { applyShadow, applyOpacity } from './painter/effects.js';
import { paintImagePlaceholder } from './painter/image.js';

/**
 * Render an IR document to a Canvas 2D context.
 */
export function render(
  ctx: CanvasRenderingContext2D,
  doc: IrDocument,
  measurer?: TextMeasurer,
): void {
  // Clear and draw canvas background
  ctx.clearRect(0, 0, doc.canvas.width, doc.canvas.height);
  if (doc.canvas.background) {
    ctx.fillStyle = doc.canvas.background;
    ctx.fillRect(0, 0, doc.canvas.width, doc.canvas.height);
  }

  // Compute layout
  const layoutResult = computeLayout(doc, measurer);

  // Render all top-level nodes
  for (const node of doc.nodes) {
    renderNode(ctx, node, layoutResult);
  }
}

function renderNode(
  ctx: CanvasRenderingContext2D,
  node: IrNode,
  layout: LayoutResult,
): void {
  const rect = layout.get(node.id);
  if (!rect) return;

  const { x, y, width, height } = rect;

  ctx.save();

  // Apply opacity
  const nodeOpacity = node.type === 'text' ? undefined : node.opacity;
  const restoreOpacity = applyOpacity(ctx, nodeOpacity);

  // Apply shadow
  const nodeShadow = (node.type === 'frame' || node.type === 'shape' || node.type === 'image') ? node.shadow : undefined;
  const restoreShadow = applyShadow(ctx, nodeShadow);

  switch (node.type) {
    case 'frame': {
      const cr = node.corner_radius;
      paintFill(ctx, x, y, width, height, node.fill, cr);
      restoreShadow();
      paintStroke(ctx, x, y, width, height, node.stroke, cr);
      break;
    }

    case 'shape': {
      if (node.shape_type === 'ellipse') {
        if (node.fill && node.fill.type === 'solid') {
          ctx.fillStyle = node.fill.color;
          drawEllipse(ctx, x, y, width, height);
          ctx.fill();
        }
        restoreShadow();
        if (node.stroke) {
          ctx.strokeStyle = node.stroke.color;
          ctx.lineWidth = node.stroke.width;
          drawEllipse(ctx, x, y, width, height);
          ctx.stroke();
        }
      } else {
        const cr = node.corner_radius;
        paintFill(ctx, x, y, width, height, node.fill, cr);
        restoreShadow();
        paintStroke(ctx, x, y, width, height, node.stroke, cr);
      }
      break;
    }

    case 'text': {
      restoreShadow();
      paintText(ctx, x, y, width, height, node);
      break;
    }

    case 'image': {
      restoreShadow();
      paintImagePlaceholder(ctx, x, y, width, height);
      break;
    }
  }

  restoreOpacity();

  // Render children
  const children = node.children;
  if (children) {
    for (const child of children) {
      renderNode(ctx, child, layout);
    }
  }

  ctx.restore();
}
