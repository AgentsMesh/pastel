import type { IrDocument, IrNode } from "./types.js";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

export interface DesignTokens {
  colors: Record<string, string>;
  spacing: Record<string, number>;
  typography: Record<string, TypographyToken>;
  radii: Record<string, number>;
}

export interface TypographyToken {
  fontSize: number;
  fontWeight?: string;
  fontFamily?: string;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/** Extract design tokens from an IR document. */
export function generateTokens(ir: IrDocument): DesignTokens {
  const tokens: DesignTokens = {
    colors: {},
    spacing: {},
    typography: {},
    radii: {},
  };

  // Canvas background
  if (ir.canvas.background) {
    tokens.colors["canvas-bg"] = ir.canvas.background;
  }

  for (const node of ir.nodes) {
    extractFromNode(node, tokens);
  }

  return tokens;
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

function extractFromNode(node: IrNode, tokens: DesignTokens): void {
  // Colors from fills
  if (node.type === 'frame' || node.type === 'shape') {
    if (node.fill && node.fill.type === "solid") {
      addColor(tokens, node.fill.color);
    }
    if (node.stroke) {
      addColor(tokens, node.stroke.color);
    }
  }

  // Colors from text
  if (node.type === 'text' && node.color) {
    addColor(tokens, node.color);
  }

  // Typography from text nodes
  if (node.type === 'text' && node.font_size != null) {
    const key = typographyKey(node.font_size, node.font_weight);
    if (!tokens.typography[key]) {
      const token: TypographyToken = { fontSize: node.font_size };
      if (node.font_weight) token.fontWeight = node.font_weight;
      if (node.font_family) token.fontFamily = node.font_family;
      tokens.typography[key] = token;
    }
  }

  // Spacing from gap (frames only)
  if (node.type === 'frame' && node.layout?.gap != null && node.layout.gap !== 0) {
    const key = `gap-${node.layout.gap}`;
    tokens.spacing[key] = node.layout.gap;
  }

  // Spacing from padding (frames only)
  if (node.type === 'frame' && node.padding) {
    for (const value of new Set(node.padding)) {
      if (value !== 0) {
        const key = `padding-${value}`;
        tokens.spacing[key] = value;
      }
    }
  }

  // Corner radii
  if (node.type !== 'text' && node.corner_radius) {
    for (const value of new Set(node.corner_radius)) {
      if (value !== 0) {
        const key = `radius-${value}`;
        tokens.radii[key] = value;
      }
    }
  }

  // Recurse
  const children = node.children ?? [];
  for (const child of children) {
    extractFromNode(child, tokens);
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function addColor(tokens: DesignTokens, hex: string): void {
  const normalized = hex.toUpperCase();
  const key = `color-${normalized.replace("#", "").toLowerCase()}`;
  if (!tokens.colors[key]) {
    tokens.colors[key] = normalized;
  }
}

function typographyKey(fontSize: number, fontWeight?: string): string {
  const parts = [`size-${fontSize}`];
  if (fontWeight) parts.push(fontWeight);
  return parts.join("-");
}
