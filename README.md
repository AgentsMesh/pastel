# Pastel

**Design System as Code.** Define design tokens, draw UI, validate consistency, generate code — all in one `.pastel` file.

```pastel
token colors {
    primary = #0066FF
    text    = #111827
}

token spacing {
    md = 16
    lg = 24
}

component button(label, color = colors.primary) {
    frame {
        padding = [10, spacing.lg]
        fill    = color
        radius  = 8
        text label { size = 14, weight = medium, color = #FFFFFF }
    }
}

frame hero {
    layout  = vertical
    align   = center
    gap     = spacing.md

    text "Hello, Pastel!" { size = 48, weight = bold, color = colors.text }
    use button("Get Started")
}
```

```bash
pastel check design.pastel        # Compile-time validation
pastel lint design.pastel          # Check values against token definitions
pastel build design.pastel -o .png # Render with Skia
pastel gen design.pastel --format tokens -o src/  # Export CSS variables
```

## Why Pastel?

Pastel is a **Design System language** — not just a drawing tool. The `.pastel` file is simultaneously:

1. **Design tokens** — structured, typed, enforceable
2. **Component library** — reusable, parameterized, compile-time expanded
3. **Design specs** — renderable to PNG/SVG/PDF
4. **Code source** — generates HTML, React, CSS custom properties
5. **Lint rules** — validates that your design uses its own tokens

| Capability | What it does |
|------------|-------------|
| `pastel check` | Compile-time syntax + semantic validation |
| `pastel lint` | Ensure design values match token definitions |
| `pastel build` | Render to PNG, SVG, or PDF (Skia engine) |
| `pastel gen` | Generate HTML, React components, CSS tokens |
| `pastel plan` | Show node tree (dry-run) |
| `pastel fmt` | Format source code |
| `pastel inspect` | Show IR summary or full JSON |
| `pastel serve` | Start live preview server |

## Architecture

Pure Rust. Single binary. Zero runtime dependencies.

```
.pastel → Lexer → Parser → Semantic → IR → Skia Renderer → PNG/SVG/PDF
                                          → Codegen → HTML/React/CSS
                                          → Lint → violation report
```

| Crate | Role |
|-------|------|
| `pastel-lang` | Compiler frontend: lexer, parser, semantic analyzer, IR, formatter |
| `pastel-render` | Skia rendering: flexbox/grid layout, painting, PNG/SVG/PDF export |
| `pastel-codegen` | Code generation: HTML, React components, CSS custom properties |
| `pastel-lint` | Design rule checking: token consistency validation |
| `pastel-cli` | CLI entry point with 8 commands |

## Installation

```bash
git clone https://github.com/AgentsMesh/pastel.git
cd pastel
cargo build --release
cp target/release/pastel /usr/local/bin/
```

