# Pastel

**Design as Code.** A compiled DSL for design — AI writes `.pastel` files, the compiler renders pixels.

```pastel
canvas "hello" {
    width  = 400
    height = 300
    background = #FFFFFF
}

let primary = #0066FF

frame hero {
    width   = fill
    height  = fill
    layout  = vertical
    align   = center
    justify = center
    gap     = 16

    text "Hello, Pastel!" {
        size   = 64
        weight = bold
        color  = #111111
    }

    frame cta {
        padding = [12, 32]
        fill    = primary
        radius  = 8

        text "Get Started" { size = 16, weight = medium, color = #FFFFFF }
    }
}
```

```bash
pastel check hello.pastel    # Validate
pastel plan hello.pastel     # Show node tree
pastel build hello.pastel -o hello.png  # Render
```

## Why Pastel?

Design tools today are GUI-first. Pastel flips this: **the source file is the design**. AI agents write `.pastel` files directly — no mouse, no canvas manipulation, no screenshot parsing. The compiler validates, the renderer draws.

This is the same philosophy as [VEAC](https://github.com/AgentsMesh/veac) (Video Editing as Code), applied to design.

| | Traditional Design Tools | Pastel |
|---|---|---|
| Source of truth | Proprietary binary file | `.pastel` text file |
| AI interaction | Screenshot → guess → click | Write code → compile → render |
| Version control | Manual export | `git diff` native |
| Validation | Runtime errors | Compile-time checks |
| Reusability | Copy-paste | `component` + `include` |

## Architecture

```
.pastel source → Lexer → Parser → Semantic → IR (JSON) → Canvas 2D / JSX / CSS
                 ─────────── Rust ──────────────          ── TypeScript ──
```

- **Rust** — Compiler frontend (lexer, parser, semantic analyzer, IR generation)
- **TypeScript** — Rendering (Canvas 2D), code generation (JSX/Tailwind/CSS), live preview

## Installation

### From source

```bash
git clone https://github.com/AgentsMesh/pastel.git
cd pastel
cargo build --release
cp target/release/pastel /usr/local/bin/
```

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+ (for renderer/preview)
- [pnpm](https://pnpm.io/) 9+ (for TypeScript packages)

## CLI Commands

| Command | Description |
|---------|-------------|
| `pastel check <file>` | Validate syntax and semantics |
| `pastel plan <file>` | Show the node tree (dry-run) |
| `pastel build <file> -o <output>` | Compile and render to PNG/SVG/JSON |
| `pastel fmt <file>` | Format source file |
| `pastel inspect <file> [--json]` | Show IR summary or full JSON |
| `pastel serve <file>` | Start live preview server |

## Language Reference

### Top-level declarations

```pastel
canvas "name" { width = 1440, height = 900, background = #F8F9FA }

asset logo = image("./assets/logo.svg")

let primary = #0066FF
let radius  = 8

include "./shared.pastel"
```

### Nodes

```pastel
frame navbar {
    width   = fill
    height  = 64
    padding = [0, 40]
    layout  = horizontal
    align   = center
    justify = space-between
    fill    = #FFFFFF
    shadow  = [0, 2, 8, #00000012]

    text "Hello" { size = 14, weight = bold, color = #111 }
    image logo { width = 120, height = 32 }
}
```

**Node types:** `frame`, `text`, `image`, `shape`

### Layout

```pastel
frame container {
    layout  = vertical       // horizontal | vertical
    gap     = 16
    padding = [16, 24]       // [v, h] or [top, right, bottom, left]
    align   = center         // start | center | end | stretch
    justify = space-between  // start | center | end | space-between | space-around
}
```

### Components

```pastel
component button(label, color = primary) {
    frame {
        padding = [10, 24]
        fill    = color
        radius  = 8

        text label { size = 14, weight = medium, color = #FFFFFF }
    }
}

use button("Sign Up")
use button("Learn More", color = #333333)
```

Components are expanded at compile time (macro-style, not runtime). The language is intentionally **non-Turing-complete** — no loops, no conditionals, no recursion. This makes AI generation reliable and predictable.

### Style properties

| Property | Type | Example |
|----------|------|---------|
| `width` / `height` | number, `fill`, `hug` | `width = 200`, `width = fill` |
| `fill` | color, `transparent` | `fill = #0066FF` |
| `stroke` | [width, color] | `stroke = [1, #DDD]` |
| `radius` | number or [tl, tr, br, bl] | `radius = 8` |
| `shadow` | [x, y, blur, color] | `shadow = [0, 2, 8, #00000012]` |
| `opacity` | 0.0–1.0 | `opacity = 0.5` |
| `padding` | number or array | `padding = [16, 24]` |

### Text properties

| Property | Values |
|----------|--------|
| `size` | number (px) |
| `weight` | `thin`, `light`, `normal`, `medium`, `semibold`, `bold`, `extrabold`, `black` |
| `color` | hex color |
| `font` | font family string |
| `align` | `left`, `center`, `right` |

## Project Structure

```
pastel/
├── crates/
│   ├── pastel-lang/          # Compiler frontend (lexer → parser → semantic → IR)
│   └── pastel-cli/           # CLI tool
├── packages/
│   ├── renderer/             # Canvas 2D rendering engine
│   ├── codegen/              # IR → JSX/Tailwind/CSS/Design Tokens
│   ├── preview/              # Live preview (file watch + WebSocket + HTTP)
│   └── web/                  # Web editor (React, code + preview split-view)
├── examples/                 # Example .pastel files
│   ├── hello-world/
│   ├── landing-page/
│   ├── component-demo/
│   ├── dashboard/
│   └── mobile-app/
└── fixtures/                 # Test fixtures
```

## Examples

### Dashboard with reusable components

```pastel
component stat-card(label, value, color = primary) {
    frame {
        width   = 260
        height  = 120
        padding = [20, 24]
        fill    = #FFFFFF
        radius  = 8
        shadow  = [0, 1, 4, #0000000F]
        layout  = vertical
        gap     = 8

        text label { size = 14, color = #8C8C8C }
        text value { size = 28, weight = bold, color = color }
    }
}

frame stats {
    layout = horizontal
    gap    = 16

    use stat-card("Users", "12,345")
    use stat-card("Revenue", "$48K", color = #52C41A)
}
```

### Plan output

```
$ pastel plan examples/dashboard/main.pastel

Document: dashboard (1200x800)
└── frame sidebar (240xfill) [vertical, gap=4]
    ├── text "Dashboard" (18px, bold)
    └── frame nav [vertical, gap=2]
        ├── frame
        │   └── text "Overview" (14px, medium)
        └── ...
└── frame main-content (fillxfill) [vertical, gap=24]
    ├── text "Overview" (24px, bold)
    ├── frame stats-row [horizontal, gap=16]
    │   ├── frame (260x120) [vertical, gap=8]
    │   │   ├── text "Total Users" (14px)
    │   │   └── text "12,345" (28px, bold)
    │   └── ...
    └── frame chart-area (fillx300) [vertical, gap=12]
```

## Code Generation

Pastel can export designs to frontend code:

```bash
pastel export landing.pastel --format jsx      # React + Tailwind
pastel export landing.pastel --format tokens   # Design Tokens JSON
pastel inspect landing.pastel --json           # Raw IR JSON
```

## Live Preview

```bash
pastel serve landing.pastel
# ✓ Compiled (12ms)
# ✓ Preview: http://localhost:3210
# ◎ Watching for changes...
```

Edit your `.pastel` file — the browser refreshes instantly via WebSocket.

## For AI Agents

Pastel is designed for AI. The typical workflow:

```bash
# 1. AI writes the .pastel file
cat > design.pastel << 'EOF'
canvas "hero" { width = 1440, height = 600, background = #F8F9FA }
frame main {
    width = fill, height = fill
    layout = vertical, align = center, justify = center
    text "Welcome" { size = 48, weight = bold, color = #111 }
}
EOF

# 2. Validate
pastel check design.pastel

# 3. Preview
pastel plan design.pastel

# 4. Render
pastel build design.pastel -o design.png

# 5. Export to code
pastel export design.pastel --format jsx
```

Every command outputs structured, parseable results. Use `--json` for machine-readable output.

## Related Projects

- [VEAC](https://github.com/AgentsMesh/veac) — Video Editing as Code (same architecture, different domain)

## License

[MIT](LICENSE)
