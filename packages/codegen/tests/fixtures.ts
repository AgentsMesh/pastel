import type { IrDocument } from "../src/types.js";

/** Minimal hello-world IR for testing. */
export const helloWorldIr: IrDocument = {
  version: 1,
  canvas: { name: "hello-world", width: 400, height: 300, background: "#FFFFFF" },
  assets: [],
  nodes: [
    {
      id: "main",
      type: "frame",
      name: "main",
      width: { type: "fill" },
      height: { type: "fill" },
      layout: { mode: "vertical", gap: 16.0, align: "center", justify: "center" },
      children: [
        {
          id: "text_1",
          type: "text",
          content: "Hello, Pastel!",
          font_size: 32.0,
          font_weight: "bold",
          color: "#111111",
          children: [],
        },
        {
          id: "text_2",
          type: "text",
          content: "Design as Code",
          font_size: 16.0,
          color: "#666666",
          children: [],
        },
      ],
    },
  ],
};

/** Landing page IR with navbar, hero, and CTA. */
export const landingPageIr: IrDocument = {
  version: 1,
  canvas: { name: "landing-page", width: 1440, height: 900, background: "#F8F9FA" },
  assets: [{ id: "logo", type: "svg", path: "logo.svg" }],
  nodes: [
    {
      id: "root",
      type: "frame",
      name: "root",
      width: { type: "fill" },
      height: { type: "fill" },
      layout: { mode: "vertical" },
      children: [
        {
          id: "navbar",
          type: "frame",
          name: "navbar",
          width: { type: "fill" },
          height: { type: "number", value: 64 },
          padding: [0, 40, 0, 40],
          fill: { type: "solid", color: "#FFFFFF" },
          shadow: { x: 0, y: 2, blur: 8, color: "#00000012" },
          layout: { mode: "horizontal", align: "center", justify: "space-between" },
          children: [
            {
              id: "logo",
              type: "image",
              name: "logo",
              width: { type: "number", value: 120 },
              height: { type: "number", value: 32 },
              asset: "logo.svg",
              children: [],
            },
            {
              id: "nav-links",
              type: "frame",
              name: "nav-links",
              layout: { mode: "horizontal", gap: 32 },
              children: [
                {
                  id: "link-1",
                  type: "text",
                  content: "Features",
                  font_size: 14,
                  color: "#111111",
                  children: [],
                },
                {
                  id: "link-2",
                  type: "text",
                  content: "Pricing",
                  font_size: 14,
                  color: "#111111",
                  children: [],
                },
                {
                  id: "link-3",
                  type: "text",
                  content: "Docs",
                  font_size: 14,
                  color: "#111111",
                  children: [],
                },
              ],
            },
            {
              id: "cta-btn",
              type: "frame",
              name: "cta-btn",
              padding: [8, 20, 8, 20],
              fill: { type: "solid", color: "#0066FF" },
              corner_radius: [8, 8, 8, 8],
              children: [
                {
                  id: "cta-text",
                  type: "text",
                  content: "Get Started",
                  font_size: 14,
                  font_weight: "medium",
                  color: "#FFFFFF",
                  children: [],
                },
              ],
            },
          ],
        },
        {
          id: "hero",
          type: "frame",
          name: "hero",
          width: { type: "fill" },
          height: { type: "number", value: 600 },
          layout: { mode: "vertical", gap: 24, align: "center", justify: "center" },
          children: [
            {
              id: "hero-title",
              type: "text",
              content: "Design as Code",
              font_size: 72,
              font_weight: "bold",
              color: "#111111",
              children: [],
            },
            {
              id: "hero-subtitle",
              type: "text",
              content: "Build beautiful interfaces with code",
              font_size: 20,
              color: "#666666",
              children: [],
            },
          ],
        },
      ],
    },
  ],
};
