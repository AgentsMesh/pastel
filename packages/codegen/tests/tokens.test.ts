import { describe, it, expect } from "vitest";
import { generateTokens } from "../src/tokens.js";
import type { IrDocument, IrNode } from "../src/types.js";
import { helloWorldIr, landingPageIr } from "./fixtures.js";

function wrapNode(node: IrNode, canvasName = "test"): IrDocument {
  return {
    version: 1,
    canvas: { name: canvasName, width: 400, height: 300 },
    assets: [],
    nodes: [node],
  };
}

describe("generateTokens", () => {
  describe("color extraction", () => {
    it("extracts colors from solid fills", () => {
      const ir = wrapNode({
        id: "box",
        type: "frame",
        fill: { type: "solid", color: "#0066FF" },
        children: [],
      });
      const tokens = generateTokens(ir);
      expect(Object.values(tokens.colors)).toContain("#0066FF");
    });

    it("extracts colors from text nodes", () => {
      const ir = wrapNode({
        id: "txt",
        type: "text",
        content: "Hello",
        color: "#111111",
        children: [],
      });
      const tokens = generateTokens(ir);
      expect(Object.values(tokens.colors)).toContain("#111111");
    });

    it("extracts colors from stroke", () => {
      const ir = wrapNode({
        id: "bordered",
        type: "frame",
        stroke: { width: 1, color: "#CCCCCC" },
        children: [],
      });
      const tokens = generateTokens(ir);
      expect(Object.values(tokens.colors)).toContain("#CCCCCC");
    });

    it("extracts canvas background color", () => {
      const ir: IrDocument = {
        version: 1,
        canvas: { name: "test", width: 400, height: 300, background: "#F0F0F0" },
        assets: [],
        nodes: [],
      };
      const tokens = generateTokens(ir);
      expect(Object.values(tokens.colors)).toContain("#F0F0F0");
    });

    it("deduplicates identical colors", () => {
      const ir = wrapNode({
        id: "parent",
        type: "frame",
        fill: { type: "solid", color: "#0066FF" },
        children: [
          {
            id: "child",
            type: "frame",
            fill: { type: "solid", color: "#0066FF" },
            children: [],
          },
        ],
      });
      const tokens = generateTokens(ir);
      const blueCount = Object.values(tokens.colors).filter((c) => c === "#0066FF").length;
      expect(blueCount).toBe(1);
    });
  });

  describe("typography extraction", () => {
    it("extracts font size and weight from text nodes", () => {
      const ir = wrapNode({
        id: "heading",
        type: "text",
        content: "Title",
        font_size: 24,
        font_weight: "bold",
        children: [],
      });
      const tokens = generateTokens(ir);
      const entries = Object.values(tokens.typography);
      expect(entries).toHaveLength(1);
      expect(entries[0]).toEqual({ fontSize: 24, fontWeight: "bold" });
    });

    it("extracts font family when present", () => {
      const ir = wrapNode({
        id: "txt",
        type: "text",
        content: "Body",
        font_size: 16,
        font_family: "Inter",
        children: [],
      });
      const tokens = generateTokens(ir);
      const entries = Object.values(tokens.typography);
      expect(entries[0]).toEqual({ fontSize: 16, fontFamily: "Inter" });
    });

    it("deduplicates identical typography tokens", () => {
      const ir: IrDocument = {
        version: 1,
        canvas: { name: "test", width: 400, height: 300 },
        assets: [],
        nodes: [
          { id: "t1", type: "text", content: "A", font_size: 16, children: [] },
          { id: "t2", type: "text", content: "B", font_size: 16, children: [] },
        ],
      };
      const tokens = generateTokens(ir);
      expect(Object.keys(tokens.typography)).toHaveLength(1);
    });

    it("does not extract typography from non-text nodes", () => {
      const ir = wrapNode({
        id: "frame",
        type: "frame",
        children: [],
      });
      const tokens = generateTokens(ir);
      expect(Object.keys(tokens.typography)).toHaveLength(0);
    });
  });

  describe("spacing extraction", () => {
    it("extracts gap from layout", () => {
      const ir = wrapNode({
        id: "row",
        type: "frame",
        layout: { mode: "horizontal", gap: 16 },
        children: [],
      });
      const tokens = generateTokens(ir);
      expect(Object.values(tokens.spacing)).toContain(16);
    });

    it("extracts padding values", () => {
      const ir = wrapNode({
        id: "padded",
        type: "frame",
        padding: [8, 16, 8, 16],
        children: [],
      });
      const tokens = generateTokens(ir);
      expect(Object.values(tokens.spacing)).toContain(8);
      expect(Object.values(tokens.spacing)).toContain(16);
    });

    it("ignores zero spacing values", () => {
      const ir = wrapNode({
        id: "no-gap",
        type: "frame",
        padding: [0, 0, 0, 0],
        layout: { mode: "horizontal", gap: 0 },
        children: [],
      });
      const tokens = generateTokens(ir);
      expect(Object.keys(tokens.spacing)).toHaveLength(0);
    });
  });

  describe("radii extraction", () => {
    it("extracts corner radius values", () => {
      const ir = wrapNode({
        id: "rounded",
        type: "frame",
        corner_radius: [8, 8, 8, 8],
        children: [],
      });
      const tokens = generateTokens(ir);
      expect(Object.values(tokens.radii)).toContain(8);
    });

    it("extracts unique radii from mixed corners", () => {
      const ir = wrapNode({
        id: "mixed",
        type: "frame",
        corner_radius: [4, 8, 4, 8],
        children: [],
      });
      const tokens = generateTokens(ir);
      expect(Object.values(tokens.radii)).toContain(4);
      expect(Object.values(tokens.radii)).toContain(8);
    });
  });

  describe("hello-world fixture", () => {
    it("extracts expected tokens from hello-world IR", () => {
      const tokens = generateTokens(helloWorldIr);
      expect(Object.values(tokens.colors)).toContain("#FFFFFF");
      expect(Object.values(tokens.colors)).toContain("#111111");
      expect(Object.values(tokens.colors)).toContain("#666666");
      expect(Object.values(tokens.typography)).toContainEqual({ fontSize: 32, fontWeight: "bold" });
      expect(Object.values(tokens.typography)).toContainEqual({ fontSize: 16 });
      expect(Object.values(tokens.spacing)).toContain(16);
    });
  });

  describe("landing page fixture", () => {
    it("extracts rich tokens from landing page IR", () => {
      const tokens = generateTokens(landingPageIr);
      expect(Object.values(tokens.colors)).toContain("#0066FF");
      expect(Object.values(tokens.colors)).toContain("#FFFFFF");
      const fontSizes = Object.values(tokens.typography).map((t) => t.fontSize);
      expect(fontSizes).toContain(72);
      expect(fontSizes).toContain(20);
      expect(fontSizes).toContain(14);
      expect(Object.values(tokens.radii)).toContain(8);
    });
  });
});
