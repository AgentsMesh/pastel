/// Output the complete Pastel DSL syntax reference for LLM consumption.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    print!("{}", SYNTAX_REFERENCE);
    Ok(())
}

const SYNTAX_REFERENCE: &str = r#"# Pastel DSL Syntax Reference

Pastel is a design-as-code language. You write `.pastel` files to define design tokens,
components, layouts, and visual styles, then compile them to PNG/SVG/PDF or generate
HTML/React/CSS code.

---

## 1. File Structure

A `.pastel` file contains top-level declarations in any order:

```
canvas "name" { ... }          // optional, at most one
asset <name> = image("path")   // external resources
asset <name> = font("path")
let <name> = <expr>            // variables
include "path.pastel"          // file imports
include "path.pastel" as ns    // namespaced import (access via ns.xxx)
token <group> { ... }          // design token blocks
component <name>(...) { ... }  // reusable components
page "name" { ... }            // named pages (for multi-page designs)
<node>                         // top-level nodes (frame/text/image/shape/use)
```

---

## 2. Comments

```
// line comment (only style supported)
```

---

## 3. Declarations

### canvas
Optional. Defines the artboard. At most one per file.

```
canvas "my-design" {
    width      = 1200
    height     = 800
    background = #F5F5F5
}
```

### asset
Declare external image or font resources.

```
asset logo   = image("./logo.png")
asset inter  = font("./Inter.ttf")
```

### let
Bind a value to a name for reuse.

```
let primary   = #0066FF
let gap_md    = 16
let corners   = [8, 8, 8, 8]
```

### include
Import another `.pastel` file. With `as`, all exported names are prefixed.

```
include "./tokens.pastel"
include "./shared.pastel" as shared
// then use: shared.primary, shared.button, etc.
```

### token
Group related design tokens. Access via `<group>.<name>` (e.g. `colors.primary`).

```
token colors {
    primary   = #0066FF
    secondary = #6B7280
    danger    = #EF4444
}

token spacing {
    xs = 4
    sm = 8
    md = 16
    lg = 24
}

token typography {
    heading = { size = 32, weight = bold }
    body    = { size = 16, weight = normal }
}

token shadow {
    sm = [0, 1, 3, #0000000D]
    md = [0, 4, 12, #0000001A]
}
```

### component
Reusable design macro. Parameters can have defaults. Body is a single node.

```
component button(label, color = primary) {
    frame {
        padding = [10, 24]
        fill    = color
        radius  = 8

        text label { size = 14, weight = medium, color = #FFFFFF }
    }
}
```

Instantiate with `use`:
```
use button("Sign Up")
use button("Cancel", color = #999999)
```

### page
Named page container. Each page renders separately; useful for multi-page designs.

```
page "home" {
    frame hero { ... }
}

page "about" {
    frame content { ... }
}
```

---

## 4. Node Types

### frame
Container with layout. Can have children (any node type).

```
frame sidebar {
    width   = 280
    height  = fill
    padding = [24, 16]
    fill    = #FFFFFF
    layout  = vertical
    gap     = 12
    align   = start
    justify = start

    // children go here
    text "Menu" { size = 18, weight = bold, color = #111111 }
    frame item { ... }
}
```

### text
Text leaf node. First string is content, optional identifier is the node name.

```
text "Hello World" title {
    size            = 32
    weight          = bold
    family          = "Inter"
    color           = #111111
    align           = center
    line-height     = 40
    letter-spacing  = 2.0
    wrap            = true
    width           = 300
    text-decoration = underline
    text-transform  = uppercase
}
```

Inline shorthand (comma-separated properties):
```
text "Label" { size = 14, weight = medium, color = #FFFFFF }
```

### image
Image leaf node referencing an asset.

```
asset avatar = image("./avatar.png")

image avatar {
    width  = 120
    height = 120
    radius = 999
    fit    = cover
}
```

### shape
Geometric primitive or free-form path.

