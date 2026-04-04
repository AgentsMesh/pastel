# Architecture Overview

## Compilation Pipeline

```
 .pastel source
      │
      ▼
  ┌─────────┐     Token stream     ┌──────────┐     AST (Program)
  │  Lexer   │ ──────────────────▶  │  Parser  │ ────────────────▶
  └─────────┘                      └──────────┘
      ▼
  ┌────────────────┐    IR (IrDocument)    ┌──────────────┐
  │    Semantic     │ ───────────────────▶  │  Skia        │ ──▶ PNG
  │    Analyzer     │                      │  Renderer    │
  └────────────────┘                      └──────────────┘
```

**Stages:**

1. **Lexer** — Scans `.pastel` source into tokens
2. **Parser** — Builds AST with canvas, variables, assets, includes, components, nodes
3. **Semantic Analyzer** — Resolves variables, processes includes, expands components, validates types
4. **IR** — `IrDocument`: fully resolved tree with no references or macros
5. **Renderer** — Skia layout + painting → PNG output

## Crate Structure

| Crate | Path | Role |
|-------|------|------|
| `pastel-lang` | `crates/pastel-lang/` | Compiler frontend: lexer, parser, semantic, IR types |
| `pastel-render` | `crates/pastel-render/` | Skia rendering: layout computation + painting + PNG export |
| `pastel-cli` | `crates/pastel-cli/` | CLI: `build`, `check`, `plan`, `fmt`, `inspect`, `serve` |

### pastel-lang modules

```
src/
├── token.rs          # Token types and Span
├── lexer/
│   ├── mod.rs        # Lexer core and dispatch
│   └── scan.rs       # Token scanning rules
├── parser/
│   ├── mod.rs        # Top-level declarations + component parsing
│   ├── frame.rs      # Node parsing (frame/text/image/shape/use)
│   └── expr.rs       # Expression parsing
├── ast.rs            # AST types (Program, NodeDecl, ComponentDecl, etc.)
├── semantic/
│   ├── mod.rs        # Entry point, include processing
│   ├── resolve.rs    # VariableResolver + PropertyResolver
│   ├── builder.rs    # IrBuilder: AST → IR nodes
│   ├── expand.rs     # Component expansion (compile-time macro)
│   └── validate.rs   # Value validation rules
├── ir/
│   ├── mod.rs        # IrDocument, IrCanvas, IrAsset
│   ├── node.rs       # IrNode with typed IrNodeData enum
│   └── style.rs      # Color, Dimension, Layout, Fill, FontWeight, etc.
├── formatter.rs      # AST round-trip source formatter
└── error.rs          # PastelError with spans and hints
```

### pastel-render modules

```
src/
├── lib.rs            # render() entry: creates Skia surface, runs layout + paint
├── layout.rs         # Flexbox-subset layout engine (measure + place)
├── painter.rs        # Skia painting (fill, stroke, shadow, text, rounded corners)
└── export.rs         # PNG/JPEG export via Skia image encoding
```

## Data Flow

```
                      ┌──────────────────────────────┐
                      │        .pastel source         │
                      └──────────────┬───────────────┘
                                     │
                      ┌──────────────▼───────────────┐
                      │     pastel-lang (Rust)       │
                      │  lex → parse → analyze → IR  │
                      └──────────────┬───────────────┘
                                     │
                      ┌──────────────▼───────────────┐
                      │    pastel-render (Rust)      │
                      │  layout → paint → export     │
                      └──────────────┬───────────────┘
                                     │
                                  PNG file
```

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Pure Rust, no JS runtime | Single binary, zero external deps. Skia gives industrial-grade rendering. |
| Non-Turing-complete | No loops/conditionals/recursion. AI generation is reliable and predictable. |
| Components as compile-time macros | `use` expanded by semantic analyzer. IR has no component references. |
| `.pastel` text is the source of truth | No binary format. `git diff` native. |
| Type-safe IR with enums | `IrNodeData` enum (Frame/Text/Image/Shape), not a flat props bag. |
| Skia for rendering | Used by Chrome, Android, Flutter. Mature text shaping, GPU-ready. |
| Include with cycle detection | Merges variables/components/assets. HashSet tracks visited paths. |
