import { useRef, useEffect } from 'react';

interface PreviewProps {
  ir: unknown;
}

export function Preview({ ir }: PreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!ir || !canvasRef.current) return;
    renderToCanvas(canvasRef.current, ir as any);
  }, [ir]);

  return (
    <div ref={containerRef} style={styles.container}>
      <div style={styles.canvasWrapper}>
        <canvas ref={canvasRef} style={styles.canvas} />
      </div>
    </div>
  );
}

function renderToCanvas(canvas: HTMLCanvasElement, ir: any) {
  if (!ir?.canvas) return;

  const { width, height, background } = ir.canvas;
  const dpr = window.devicePixelRatio || 1;

  canvas.width = width * dpr;
  canvas.height = height * dpr;
  canvas.style.width = `${width}px`;
  canvas.style.height = `${height}px`;

  const ctx = canvas.getContext('2d')!;
  ctx.scale(dpr, dpr);

  // Clear + background
  ctx.clearRect(0, 0, width, height);
  if (background) {
    ctx.fillStyle = background;
    ctx.fillRect(0, 0, width, height);
  }

  // Layout + render nodes
  const layoutMap: Record<string, { x: number; y: number; w: number; h: number }> = {};
  layoutChildren(ir.nodes, 0, 0, width, height, null, layoutMap);

  for (const node of ir.nodes) {
    drawNode(ctx, node, layoutMap);
  }
}

function layoutChildren(
  nodes: any[],
  px: number, py: number, pw: number, ph: number,
  parentLayout: any | null,
  map: Record<string, any>
) {
  if (!nodes?.length) return;

  const mode = parentLayout?.mode || 'vertical';
  const gap = parentLayout?.gap || 0;
  const padding = nodes._parentPadding || [0, 0, 0, 0];

  const ix = px + (padding[3] || 0);
  const iy = py + (padding[0] || 0);
  const iw = pw - (padding[1] || 0) - (padding[3] || 0);
  const ih = ph - (padding[0] || 0) - (padding[2] || 0);

  const isH = mode === 'horizontal';
  const sizes = nodes.map((n: any) => ({
    w: dimVal(n.props?.width, iw),
    h: dimVal(n.props?.height, ih),
  }));

  // Text size estimation
  sizes.forEach((s: any, i: number) => {
    const n = nodes[i];
    if (n.type === 'text' && n.props?.content) {
      const fs = n.props.font_size || 14;
      if (!s.w) s.w = n.props.content.length * fs * 0.6;
      if (!s.h) s.h = fs * 1.4;
    }
  });

  const totalGap = gap * Math.max(0, nodes.length - 1);
  const totalMain = sizes.reduce((s: number, d: any) => s + (isH ? d.w : d.h), 0) + totalGap;
  const free = (isH ? iw : ih) - totalMain;
  const justify = parentLayout?.justify || 'start';
  const align = parentLayout?.align || 'start';

  let cx = ix, cy = iy;
  if (justify === 'center') { if (isH) cx += free / 2; else cy += free / 2; }
  if (justify === 'end') { if (isH) cx += free; else cy += free; }

  let spaceBetween = 0;
  if (justify === 'space-between' && nodes.length > 1) {
    spaceBetween = free / (nodes.length - 1);
  }

  nodes.forEach((node: any, i: number) => {
    let { w, h } = sizes[i];
    let nx = cx, ny = cy;

    if (isH) {
      if (align === 'center') ny = iy + (ih - h) / 2;
      else if (align === 'end') ny = iy + ih - h;
    } else {
      if (align === 'center') nx = ix + (iw - w) / 2;
      else if (align === 'end') nx = ix + iw - w;
    }

    map[node.id] = { x: nx, y: ny, w, h };

    if (node.children?.length) {
      (node.children as any)._parentPadding = node.props?.padding || [0, 0, 0, 0];
      layoutChildren(node.children, nx, ny, w, h, node.layout || null, map);
    }

    const gapVal = gap + (justify === 'space-between' ? spaceBetween : 0);
    if (isH) cx += w + gapVal;
    else cy += h + gapVal;
  });
}

