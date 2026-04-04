import { describe, it, expect } from "vitest";
import { generateJsx, toComponentName } from "../src/jsx.js";
import type { IrDocument, IrNode } from "../src/types.js";
import { helloWorldIr, landingPageIr } from "./fixtures.js";

// Helper: build a minimal IR doc around a single node
function wrapNode(node: IrNode, canvasName = "test"): IrDocument {
  return {
    version: 1,
    canvas: { name: canvasName, width: 400, height: 300, background: "#FFFFFF" },
    assets: [],
    nodes: [node],
  };
}

describe("generateJsx", () => {
  describe("frame -> div conversion", () => {
    it("renders a simple frame as a div with dimension classes", () => {
      const ir = wrapNode({
        id: "box",
        type: "frame",
        width: { type: "number", value: 200 },
        height: { type: "number", value: 100 },
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("<div");
      expect(jsx).toContain("w-[200px]");
      expect(jsx).toContain("h-[100px]");
    });

    it("renders fill dimension as w-full", () => {
      const ir = wrapNode({
        id: "fill-box",
        type: "frame",
        width: { type: "fill" },
        height: { type: "fill" },
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("w-full");
      expect(jsx).toContain("h-full");
    });

    it("renders hug dimension as w-fit", () => {
      const ir = wrapNode({
        id: "hug-box",
        type: "frame",
        width: { type: "hug" },
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("w-fit");
    });

    it("renders an empty frame as self-closing div", () => {
      const ir = wrapNode({ id: "empty", type: "frame", children: [] });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("<div />");
    });
  });

  describe("text -> span conversion", () => {
    it("renders text node as span with Tailwind classes", () => {
      const ir = wrapNode({
        id: "txt",
        type: "text",
        content: "Hello",
        font_size: 16,
        font_weight: "bold",
        color: "#333333",
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("<span");
      expect(jsx).toContain("text-[16px]");
      expect(jsx).toContain("font-bold");
      expect(jsx).toContain("text-[#333333]");
      expect(jsx).toContain("Hello");
    });

    it("renders multi-line text as <p>", () => {
      const ir = wrapNode({
        id: "multi",
        type: "text",
        content: "Line one\nLine two",
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("<p");
      expect(jsx).toContain("</p>");
    });

    it("escapes JSX special characters", () => {
      const ir = wrapNode({
        id: "escaped",
        type: "text",
        content: "<script>{alert()}</script>",
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).not.toContain("<script>");
      expect(jsx).toContain("&lt;script&gt;");
    });
  });

  describe("image -> img conversion", () => {
    it("renders image with src and alt", () => {
      const ir = wrapNode({
        id: "img",
        type: "image",
        name: "hero-image",
        width: { type: "number", value: 300 },
        height: { type: "number", value: 200 },
        asset: "hero.png",
        fit: "cover",
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("<img");
      expect(jsx).toContain('src="./assets/hero.png"');
      expect(jsx).toContain('alt="hero-image"');
      expect(jsx).toContain("w-[300px]");
      expect(jsx).toContain("object-cover");
    });
  });

  describe("nested layout", () => {
    it("renders horizontal layout with flex", () => {
      const ir = wrapNode({
        id: "row",
        type: "frame",
        layout: { mode: "horizontal", gap: 16, align: "center" },
        children: [
          { id: "a", type: "text", content: "A", children: [] },
          { id: "b", type: "text", content: "B", children: [] },
        ],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("flex");
      expect(jsx).not.toContain("flex-col");
      expect(jsx).toContain("gap-[16px]");
      expect(jsx).toContain("items-center");
    });

    it("renders vertical layout with flex flex-col", () => {
      const ir = wrapNode({
        id: "col",
        type: "frame",
        layout: { mode: "vertical", gap: 8 },
        children: [
          { id: "a", type: "text", content: "A", children: [] },
        ],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("flex");
      expect(jsx).toContain("flex-col");
      expect(jsx).toContain("gap-[8px]");
    });
  });

  describe("visual properties", () => {
    it("renders fill, padding, corner radius, shadow", () => {
      const ir = wrapNode({
        id: "card",
        type: "frame",
        fill: { type: "solid", color: "#FFFFFF" },
        padding: [16, 24, 16, 24],
        corner_radius: [12, 12, 12, 12],
        shadow: { x: 0, y: 4, blur: 12, color: "#00000020" },
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("bg-[#FFFFFF]");
      expect(jsx).toContain("py-[16px]");
      expect(jsx).toContain("px-[24px]");
      expect(jsx).toContain("rounded-[12px]");
      expect(jsx).toContain("shadow-[0px_4px_12px_#00000020]");
    });

    it("renders stroke as border", () => {
      const ir = wrapNode({
        id: "bordered",
        type: "frame",
        stroke: { width: 1, color: "#CCCCCC" },
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("border-[1px]");
      expect(jsx).toContain("border-[#CCCCCC]");
    });

    it("renders opacity", () => {
      const ir = wrapNode({
        id: "faded",
        type: "frame",
        opacity: 0.5,
        children: [],
      });
      const jsx = generateJsx(ir);
      expect(jsx).toContain("opacity-[0.5]");
    });
  });

  describe("hello-world snapshot", () => {
    it("generates expected JSX output", () => {
      const jsx = generateJsx(helloWorldIr);
      expect(jsx).toContain("function HelloWorld()");
      expect(jsx).toContain("w-full");
      expect(jsx).toContain("h-full");
      expect(jsx).toContain("flex");
      expect(jsx).toContain("flex-col");
      expect(jsx).toContain("gap-[16px]");
      expect(jsx).toContain("items-center");
      expect(jsx).toContain("justify-center");
      expect(jsx).toContain("Hello, Pastel!");
      expect(jsx).toContain("text-[32px]");
      expect(jsx).toContain("font-bold");
      expect(jsx).toContain("Design as Code");
      expect(jsx).toContain("text-[16px]");
    });
  });

  describe("landing page snapshot", () => {
    it("generates a full landing page component", () => {
      const jsx = generateJsx(landingPageIr);
      expect(jsx).toContain("function LandingPage()");
      expect(jsx).toContain("bg-[#F8F9FA]");
      expect(jsx).toContain("shadow-[0px_2px_8px_#00000012]");
      expect(jsx).toContain("justify-between");
      expect(jsx).toContain('src="./assets/logo.svg"');
      expect(jsx).toContain("Features");
      expect(jsx).toContain("Pricing");
      expect(jsx).toContain("Docs");
      expect(jsx).toContain("bg-[#0066FF]");
      expect(jsx).toContain("rounded-[8px]");
      expect(jsx).toContain("Get Started");
      expect(jsx).toContain("text-[72px]");
    });
  });
});

describe("toComponentName", () => {
  it("converts kebab-case to PascalCase", () => {
    expect(toComponentName("landing-page")).toBe("LandingPage");
  });

  it("converts snake_case to PascalCase", () => {
    expect(toComponentName("my_component")).toBe("MyComponent");
  });

  it("handles single word", () => {
    expect(toComponentName("hero")).toBe("Hero");
  });

  it("handles multiple separators", () => {
    expect(toComponentName("my-cool_widget")).toBe("MyCoolWidget");
  });

  it("handles spaces", () => {
    expect(toComponentName("hello world")).toBe("HelloWorld");
  });
});
