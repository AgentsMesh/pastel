// ============================================================
// Layout Engine Tests
// ============================================================

import { describe, it, expect } from 'vitest';
import { computeLayout } from '../src/layout.js';
import type { IrDocument, IrNode, TextMeasurer } from '../src/types.js';

/** Fixed-size text measurer for deterministic tests */
const testMeasurer: TextMeasurer = {
  measure(text: string, _font: string) {
    return { width: text.length * 8, height: 16 };
  },
};

function makeDoc(nodes: IrNode[], width = 800, height = 600): IrDocument {
  return {
    version: 1,
    canvas: { name: 'test', width, height, background: '#FFF' },
    assets: [],
    nodes,
  };
}

describe('computeLayout', () => {
  it('should compute horizontal layout with gap', () => {
    const doc = makeDoc([
      {
        id: 'parent',
        type: 'frame',
        width: { type: 'number', value: 300 },
        height: { type: 'number', value: 100 },
        layout: { mode: 'horizontal', gap: 10 },
        children: [
          { id: 'a', type: 'frame', width: { type: 'number', value: 50 }, height: { type: 'number', value: 50 }, children: [] },
          { id: 'b', type: 'frame', width: { type: 'number', value: 50 }, height: { type: 'number', value: 50 }, children: [] },
          { id: 'c', type: 'frame', width: { type: 'number', value: 50 }, height: { type: 'number', value: 50 }, children: [] },
        ],
      },
    ]);

    const result = computeLayout(doc, testMeasurer);

    const parent = result.get('parent')!;
    expect(parent).toEqual({ x: 0, y: 0, width: 300, height: 100 });

    expect(result.get('a')!.x).toBe(0);
    expect(result.get('b')!.x).toBe(60);
    expect(result.get('c')!.x).toBe(120);
  });

  it('should compute vertical layout with gap', () => {
    const doc = makeDoc([
      {
        id: 'parent',
        type: 'frame',
        width: { type: 'number', value: 200 },
        height: { type: 'number', value: 300 },
        layout: { mode: 'vertical', gap: 20 },
        children: [
          { id: 'a', type: 'frame', width: { type: 'number', value: 100 }, height: { type: 'number', value: 40 }, children: [] },
          { id: 'b', type: 'frame', width: { type: 'number', value: 100 }, height: { type: 'number', value: 40 }, children: [] },
        ],
      },
    ]);

    const result = computeLayout(doc, testMeasurer);

    expect(result.get('a')!.y).toBe(0);
    expect(result.get('b')!.y).toBe(60);
  });

  it('should handle nested layouts', () => {
    const doc = makeDoc([
      {
        id: 'outer',
        type: 'frame',
        width: { type: 'number', value: 400 },
        height: { type: 'number', value: 200 },
        layout: { mode: 'horizontal', gap: 10 },
        children: [
          {
            id: 'left',
            type: 'frame',
            width: { type: 'number', value: 150 },
            height: { type: 'number', value: 200 },
            layout: { mode: 'vertical', gap: 5 },
            children: [
              { id: 'left-a', type: 'frame', width: { type: 'number', value: 100 }, height: { type: 'number', value: 30 }, children: [] },
              { id: 'left-b', type: 'frame', width: { type: 'number', value: 100 }, height: { type: 'number', value: 30 }, children: [] },
            ],
          },
          { id: 'right', type: 'frame', width: { type: 'number', value: 100 }, height: { type: 'number', value: 100 }, children: [] },
        ],
      },
    ]);

    const result = computeLayout(doc, testMeasurer);

    expect(result.get('left')!.x).toBe(0);
    expect(result.get('right')!.x).toBe(160);
    expect(result.get('left-a')!.y).toBe(0);
    expect(result.get('left-b')!.y).toBe(35);
  });

  it('should apply padding', () => {
    const doc = makeDoc([
      {
        id: 'parent',
        type: 'frame',
        width: { type: 'number', value: 200 },
        height: { type: 'number', value: 200 },
        padding: [10, 20, 10, 20],
        layout: { mode: 'horizontal' },
        children: [
          { id: 'child', type: 'frame', width: { type: 'number', value: 50 }, height: { type: 'number', value: 50 }, children: [] },
        ],
      },
    ]);

    const result = computeLayout(doc, testMeasurer);
    const child = result.get('child')!;
    expect(child.x).toBe(20);
    expect(child.y).toBe(10);
  });

  it('should align children to center on cross axis', () => {
    const doc = makeDoc([
      {
        id: 'parent',
        type: 'frame',
        width: { type: 'number', value: 300 },
        height: { type: 'number', value: 100 },
        layout: { mode: 'horizontal', align: 'center' },
        children: [
          { id: 'child', type: 'frame', width: { type: 'number', value: 50 }, height: { type: 'number', value: 30 }, children: [] },
        ],
      },
    ]);

    const result = computeLayout(doc, testMeasurer);
    expect(result.get('child')!.y).toBe(35);
  });

  it('should align children to end on cross axis', () => {
    const doc = makeDoc([
      {
        id: 'parent',
        type: 'frame',
        width: { type: 'number', value: 300 },
        height: { type: 'number', value: 100 },
        layout: { mode: 'horizontal', align: 'end' },
        children: [
          { id: 'child', type: 'frame', width: { type: 'number', value: 50 }, height: { type: 'number', value: 30 }, children: [] },
        ],
      },
    ]);

    const result = computeLayout(doc, testMeasurer);
    expect(result.get('child')!.y).toBe(70);
  });

  it('should justify children with space-between', () => {
    const doc = makeDoc([
      {
        id: 'parent',
        type: 'frame',
        width: { type: 'number', value: 300 },
        height: { type: 'number', value: 100 },
        layout: { mode: 'horizontal', justify: 'space-between' },
        children: [
          { id: 'a', type: 'frame', width: { type: 'number', value: 50 }, height: { type: 'number', value: 50 }, children: [] },
          { id: 'b', type: 'frame', width: { type: 'number', value: 50 }, height: { type: 'number', value: 50 }, children: [] },
          { id: 'c', type: 'frame', width: { type: 'number', value: 50 }, height: { type: 'number', value: 50 }, children: [] },
        ],
      },
    ]);

    const result = computeLayout(doc, testMeasurer);
    expect(result.get('a')!.x).toBe(0);
    expect(result.get('b')!.x).toBe(125);
    expect(result.get('c')!.x).toBe(250);
  });

  it('should expand fill dimension to available space', () => {
    const doc = makeDoc([
      {
        id: 'parent',
        type: 'frame',
        width: { type: 'number', value: 300 },
        height: { type: 'number', value: 100 },
        layout: { mode: 'horizontal', gap: 0 },
        children: [
          { id: 'fixed', type: 'frame', width: { type: 'number', value: 100 }, height: { type: 'number', value: 50 }, children: [] },
          { id: 'flexible', type: 'frame', width: { type: 'fill' }, height: { type: 'number', value: 50 }, children: [] },
        ],
      },
    ]);

    const result = computeLayout(doc, testMeasurer);
    const flexible = result.get('flexible')!;
    expect(flexible.width).toBe(200);
    expect(flexible.x).toBe(100);
  });

  it('should respect fixed dimensions', () => {
    const doc = makeDoc([
      { id: 'box', type: 'frame', width: { type: 'number', value: 123 }, height: { type: 'number', value: 456 }, children: [] },
    ]);

    const result = computeLayout(doc, testMeasurer);
    const box = result.get('box')!;
    expect(box.width).toBe(123);
    expect(box.height).toBe(456);
  });
});