function dimVal(dim: any, parentSize: number): number {
  if (!dim) return 0;
  if (typeof dim === 'number') return dim;
  if (dim.type === 'number') return dim.value;
  if (dim.type === 'keyword' && dim.value === 'fill') return parentSize;
  return 0;
}

function drawNode(ctx: CanvasRenderingContext2D, node: any, map: Record<string, any>) {
  const rect = map[node.id];
  if (!rect) return;

  const { x, y, w, h } = rect;
  const props = node.props || {};

  // Shadow
  if (props.shadow) {
    ctx.save();
    ctx.shadowColor = props.shadow.color || 'transparent';
    ctx.shadowBlur = props.shadow.blur || 0;
    ctx.shadowOffsetX = props.shadow.x || 0;
    ctx.shadowOffsetY = props.shadow.y || 0;
  }

  // Fill
  if (props.fill?.type === 'solid') {
    ctx.fillStyle = props.fill.color;
    const r = props.corner_radius;
    if (r?.some((v: number) => v > 0)) {
      drawRoundRect(ctx, x, y, w, h, r);
      ctx.fill();
    } else {
      ctx.fillRect(x, y, w, h);
    }
  }

  if (props.shadow) ctx.restore();

  // Stroke
  if (props.stroke) {
    ctx.strokeStyle = props.stroke.color;
    ctx.lineWidth = props.stroke.width;
    const r = props.corner_radius;
    if (r?.some((v: number) => v > 0)) {
      drawRoundRect(ctx, x, y, w, h, r);
      ctx.stroke();
    } else {
      ctx.strokeRect(x, y, w, h);
    }
  }

  // Text
  if (node.type === 'text' && props.content) {
    const fs = props.font_size || 14;
    const fw = props.font_weight || 'normal';
    const ff = props.font_family || '-apple-system, sans-serif';
    ctx.font = `${fw} ${fs}px ${ff}`;
    ctx.fillStyle = props.color || '#000';
    ctx.textBaseline = 'top';
    const ta = props.text_align || 'left';
    let tx = x;
    if (ta === 'center') { ctx.textAlign = 'center'; tx = x + w / 2; }
    else if (ta === 'right') { ctx.textAlign = 'right'; tx = x + w; }
    else { ctx.textAlign = 'left'; }
    ctx.fillText(props.content, tx, y);
  }

  // Image placeholder
  if (node.type === 'image' && w > 0 && h > 0) {
    ctx.fillStyle = '#f0f0f0';
    const r = props.corner_radius;
    if (r?.some((v: number) => v > 0)) {
      drawRoundRect(ctx, x, y, w, h, r);
      ctx.fill();
    } else {
      ctx.fillRect(x, y, w, h);
    }
    ctx.strokeStyle = '#ddd';
    ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(x, y); ctx.lineTo(x + w, y + h); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(x + w, y); ctx.lineTo(x, y + h); ctx.stroke();
    ctx.fillStyle = '#999';
    ctx.font = '11px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(props.asset || node.name || 'img', x + w / 2, y + h / 2);
  }

  // Children
  if (node.children) {
    for (const child of node.children) {
      drawNode(ctx, child, map);
    }
  }
}

function drawRoundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number[]) {
  const [tl, tr, br, bl] = r;
  ctx.beginPath();
  ctx.moveTo(x + tl, y);
  ctx.lineTo(x + w - tr, y);
  ctx.quadraticCurveTo(x + w, y, x + w, y + tr);
  ctx.lineTo(x + w, y + h - br);
  ctx.quadraticCurveTo(x + w, y + h, x + w - br, y + h);
  ctx.lineTo(x + bl, y + h);
  ctx.quadraticCurveTo(x, y + h, x, y + h - bl);
  ctx.lineTo(x, y + tl);
  ctx.quadraticCurveTo(x, y, x + tl, y);
  ctx.closePath();
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    flex: 1,
    overflow: 'auto',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    background: '#2a2a3e',
    padding: 24,
  },
  canvasWrapper: {
    boxShadow: '0 4px 24px rgba(0,0,0,0.3)',
    borderRadius: 8,
    overflow: 'hidden',
  },
  canvas: {
    display: 'block',
  },
};
