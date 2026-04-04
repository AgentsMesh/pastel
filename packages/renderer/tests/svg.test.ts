// ============================================================
// SVG Export Tests
// ============================================================

import { describe, it, expect } from 'vitest';
import { exportSvg } from '../src/export/svg.js';
import type { IrDocument, TextMeasurer } from '../src/types.js';

const testMeasurer: TextMeasurer = {
  measure(text: string, _font: string) {
    return { width: text.length * 8, height: 16 };
  },
};

describe('exportSvg', () => {
  it('should produce valid SVG with correct dimensions', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 1440, height: 900, background: '#F8F9FA' },
      assets: [],
      nodes: [],
    };

    const svg = exportSvg(doc, testMeasurer);

    expect(svg).toContain('<svg xmlns="http://www.w3.org/2000/svg"');
    expect(svg).toContain('width="1440"');
    expect(svg).toContain('height="900"');
    expect(svg).toContain('viewBox="0 0 1440 900"');
    expect(svg).toContain('fill="#F8F9FA"');
    expect(svg).toContain('</svg>');
  });

  it('should render shape rectangle nodes as <rect> elements', () => {
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
          stroke: { width: 2, color: '#000000' },
          children: [],
        },
      ],
    };

    const svg = exportSvg(doc, testMeasurer);

    expect(svg).toContain('<rect');
    expect(svg).toContain('width="100"');
    expect(svg).toContain('height="50"');
    expect(svg).toContain('fill="#FF0000"');
    expect(svg).toContain('stroke="#000000"');
    expect(svg).toContain('stroke-width="2"');
  });

  it('should render text nodes as <text> elements', () => {
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
          children: [],
        },
      ],
    };

    const svg = exportSvg(doc, testMeasurer);

    expect(svg).toContain('<text');
    expect(svg).toContain('Hello World</text>');
    expect(svg).toContain('font-size="16"');
    expect(svg).toContain('font-weight="bold"');
    expect(svg).toContain('font-family="Arial"');
    expect(svg).toContain('fill="#333"');
  });

  it('should render shape ellipse nodes as <ellipse> elements', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'circle',
          type: 'shape',
          shape_type: 'ellipse',
          width: { type: 'number', value: 80 },
          height: { type: 'number', value: 80 },
          fill: { type: 'solid', color: '#00FF00' },
          children: [],
        },
      ],
    };

    const svg = exportSvg(doc, testMeasurer);

    expect(svg).toContain('<ellipse');
    expect(svg).toContain('cx="40"');
    expect(svg).toContain('cy="40"');
    expect(svg).toContain('rx="40"');
    expect(svg).toContain('ry="40"');
    expect(svg).toContain('fill="#00FF00"');
  });

  it('should handle transparent fills', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'ghost',
          type: 'shape',
          shape_type: 'rectangle',
          width: { type: 'number', value: 100 },
          height: { type: 'number', value: 100 },
          fill: { type: 'transparent' },
          children: [],
        },
      ],
    };

    const svg = exportSvg(doc, testMeasurer);

    expect(svg).toContain('fill="none"');
  });

  it('should render corner radius as rx/ry when uniform', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'rounded',
          type: 'shape',
          shape_type: 'rectangle',
          width: { type: 'number', value: 100 },
          height: { type: 'number', value: 100 },
          fill: { type: 'solid', color: '#CCC' },
          corner_radius: [8, 8, 8, 8],
          children: [],
        },
      ],
    };

    const svg = exportSvg(doc, testMeasurer);

    expect(svg).toContain('rx="8"');
    expect(svg).toContain('ry="8"');
  });

  it('should render frames with children as groups', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'card',
          type: 'frame',
          width: { type: 'number', value: 300 },
          height: { type: 'number', value: 200 },
          fill: { type: 'solid', color: '#FFF' },
          layout: { mode: 'vertical' },
          children: [
            {
              id: 'title',
              type: 'text',
              content: 'Card Title',
              font_size: 18,
              children: [],
            },
          ],
        },
      ],
    };

    const svg = exportSvg(doc, testMeasurer);

    expect(svg).toContain('<g');
    expect(svg).toContain('Card Title</text>');
    expect(svg).toContain('</g>');
  });

  it('should escape special XML characters in text', () => {
    const doc: IrDocument = {
      version: 1,
      canvas: { name: 'test', width: 400, height: 300, background: '#FFF' },
      assets: [],
      nodes: [
        {
          id: 'special',
          type: 'text',
          content: 'A < B & C > D',
          children: [],
        },
      ],
    };

    const svg = exportSvg(doc, testMeasurer);

    expect(svg).toContain('A &lt; B &amp; C &gt; D');
    expect(svg).not.toContain('A < B');
  });
});
