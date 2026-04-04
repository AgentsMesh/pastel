import { useRef, useEffect, useCallback } from 'react';

interface EditorProps {
  source: string;
  onChange: (source: string) => void;
  onCompiled: (ir: unknown) => void;
  onError: (error: string) => void;
}

/**
 * Simple code editor with .pastel syntax highlighting.
 * Uses a plain textarea for now — CodeMirror integration in future.
 */
export function Editor({ source, onChange, onCompiled, onError }: EditorProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const compileTimeoutRef = useRef<ReturnType<typeof setTimeout>>();

  // Simple client-side "compiler" that parses to a basic IR
  // In production this would call pastel-lang WASM
  const compileSource = useCallback(
    (src: string) => {
      try {
        const ir = parseToIr(src);
        onCompiled(ir);
      } catch (e: unknown) {
        onError(e instanceof Error ? e.message : String(e));
      }
    },
    [onCompiled, onError]
  );

  // Debounced compile on change
  useEffect(() => {
    if (compileTimeoutRef.current) {
      clearTimeout(compileTimeoutRef.current);
    }
    compileTimeoutRef.current = setTimeout(() => {
      compileSource(source);
    }, 150);
    return () => {
      if (compileTimeoutRef.current) clearTimeout(compileTimeoutRef.current);
    };
  }, [source, compileSource]);

  // Initial compile
  useEffect(() => {
    compileSource(source);
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Tab key inserts spaces
    if (e.key === 'Tab') {
      e.preventDefault();
      const target = e.currentTarget;
      const start = target.selectionStart;
      const end = target.selectionEnd;
      const newValue = source.substring(0, start) + '    ' + source.substring(end);
      onChange(newValue);
      requestAnimationFrame(() => {
        target.selectionStart = target.selectionEnd = start + 4;
      });
    }
  };

  return (
    <div style={styles.container}>
      <textarea
        ref={textareaRef}
        value={source}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        style={styles.textarea}
        spellCheck={false}
      />
    </div>
  );
}

/**
 * Minimal client-side .pastel parser for live preview.
 * This is a simplified version — the real compiler runs in Rust/WASM.
 */
function parseToIr(source: string): unknown {
  const lines = source.split('\n');
  let canvasName = 'untitled';
  let canvasWidth = 400;
  let canvasHeight = 300;
  let canvasBackground: string | null = null;

  // Extract canvas info
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('canvas')) {
      const nameMatch = trimmed.match(/canvas\s+"([^"]+)"/);
      if (nameMatch) canvasName = nameMatch[1];
    }
    if (trimmed.startsWith('width')) {
      const m = trimmed.match(/width\s*=\s*(\d+)/);
      if (m) canvasWidth = parseInt(m[1]);
    }
    if (trimmed.startsWith('height')) {
      const m = trimmed.match(/height\s*=\s*(\d+)/);
      if (m) canvasHeight = parseInt(m[1]);
    }
    if (trimmed.startsWith('background')) {
      const m = trimmed.match(/background\s*=\s*(#[0-9A-Fa-f]+)/);
      if (m) canvasBackground = m[1];
    }
  }

  // Build a simple node tree from indentation-based parsing
  const nodes = parseNodes(lines);

  return {
    version: 1,
    canvas: {
      name: canvasName,
      width: canvasWidth,
      height: canvasHeight,
      background: canvasBackground,
    },
    assets: [],
    nodes,
  };
}

interface ParsedNode {
  id: string;
  type: string;
  name?: string;
  props: Record<string, unknown>;
  layout?: Record<string, unknown>;
  children: ParsedNode[];
}

