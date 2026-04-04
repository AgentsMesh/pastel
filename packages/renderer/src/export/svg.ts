// ============================================================
// SVG Export — generate SVG string from IR document
// ============================================================

import type { IrDocument, IrNode, LayoutResult, TextMeasurer } from '../types.js';
import { computeLayout } from '../layout.js';

/**
 * Export an IR document to an SVG string.
 */
export function exportSvg(
  doc: IrDocument,
  measurer?: TextMeasurer,
): string {
  const layout = computeLayout(doc, measurer);
  const { width, height, background } = doc.canvas;

  const lines: string[] = [];
  lines.push(`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">`);
  lines.push(`  <rect width="${width}" height="${height}" fill="${escapeAttr(background ?? '#FFFFFF')}" />`);

  for (const node of doc.nodes) {
    renderSvgNode(node, layout, lines, '  ');
  }

  lines.push('</svg>');
  return lines.join('\n');
}

function renderSvgNode(
  node: IrNode,
  layout: LayoutResult,
  lines: string[],
  indent: string,
): void {
  const rect = layout.get(node.id);
  if (!rect) return;

  const { x, y, width, height } = rect;
  const hasChildren = node.children && node.children.length > 0;

  switch (node.type) {
    case 'frame': {
      const opacityAttr = buildOpacityAttr(node.opacity);
      const shadowAttr = node.shadow ? buildShadowAttr(node.shadow) : '';
      if (hasChildren) {
        lines.push(`${indent}<g${opacityAttr}${shadowAttr}>`);
        lines.push(`${indent}  <rect x="${x}" y="${y}" width="${width}" height="${height}"${buildFillStrokeAttrs(node.fill, node.stroke)} />`);
        for (const child of node.children!) {
          renderSvgNode(child, layout, lines, indent + '  ');
        }
        lines.push(`${indent}</g>`);
      } else {
        lines.push(`${indent}<rect x="${x}" y="${y}" width="${width}" height="${height}"${buildFillStrokeAttrs(node.fill, node.stroke)}${buildCornerAttr(node.corner_radius)} />`);
      }
      break;
    }

    case 'shape': {
      if (node.shape_type === 'ellipse') {
        const cx = x + width / 2;
        const cy = y + height / 2;
        const rx = width / 2;
        const ry = height / 2;
        let attrs = `cx="${cx}" cy="${cy}" rx="${rx}" ry="${ry}"`;
        attrs += buildFillStrokeAttrs(node.fill, node.stroke);
        attrs += buildOpacityAttr(node.opacity);
        lines.push(`${indent}<ellipse ${attrs} />`);
      } else {
        lines.push(`${indent}<rect x="${x}" y="${y}" width="${width}" height="${height}"${buildFillStrokeAttrs(node.fill, node.stroke)}${buildCornerAttr(node.corner_radius)} />`);
      }
      break;
    }

    case 'text': {
      const content = node.content ?? '';
      const fontSize = node.font_size ?? 14;
      const fontWeight = node.font_weight ?? 'normal';
      const fontFamily = node.font_family ?? 'sans-serif';
      const color = node.color ?? '#000000';
      const textAlign = node.text_align ?? 'left';

      let textX = x;
      let anchor = 'start';
      if (textAlign === 'center') {
        textX = x + width / 2;
        anchor = 'middle';
      } else if (textAlign === 'right') {
        textX = x + width;
        anchor = 'end';
      }

      const textY = y + height / 2;

      let attrs = `x="${textX}" y="${textY}"`;
      attrs += ` font-size="${fontSize}" font-weight="${fontWeight}" font-family="${escapeAttr(fontFamily)}"`;
      attrs += ` fill="${escapeAttr(color)}" text-anchor="${anchor}" dominant-baseline="central"`;
      lines.push(`${indent}<text ${attrs}>${escapeXml(content)}</text>`);
      break;
    }

    case 'image': {
      let attrs = `x="${x}" y="${y}" width="${width}" height="${height}"`;
      attrs += ` fill="#E0E0E0" stroke="#BDBDBD"`;
      lines.push(`${indent}<rect ${attrs} />`);
      break;
    }
  }
}

function buildFillStrokeAttrs(fill: { type: string; color?: string } | undefined, stroke: { width: number; color: string } | undefined): string {
  let attrs = '';
  if (fill) {
    if (fill.type === 'solid' && fill.color) {
      attrs += ` fill="${escapeAttr(fill.color)}"`;
    } else {
      attrs += ` fill="none"`;
    }
  } else {
    attrs += ` fill="none"`;
  }
  if (stroke) {
    attrs += ` stroke="${escapeAttr(stroke.color)}" stroke-width="${stroke.width}"`;
  }
  return attrs;
}

function buildCornerAttr(cornerRadius?: [number, number, number, number]): string {
  if (!cornerRadius) return '';
  const [tl, tr, br, bl] = cornerRadius;
  if (tl === tr && tr === br && br === bl && tl > 0) {
    return ` rx="${tl}" ry="${tl}"`;
  }
  return '';
}

function buildOpacityAttr(opacity?: number): string {
  if (opacity !== undefined && opacity < 1.0) {
    return ` opacity="${opacity}"`;
  }
  return '';
}

function buildShadowAttr(s: { x: number; y: number; blur: number; color: string }): string {
  return ` data-shadow="${s.x},${s.y},${s.blur},${escapeAttr(s.color)}"`;
}

function escapeAttr(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function escapeXml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
