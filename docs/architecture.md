# Architecture Overview

## Pipeline

```
.pastel source → Lexer → Parser → Semantic → IR → Skia Renderer → PNG/SVG/PDF
                                                 → Codegen → HTML/React/CSS
                                                 → Lint → violation report
```

## 5 Crates

```
pastel-lang          Compiler frontend
├── lexer/           Tokenization (keywords, colors, numbers, strings, dot-access)
├── parser/          Recursive descent (top-level decls, nodes, expressions, components, pages)
├── semantic/        Variable resolution, token registration, component expansion, include processing
├── ir/              IrDocument with typed IrNodeData enum + design tokens
└── formatter        AST round-trip source formatting

pastel-render        Skia rendering engine
├── layout           Two-pass flexbox/grid layout (measure + place, absolute positioning)
├── painter          Skia painting (fill/gradient, stroke/dash, shadow, blur, rotation, blend, path)
└── export           PNG (Skia), SVG (string gen), PDF (Skia PDF backend)

pastel-codegen       Code generation
├── tokens           IR tokens → CSS custom properties + JSON
├── html             IR nodes → standalone HTML page
└── react            IR nodes → React component + tokens.css

pastel-lint          Design rule checking
├── rules            Check colors/spacing/radius/font-size against token definitions
└── report           Violation output (text + JSON)

pastel-cli           CLI entry point (8 commands)
```

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Pure Rust + Skia | Single binary, industrial-grade rendering |
| Non-Turing-complete DSL | AI generation is reliable and predictable |
| Design tokens as first-class | `token` blocks define the system, `lint` enforces it |
| Namespace includes | `include "x" as ns` prevents conflicts, enables composition |
| Compile-time component expansion | `use` is macro expansion, IR has no component refs |
| SVG path data for free drawing | Standard path syntax, Skia parses and renders |
| Multi-page support | `page` blocks → separate PNG files or multi-page PDF |