function parseNodes(lines: string[]): ParsedNode[] {
  const nodes: ParsedNode[] = [];
  let idCounter = 0;

  const nodeStack: { node: ParsedNode; indent: number }[] = [];
  let currentNode: ParsedNode | null = null;
  let inCanvas = false;

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('//')) continue;

    const indent = line.length - line.trimStart().length;

    // Skip canvas, asset, let, include blocks
    if (trimmed.startsWith('canvas ')) { inCanvas = true; continue; }
    if (trimmed.startsWith('asset ') || trimmed.startsWith('let ') || trimmed.startsWith('include ')) continue;
    if (inCanvas && trimmed === '}') { inCanvas = false; continue; }
    if (inCanvas) continue;

    // Node declarations
    const frameMatch = trimmed.match(/^frame\s+([a-zA-Z_][\w-]*)\s*\{/);
    const textMatch = trimmed.match(/^text\s+"([^"]+)"\s*\{/);
    const imageMatch = trimmed.match(/^image\s+([a-zA-Z_][\w-]*)\s*\{/);

    if (frameMatch || textMatch || imageMatch) {
      const node: ParsedNode = {
        id: '',
        type: '',
        props: {},
        children: [],
      };

      if (frameMatch) {
        node.type = 'frame';
        node.name = frameMatch[1];
        node.id = frameMatch[1];
      } else if (textMatch) {
        idCounter++;
        node.type = 'text';
        node.id = `text_${idCounter}`;
        node.props.content = textMatch[1];
      } else if (imageMatch) {
        node.type = 'image';
        node.name = imageMatch[1];
        node.id = imageMatch[1];
        node.props.asset = imageMatch[1];
      }

      // Find parent based on indent
      while (nodeStack.length > 0 && nodeStack[nodeStack.length - 1].indent >= indent) {
        nodeStack.pop();
      }

      if (nodeStack.length > 0) {
        nodeStack[nodeStack.length - 1].node.children.push(node);
      } else {
        nodes.push(node);
      }

      nodeStack.push({ node, indent });
      currentNode = node;
      continue;
    }

    // Inline text: text "label" { size = 14, color = #111 }
    const inlineTextMatch = trimmed.match(/^text\s+"([^"]+)"\s*\{\s*(.+?)\s*\}$/);
    if (inlineTextMatch) {
      idCounter++;
      const node: ParsedNode = {
        id: `text_${idCounter}`,
        type: 'text',
        props: { content: inlineTextMatch[1] },
        children: [],
      };
      parseInlineAttrs(inlineTextMatch[2], node);

      while (nodeStack.length > 0 && nodeStack[nodeStack.length - 1].indent >= indent) {
        nodeStack.pop();
      }
      if (nodeStack.length > 0) {
        nodeStack[nodeStack.length - 1].node.children.push(node);
      } else {
        nodes.push(node);
      }
      continue;
    }

    // Closing brace
    if (trimmed === '}') {
      while (nodeStack.length > 0 && nodeStack[nodeStack.length - 1].indent >= indent) {
        nodeStack.pop();
      }
      currentNode = nodeStack.length > 0 ? nodeStack[nodeStack.length - 1].node : null;
      continue;
    }

    // Attribute: key = value
    if (currentNode) {
      const attrMatch = trimmed.match(/^(\w+)\s*=\s*(.+?)(?:,\s*)?$/);
      if (attrMatch) {
        applyAttr(currentNode, attrMatch[1], attrMatch[2].trim());
      }
    }
  }

  return nodes;
}

function parseInlineAttrs(attrStr: string, node: ParsedNode): void {
  const parts = attrStr.split(',');
  for (const part of parts) {
    const m = part.trim().match(/(\w+)\s*=\s*(.+)/);
    if (m) {
      applyAttr(node, m[1], m[2].trim());
    }
  }
}

function applyAttr(node: ParsedNode, key: string, rawValue: string): void {
  const layoutKeys = ['layout', 'gap', 'align', 'justify'];

  if (layoutKeys.includes(key)) {
    if (!node.layout) node.layout = {};
    if (key === 'layout') node.layout.mode = rawValue;
    else if (key === 'gap') node.layout.gap = parseFloat(rawValue);
    else node.layout[key] = rawValue;
    return;
  }

  switch (key) {
    case 'width':
    case 'height':
      if (rawValue === 'fill' || rawValue === 'hug') {
        node.props[key] = { type: 'keyword', value: rawValue };
      } else {
        node.props[key] = { type: 'number', value: parseFloat(rawValue) };
      }
      break;
    case 'fill':
      if (rawValue === 'transparent') {
        node.props.fill = { type: 'transparent' };
      } else if (rawValue.startsWith('#')) {
        node.props.fill = { type: 'solid', color: rawValue };
      }
      break;
    case 'color':
      node.props.color = rawValue.startsWith('#') ? rawValue : rawValue;
      break;
    case 'size':
      node.props.font_size = parseFloat(rawValue);
      break;
    case 'weight':
      node.props.font_weight = rawValue;
      break;
    case 'font':
      node.props.font_family = rawValue.replace(/"/g, '');
      break;
    case 'radius':
      if (rawValue.startsWith('[')) {
        node.props.corner_radius = JSON.parse(rawValue);
      } else {
        const r = parseFloat(rawValue);
        node.props.corner_radius = [r, r, r, r];
      }
      break;
    case 'padding':
      if (rawValue.startsWith('[')) {
        const arr = JSON.parse(rawValue);
        if (arr.length === 2) node.props.padding = [arr[0], arr[1], arr[0], arr[1]];
        else if (arr.length === 4) node.props.padding = arr;
        else node.props.padding = [arr[0], arr[0], arr[0], arr[0]];
      } else {
        const v = parseFloat(rawValue);
        node.props.padding = [v, v, v, v];
      }
      break;
    case 'shadow':
      // skip for now in client parser
      break;
    case 'stroke':
      // skip for now
      break;
    case 'opacity':
      node.props.opacity = parseFloat(rawValue);
      break;
    case 'fit':
      node.props.fit = rawValue;
      break;
    case 'content':
      node.props.content = rawValue.replace(/"/g, '');
      break;
  }
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    flex: 1,
    overflow: 'hidden',
  },
  textarea: {
    width: '100%',
    height: '100%',
    background: 'var(--bg-secondary)',
    color: 'var(--text-primary)',
    fontFamily: 'var(--font-mono)',
    fontSize: 13,
    lineHeight: 1.6,
    padding: 16,
    border: 'none',
    outline: 'none',
    resize: 'none',
    tabSize: 4,
  },
};