```
// Rectangle
shape box {
    type   = rectangle
    width  = 100
    height = 100
    fill   = #FF6B6B
    radius = 8
}

// Ellipse
shape circle {
    type   = ellipse
    width  = 80
    height = 80
    fill   = #4ECDC4
}

// Line
shape divider {
    type   = line
    width  = 200
    height = 0
    stroke = [1, #CCCCCC]
}

// SVG Path (bezier curves, polygons, free-form)
shape heart {
    path   = "M 300 120 C 300 80, 240 40, 200 80 C 160 120, 160 180, 300 280 Z"
    width  = 300
    height = 250
    fill   = #FF4466
}
```

### use
Instantiate a component. Args can be positional or named.

```
use button("Click me")
use card(title = "Welcome", description = "Hello")
```

---

## 5. Property Reference

### Layout & Sizing

| Property   | Type                         | Example                          |
|-----------|------------------------------|----------------------------------|
| width     | `fill` / `hug` / number     | `width = fill`, `width = 400`    |
| height    | `fill` / `hug` / number     | `height = hug`, `height = 300`   |
| padding   | number or array              | `padding = 24`, `padding = [12, 24, 12, 24]` |
| layout    | `vertical` / `horizontal` / `grid` | `layout = horizontal`    |
| gap       | number                       | `gap = 16`                       |
| align     | `start`/`center`/`end`/`stretch` | `align = center`           |
| justify   | `start`/`center`/`end`/`space-between`/`space-around` | `justify = space-between` |
| columns   | integer (grid only)          | `columns = 3`                    |
| rows      | integer (grid only)          | `rows = 2`                       |

Padding array forms:
- `[all]` — `[24]`
- `[vertical, horizontal]` — `[12, 24]`
- `[top, right, bottom, left]` — `[12, 24, 12, 24]`

### Positioning

| Property  | Type                        | Example               |
|----------|-----------------------------|-----------------------|
| position | `relative` / `absolute`     | `position = absolute` |
| top      | number                      | `top = 10`            |
| right    | number                      | `right = -8`          |
| bottom   | number                      | `bottom = 0`          |
| left     | number                      | `left = 20`           |
| rotation | number (degrees)            | `rotation = 45`       |

### Visual Styling

| Property        | Type                    | Example                              |
|----------------|-------------------------|--------------------------------------|
| fill           | color / gradient / `transparent` | `fill = #FF0066`             |
| stroke         | `[width, color]`        | `stroke = [2, #333333]`             |
| stroke-dash    | `[on, off]`             | `stroke-dash = [8, 4]`              |
| radius         | number or array         | `radius = 12`, `radius = [8, 12, 8, 12]` |
| shadow         | `[x, y, blur, color]`   | `shadow = [0, 4, 12, #0000001A]`    |
| inner-shadow   | `[x, y, blur, color]`   | `inner-shadow = [0, 2, 8, #00000040]` |
| opacity        | number (0–1)            | `opacity = 0.5`                      |
| blur           | number                  | `blur = 6`                           |
| background-blur| number                  | `background-blur = 20`               |
| blend          | blend mode enum         | `blend = multiply`                   |

Corner radius array: `[top-left, top-right, bottom-right, bottom-left]`

### Text Properties

| Property        | Type                        | Example                    |
|----------------|-----------------------------|----------------------------|
| size / font-size | number                    | `size = 16`                |
| weight / font-weight | weight enum            | `weight = bold`            |
| family / font-family | string                 | `family = "Inter"`         |
| color          | color                        | `color = #111111`          |
| align / text-align | `left`/`center`/`right` | `align = center`           |
| line-height    | number                       | `line-height = 24`         |
| letter-spacing | number                       | `letter-spacing = 2.0`     |
| wrap           | boolean                      | `wrap = true`              |
| text-decoration| `none`/`underline`/`strikethrough` | `text-decoration = underline` |
| text-transform | `none`/`uppercase`/`lowercase` | `text-transform = uppercase` |

### Image Properties

| Property | Type                              | Example        |
|---------|-----------------------------------|----------------|
| fit     | `cover`/`contain`/`fill`/`none`   | `fit = cover`  |

### Shape Properties

| Property | Type                                     | Example                        |
|---------|------------------------------------------|--------------------------------|
| type    | `rectangle`/`ellipse`/`line`/`path`      | `type = ellipse`               |
| path    | SVG path data string                     | `path = "M 0 0 L 100 100"`    |

