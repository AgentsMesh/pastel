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
  │    Semantic     │ ───────────────────▶  │   Renderer   │ ──▶ PNG/SVG
  │    Analyzer     │                      │   (Canvas2D)  │
  └────────────────┘                      └──────────────┘
                        │
                        ├──▶ Codegen ──▶ JSX / CSS / Tokens
                        └──▶ Preview ──▶ Browser (WebSocket)
```

**Stages:**

1. **Lexer** — Scans `.pastel` source into tokens (keywords, identifiers, colors, numbers, strings, punctuation)
2. **Parser** — Builds an AST (`Program`) with canvas, variables, assets, includes, components, and nodes
3. **Semantic Analyzer** — Resolves variables, processes includes, expands components, validates types, produces IR
4. **IR** — `IrDocument` is a fully resolved, serializable JSON tree with no references or macros
5. **Render/Codegen** — TypeScript consumes IR JSON to render pixels or generate code

## Crate Structure (Rust)

| Crate | Path | Role |
|-------|------|------|
| `pastel-lang` | `crates/pastel-lang/` | Compiler frontend: lexer, parser, semantic analyzer, IR types |
| `pastel-cli` | `crates/pastel-cli/` | CLI entry point: `build`, `check`, `plan`, `fmt`, `inspect`, `serve` |

### pastel-lang modules

```
src/
├── token.rs          # Token types and Span
├── lexer/
│   ├── mod.rs        # Lexer core and dispatch
│   └── scan.rs       # Token scanning rules
├── parser/
│   ├── mod.rs        # Parser core and top-level declarations
│   ├── frame.rs      # Frame/node parsing
│   └── expr.rs       # Expression parsing
├── ast.rs            # AST node types (Program, NodeDecl, Expression, etc.)
├── semantic/
│   ├── mod.rs        # SemanticAnalyzer: entry point, include processing
│   ├── resolve.rs    # VariableResolver + PropertyResolver (type conversions)
│   ├── builder.rs    # IrBuilder: AST nodes → IR nodes
│   ├── expand.rs     # Component expansion (compile-time macro)
│   └── validate.rs   # Value validation rules
├── ir/
│   ├── mod.rs        # IrDocument, IrCanvas, IrAsset
│   ├── node.rs       # IrNode, IrNodeData (Frame/Text/Image/Shape)
│   └── style.rs      # Style types (Color, Dimension, Layout, Fill, etc.)
├── formatter.rs      # Source code formatter
└── error.rs          # PastelError with spans and hints
```

## Package Structure (TypeScript)

| Package | Path | Role |
|---------|------|------|
| `@pastel/renderer` | `packages/renderer/` | Canvas 2D rendering engine — turns IR into pixels |
| `@pastel/codegen` | `packages/codegen/` | IR to code: JSX, CSS, design tokens |
| `@pastel/preview` | `packages/preview/` | Live preview server: file watch + WebSocket + HTTP |
| `@pastel/web` | `packages/web/` | Web editor: React split-view (code + preview) |

### renderer modules

- `renderer.ts` — Main render orchestration
- `layout.ts` — Flexbox-like layout calculation
- `painter/` — Canvas 2D drawing primitives
- `export/` — PNG/SVG output

### codegen modules

- `jsx.ts` — IR to React JSX + Tailwind
- `css.ts` — IR to vanilla CSS
- `tokens.ts` — IR to design tokens JSON

### preview modules

- `server.ts` — HTTP + WebSocket server
- `watcher.ts` — File system watcher (chokidar)
- `compiler.ts` — Invokes pastel CLI and serves IR
- `template.ts` — HTML template for preview page

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
                               IrDocument (JSON)
                                         │
                  ┌──────────┬───────────┼───────────┐
                  ▼          ▼           ▼           ▼
             renderer    codegen     preview       web
            (Canvas2D)  (JSX/CSS)  (live serve)  (editor)
                  │          │           │           │
                  ▼          ▼           ▼           ▼
              PNG/SVG    .tsx/.css   Browser WS   React UI
```

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Non-Turing-complete language | No loops, conditionals, or recursion. Makes AI generation reliable and output predictable. |
| Components are compile-time macros | `use` is expanded by the semantic analyzer, not at runtime. IR has no component references. |
| Rust frontend, TypeScript backend | Rust gives fast parsing + validation; TypeScript gives easy Canvas 2D / browser integration. |
| IR is the sole interchange format | JSON-serializable `IrDocument` is the handoff between Rust compiler and TS renderers. |
| `.pastel` is the source of truth | No GUI, no binary format. Text in, pixels out. Designed for `git diff`. |
| Variable resolution at compile time | All `let` bindings and component params resolved before IR emission. IR has only concrete values. |
| Include with circular detection | `include` merges variables/components/assets. Visited set prevents infinite loops. |
| Canvas defaults (1440x900) | Sensible defaults so minimal files still produce usable output. |
