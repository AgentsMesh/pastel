import { describe, it, expect } from "vitest";
import { generateCss } from "../src/css.js";
import type { IrDocument, IrNode } from "../src/types.js";
import { helloWorldIr, landingPageIr } from "./fixtures.js";

function wrapNode(node: IrNode, canvasName = "test"): IrDocument {
  return {
    version: 1,
    canvas: { name: canvasName, width: 400, height: 300, background: "#FFFFFF" },
    assets: [],
    nodes: [node],
  };
}

describe("generateCss", () => {
  describe("canvas root", () => {
    it("generates canvas rule with width, height, and background", () => {
      const css = generateCss(helloWorldIr);
      expect(css).toContain(".hello-world {");
      expect(css).toContain("width: 400px;");
      expect(css).toContain("height: 300px;");
      expect(css).toContain("background: #FFFFFF;");
    });
  });

  describe("frame -> CSS flex rules", () => {
    it("generates flex properties for horizontal layout", () => {
      const ir = wrapNode({
        id: "row",
        type: "frame",
        name: "row",
        width: { type: "number", value: 500 },
        height: { type: "number", value: 60 },
        layout: { mode: "horizontal", gap: 16, align: "center", justify: "space-between" },
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain(".row {");
      expect(css).toContain("width: 500px;");
      expect(css).toContain("height: 60px;");
      expect(css).toContain("display: flex;");
      expect(css).toContain("gap: 16px;");
      expect(css).toContain("align-items: center;");
      expect(css).toContain("justify-content: space-between;");
    });

    it("generates flex-direction: column for vertical layout", () => {
      const ir = wrapNode({
        id: "col",
        type: "frame",
        name: "col",
        layout: { mode: "vertical", gap: 8 },
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("display: flex;");
      expect(css).toContain("flex-direction: column;");
      expect(css).toContain("gap: 8px;");
    });

    it("uses fill -> 100% for keyword dimensions", () => {
      const ir = wrapNode({
        id: "fill-frame",
        type: "frame",
        name: "fill-frame",
        width: { type: "fill" },
        height: { type: "fill" },
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("width: 100%;");
      expect(css).toContain("height: 100%;");
    });
  });

  describe("text styles -> CSS font properties", () => {
    it("generates font properties for text nodes", () => {
      const ir = wrapNode({
        id: "heading",
        type: "text",
        content: "Title",
        font_size: 24,
        font_weight: "bold",
        font_family: "Inter",
        color: "#111111",
        text_align: "center",
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain(".heading {");
      expect(css).toContain("font-size: 24px;");
      expect(css).toContain("font-weight: bold;");
      expect(css).toContain("font-family: 'Inter';");
      expect(css).toContain("color: #111111;");
      expect(css).toContain("text-align: center;");
    });
  });

  describe("shadow -> box-shadow", () => {
    it("generates box-shadow from shadow property", () => {
      const ir = wrapNode({
        id: "card",
        type: "frame",
        name: "card",
        shadow: { x: 0, y: 4, blur: 12, color: "#00000020" },
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("box-shadow: 0px 4px 12px #00000020;");
    });
  });

  describe("padding/gap mapping", () => {
    it("generates uniform padding", () => {
      const ir = wrapNode({
        id: "padded",
        type: "frame",
        name: "padded",
        padding: [16, 16, 16, 16],
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("padding: 16px;");
    });

    it("generates 4-value padding for non-uniform values", () => {
      const ir = wrapNode({
        id: "padded-asym",
        type: "frame",
        name: "padded-asym",
        padding: [8, 16, 8, 16],
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("padding: 8px 16px 8px 16px;");
    });
  });

  describe("fill and stroke", () => {
    it("generates background from solid fill", () => {
      const ir = wrapNode({
        id: "filled",
        type: "frame",
        name: "filled",
        fill: { type: "solid", color: "#0066FF" },
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("background: #0066FF;");
    });

    it("generates transparent background", () => {
      const ir = wrapNode({
        id: "transparent",
        type: "frame",
        name: "transparent",
        fill: { type: "transparent" },
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("background: transparent;");
    });

    it("generates border from stroke", () => {
      const ir = wrapNode({
        id: "bordered",
        type: "frame",
        name: "bordered",
        stroke: { width: 2, color: "#CCCCCC" },
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("border: 2px solid #CCCCCC;");
    });
  });

  describe("corner radius", () => {
    it("generates uniform border-radius", () => {
      const ir = wrapNode({
        id: "rounded",
        type: "frame",
        name: "rounded",
        corner_radius: [8, 8, 8, 8],
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("border-radius: 8px;");
    });

    it("generates per-corner border-radius", () => {
      const ir = wrapNode({
        id: "rounded-mixed",
        type: "frame",
        name: "rounded-mixed",
        corner_radius: [4, 8, 12, 16],
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("border-radius: 4px 8px 12px 16px;");
    });
  });

  describe("opacity", () => {
    it("generates opacity declaration", () => {
      const ir = wrapNode({
        id: "faded",
        type: "frame",
        name: "faded",
        opacity: 0.7,
        children: [],
      });
      const css = generateCss(ir);
      expect(css).toContain("opacity: 0.7;");
    });
  });

  describe("nested nodes", () => {
    it("generates rules for nested children", () => {
      const css = generateCss(helloWorldIr);
      expect(css).toContain(".main {");
      expect(css).toContain(".text_1 {");
      expect(css).toContain(".text_2 {");
    });
  });

  describe("landing page", () => {
    it("generates CSS for a full landing page", () => {
      const css = generateCss(landingPageIr);
      expect(css).toContain(".landing-page {");
      expect(css).toContain(".navbar {");
      expect(css).toContain("box-shadow:");
      expect(css).toContain(".hero {");
    });
  });
});