---

## 6. Expressions

| Type         | Syntax                                    | Example                              |
|-------------|-------------------------------------------|--------------------------------------|
| Integer     | digits                                    | `42`, `-8`                           |
| Float       | digits.digits                             | `1.5`, `-0.5`                        |
| String      | `"..."`                                   | `"Hello"` (escapes: `\n \t \\ \"`)  |
| Color       | `#RRGGBB` or `#RRGGBBAA`                 | `#FF0066`, `#0000001A`               |
| Boolean     | `true` / `false`                          | `wrap = true`                        |
| Identifier  | name or dot-path                          | `primary`, `colors.primary`          |
| Array       | `[expr, expr, ...]`                       | `[12, 24, 12, 24]`                  |
| Object      | `{ key = value, ... }`                    | `{ size = 32, weight = bold }`       |
| Function    | `name(args...)`                           | `linear-gradient(135, #F00, #00F)`   |

---

## 7. Fill & Gradient Syntax

```
// Solid color
fill = #FF0066
fill = primary              // variable reference

// Transparent
fill = transparent

// Linear gradient: linear-gradient(angle, color1, color2, ...)
fill = linear-gradient(135, #6366F1, #EC4899)
fill = linear-gradient(90, colors.primary, colors.secondary, #FFFFFF)

// Radial gradient: radial-gradient(color1, color2, ...)
// or with center: radial-gradient(cx, cy, color1, color2, ...)
fill = radial-gradient(#FF6B6B, #4ECDC4)
fill = radial-gradient(50, 50, #6C5CE7, #A29BFE)
```

---

## 8. Enum Quick Reference

**layout:**    `vertical` | `horizontal` | `grid`
**align:**     `start` | `center` | `end` | `stretch`
**justify:**   `start` | `center` | `end` | `space-between` | `space-around`
**position:**  `relative` | `absolute`
**blend:**     `normal` | `multiply` | `screen` | `overlay` | `darken` | `lighten`
**weight:**    `thin` (100) | `light` (300) | `normal` (400) | `medium` (500) | `semibold` (600) | `bold` (700) | `extrabold` (800) | `black` (900)
**fit:**       `cover` | `contain` | `fill` | `none`
**text-decoration:** `none` | `underline` | `strikethrough`
**text-transform:**  `none` | `uppercase` | `lowercase`
**text-align:**      `left` | `center` | `right`
**shape type:**      `rectangle` | `ellipse` | `line` | `path`

---

## 9. Complete Example

```pastel
canvas "app" {
    width  = 800
    height = 600
    background = #F5F5F5
}

token colors {
    primary = #0066FF
    text    = #111827
    muted   = #6B7280
    bg      = #FFFFFF
}

token spacing {
    sm = 8
    md = 16
    lg = 24
}

component card(title, description) {
    frame {
        padding = spacing.lg
        fill    = colors.bg
        radius  = 12
        shadow  = [0, 4, 12, #0000001A]
        layout  = vertical
        gap     = spacing.sm

        text title       { size = 20, weight = semibold, color = colors.text }
        text description { size = 14, color = colors.muted, wrap = true, width = 280 }
    }
}

frame header {
    width   = fill
    height  = 64
    padding = [0, spacing.lg]
    fill    = colors.primary
    layout  = horizontal
    align   = center

    text "My App" { size = 20, weight = bold, color = #FFFFFF }
}

frame content {
    width   = fill
    padding = spacing.lg
    layout  = grid
    columns = 2
    gap     = spacing.md

    use card("Feature 1", "Description of feature one")
    use card("Feature 2", "Description of feature two")
}
```

---

## 10. CLI Usage

```bash
pastel check  <file>                       # validate syntax & semantics
pastel plan   <file>                       # show node tree
pastel build  <file> -o out.png            # render PNG/SVG/PDF
pastel fmt    <file>                       # format source
pastel inspect <file> --json               # output IR as JSON
pastel lint   <file>                       # check values against tokens
pastel gen    <file> --format react -o dir # generate React/HTML/CSS tokens
pastel syntax                              # print this reference
```
"#;