Requires [Rust](https://rustup.rs/) 1.75+.

## Language Reference

### Design Tokens

```pastel
token colors {
    primary    = #0066FF
    secondary  = #6B7280
    danger     = #EF4444
    text       = #111827
    text-muted = #6B7280
}

token spacing {
    xs = 4
    sm = 8
    md = 16
    lg = 24
    xl = 32
    scale = [0, 4, 8, 12, 16, 24, 32, 48, 64]
}

token radius { sm = 4, md = 8, lg = 12, full = 999 }

token typography {
    heading = { size = 32, weight = bold, line-height = 40 }
    body    = { size = 16, weight = normal, line-height = 24 }
}

token shadow {
    sm = [0, 1, 3, #0000000D]
    md = [0, 4, 12, #0000001A]
}
```

Reference tokens with dot syntax: `colors.primary`, `spacing.lg`, `shadow.md`.

### Includes & Namespaces

```pastel
include "./brand-tokens.pastel" as brand   // namespaced: brand.colors.primary
include "./shared.pastel"                   // bare merge (errors on name conflicts)
```

### Nodes

**Frame** — container with layout:
```pastel
frame card {
    width = 300, height = 200, padding = [16, 24]
    layout = vertical, gap = 12, align = center, justify = space-between
    fill = #FFFFFF, radius = 12, shadow = [0, 4, 12, #0000001A]
}
```

**Text** — with full typography control:
```pastel
text "Hello World" {
    size = 24, weight = bold, font = "Inter", color = #111
    align = center, line-height = 32, letter-spacing = 0.5
    text-decoration = underline, text-transform = uppercase
    width = 300, wrap = true
}
```

**Image** — asset reference:
```pastel
asset hero = image("./hero.jpg")
image hero { width = 800, height = 400, radius = 12, fit = cover }
```

**Shape** — basic geometry + SVG paths:
```pastel
shape circle { type = ellipse, width = 100, height = 100, fill = #FF0066 }
shape heart {
    path = "M 100 200 C 100 100, 250 100, 250 200 S 400 300, 250 400 Z"
    width = 200, height = 200, fill = #FF4466
}
```

### Layout

```pastel
layout = horizontal      // or vertical, grid
gap = 16
padding = [16, 24]       // [v, h] or [top, right, bottom, left]
align = center           // start | center | end | stretch
justify = space-between  // start | center | end | space-between | space-around

// Grid
layout = grid, columns = 3, gap = 16

// Absolute positioning
position = absolute, top = 10, right = 20
```

### Visual Properties

| Property | Syntax |
|----------|--------|
| Solid fill | `fill = #0066FF` |
| Transparent | `fill = transparent` |
| Linear gradient | `fill = linear-gradient(135, #6366F1, #EC4899)` |
| Radial gradient | `fill = radial-gradient(#FF6B6B, #4ECDC4)` |
| Stroke | `stroke = [2, #DDD]` |
| Dashed stroke | `stroke-dash = [8, 4]` |
| Corner radius | `radius = 8` or `radius = [8, 8, 4, 4]` |
| Drop shadow | `shadow = [0, 4, 12, #0000001A]` |
| Inner shadow | `inner-shadow = [0, 2, 8, #00000020]` |
| Opacity | `opacity = 0.8` |
| Blur | `blur = 6` |
| Background blur | `background-blur = 20` |
| Rotation | `rotation = 45` |
| Blend mode | `blend = multiply` (screen, overlay, darken, lighten) |

### Components

```pastel
component card(title, description, color = colors.primary) {
    frame {
        padding = spacing.lg, fill = #FFFFFF, radius = radius.lg, shadow = shadow.md
        layout = vertical, gap = spacing.sm
        text title { size = 20, weight = semibold, color = colors.text }
        text description { size = 14, color = colors.text-muted, wrap = true, width = 280 }
    }
}

use card("Features", "Token system, lint, codegen")
use card("Custom", "Override defaults", color = colors.danger)
```

### Pages

```pastel
page "home" {
    frame hero { ... }
}
page "about" {
    frame content { ... }
}
```

Multi-page output: `pastel build design.pastel -o output.png` → `output_home.png`, `output_about.png`.
PDF: `pastel build design.pastel -o design.pdf` → multi-page PDF.

## Design System Workflow

```bash
# 1. Define your system + draw designs
vim design.pastel

# 2. Validate syntax
pastel check design.pastel

# 3. Lint: do all values use tokens?
pastel lint design.pastel
# → [card] padding 13px not on spacing scale (use 12px or 16px)
# → [text_2] color #0065FE not in token colors (use #0066FF)

# 4. Preview
pastel build design.pastel -o preview.png

# 5. Generate code
pastel gen design.pastel --format tokens -o src/    # CSS variables + JSON
pastel gen design.pastel --format html -o dist/     # Standalone HTML
pastel gen design.pastel --format react -o src/     # React component

# 6. Export
pastel build design.pastel -o design.svg            # Vector
pastel build design.pastel -o design.pdf            # Print-ready
pastel inspect design.pastel --json                 # Machine-readable IR
```

## Examples

| Example | Demonstrates |
|---------|-------------|
| `hello-world` | Minimal .pastel file |
| `landing-page` | Navbar, hero, buttons, gradients |
| `dashboard` | Sidebar + grid layout + components |
| `mobile-app` | Mobile UI, tab bar, list items |
| `component-demo` | Component params and reuse |
| `design-system` | Token definitions + token-driven design |
| `advanced` | Gradients, text wrapping, absolute positioning |
| `effects` | Rotation, blur, inner shadow, dashed stroke, blend |
| `multipage` | Multi-page design, grid, radial gradient |
| `paths` | SVG path data: heart, star, triangle, bezier curves |
| `fortune-shrine` | Chinese traditional design with absolute layout |

## Related Projects

- [VEAC](https://github.com/AgentsMesh/veac) — Video Editing as Code (same architecture, different domain)

## License

[MIT](LICENSE)
