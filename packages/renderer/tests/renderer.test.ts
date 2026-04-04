// ============================================================
// Renderer Tests — verify Canvas 2D API calls
// ============================================================

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '../src/renderer.js';
import type { IrDocument, TextMeasurer } from '../src/types.js';

/** Create a mock CanvasRenderingContext2D with all required methods */
function createMockCtx(): CanvasRenderingContext2D {
  const ctx: Record<string, unknown> = {
    fillStyle: '',
    strokeStyle: '',
    lineWidth: 0,
    font: '',
    textAlign: 'left',
    textBaseline: 'top',
    globalAlpha: 1,
    shadowColor: '',
    shadowBlur: 0,
    shadowOffsetX: 0,
    shadowOffsetY: 0,
    clearRect: vi.fn(),
    fillRect: vi.fn(),
    strokeRect: vi.fn(),
    fillText: vi.fn(),
    measureText: vi.fn(() => ({
      width: 50,
      actualBoundingBoxAscent: 10,
      actualBoundingBoxDescent: 3,
    })),
    beginPath: vi.fn(),
    closePath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    arcTo: vi.fn(),
    ellipse: vi.fn(),
    fill: vi.fn(),
    stroke: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
  };
  return ctx as unknown as CanvasRenderingContext2D;
}

const testMeasurer: TextMeasurer = {
  measure(text: string, _font: string) {
    return { width: text.length * 8, height: 16 };
  },
};

describe('render', () => {
  let ctx: CanvasRenderingContext2D;

  beforeEach(() => {
    ctx = createMockCtx();
  });

  it('should clear canvas and draw background', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 800, height: 600, background: '#F8F9FA' },
      assets: [],
      nodes: [],
    };

    render(ctx, doc, testMeasurer);

    expect(ctx.clearRect).toHaveBeenCalledWith(0, 0, 800, 600);
    expect(ctx.fillRect).toHaveBeenCalledWith(0, 0, 800, 600);
    expect(ctx.fillStyle).toBe('#F8F9FA');
  });

  it('should render a solid fill rectangle (shape)', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'box',
          type: 'shape',
          shape_type: 'rectangle',
          width: { type: 'number', value: 100 },
          height: { type: 'number', value: 50 },
          fill: { type: 'solid', color: '#FF0000' },
        },
      ],
    };

    render(ctx, doc, testMeasurer);

    expect(ctx.fillRect).toHaveBeenCalledTimes(2);
    expect(ctx.fillRect).toHaveBeenCalledWith(0, 0, 100, 50);
  });

  it('should render text with correct properties', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'label',
          type: 'text',
          content: 'Hello World',
          font_size: 16,
          font_weight: 'bold',
          font_family: 'Arial',
          color: '#333',
          text_align: 'center',
        },
      ],
    };

    render(ctx, doc, testMeasurer);

    expect(ctx.fillText).toHaveBeenCalled();
    expect(ctx.font).toBe('bold 16px Arial');
  });

  it('should apply opacity to nodes', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'faded',
          type: 'shape',
          shape_type: 'rectangle',
          width: { type: 'number', value: 100 },
          height: { type: 'number', value: 100 },
          fill: { type: 'solid', color: '#00F' },
          opacity: 0.5,
        },
      ],
    };

    render(ctx, doc, testMeasurer);

    expect(ctx.save).toHaveBeenCalled();
    expect(ctx.restore).toHaveBeenCalled();
  });

  it('should render an image placeholder', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [{ id: 'logo', type: 'image', path: './logo.png' }],
      nodes: [
        {
          id: 'img',
          type: 'image',
          asset: 'logo',
          width: { type: 'number', value: 120 },
          height: { type: 'number', value: 80 },
        },
      ],
    };

    render(ctx, doc, testMeasurer);

    expect(ctx.fillRect).toHaveBeenCalled();
    expect(ctx.strokeRect).toHaveBeenCalled();
    expect(ctx.beginPath).toHaveBeenCalled();
    expect(ctx.stroke).toHaveBeenCalled();
  });

  it('should render a frame with children', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'frame',
          type: 'frame',
          width: { type: 'number', value: 300 },
          height: { type: 'number', value: 200 },
          fill: { type: 'solid', color: '#EEE' },
          layout: { mode: 'vertical', gap: 10 },
          children: [
            {
              id: 'title',
              type: 'text',
              content: 'Title',
              font_size: 18,
              color: '#111',
            },
            {
              id: 'body',
              type: 'text',
              content: 'Body text',
              font_size: 14,
              color: '#666',
            },
          ],
        },
      ],
    };

    render(ctx, doc, testMeasurer);

    expect(ctx.fillText).toHaveBeenCalledTimes(2);
    expect(ctx.save).toHaveBeenCalledTimes(3);
    expect(ctx.restore).toHaveBeenCalledTimes(3);
  });

  it('should render a stroke', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'bordered',
          type: 'shape',
          shape_type: 'rectangle',
          width: { type: 'number', value: 100 },
          height: { type: 'number', value: 100 },
          fill: { type: 'solid', color: '#FFF' },
          stroke: { width: 2, color: '#000' },
        },
      ],
    };

    render(ctx, doc, testMeasurer);

    expect(ctx.strokeRect).toHaveBeenCalledWith(0, 0, 100, 100);
  });
});
