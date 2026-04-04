# Style Properties

Visual properties that control appearance of nodes.

## Fill

Background color of a frame or shape.

```pastel
fill = #0066FF             // solid color
fill = #0066FF80           // with alpha (50% opacity)
fill = transparent         // fully transparent
```

Color format: `#RRGGBB` or `#RRGGBBAA` (hex). The `transparent` keyword is also accepted.

## Stroke

Border around a frame or shape.

```pastel
stroke = [1, #DDDDDD]     // [width, color]
stroke = [2, #0066FF]
```

## Corner Radius

```pastel
radius = 8                         // uniform radius
radius = [8, 8, 0, 0]             // [top-left, top-right, bottom-right, bottom-left]
```

## Shadow

Drop shadow with x-offset, y-offset, blur radius, and color.

```pastel
shadow = [0, 2, 8, #00000012]     // [x, y, blur, color]
shadow = [0, 4, 16, #0000001A]
```

## Opacity

Overall node opacity from 0.0 (invisible) to 1.0 (fully visible).

```pastel
opacity = 0.5
opacity = 1.0
```

## Color Format

All colors use hex notation:

| Format | Example | Description |
|--------|---------|-------------|
| `#RRGGBB` | `#0066FF` | Opaque color |
| `#RRGGBBAA` | `#0066FF80` | Color with alpha |
| `transparent` | — | Fully transparent (`#00000000`) |

Colors can be stored in variables:

```pastel
let primary = #0066FF
let subtle  = #00000012

frame card {
    fill   = primary
    shadow = [0, 2, 8, subtle]
}
```

## Text Properties

Text-specific styling properties (only valid on `text` nodes).

| Property | Type | Values |
|----------|------|--------|
| `size` | number | Font size in pixels |
| `weight` | keyword | `thin`, `light`, `normal`, `medium`, `semibold`, `bold`, `extrabold`, `black` |
| `font` | string | Font family name |
| `color` | color | Text color |
| `align` | keyword | `left`, `center`, `right` |
| `line-height` | number | Line height multiplier |

### Example

```pastel
text "Heading" {
    size = 32, weight = bold, font = "Inter"
    color = #111111, align = center, line-height = 1.4
}
```

Weight CSS values: thin=100, light=300, normal=400, medium=500, semibold=600, bold=700, extrabold=800, black=900.

## Shape Properties

Shape nodes require a `type` property.

| Type | Description |
|------|-------------|
| `rectangle` (or `rect`) | Rectangle, supports radius |
| `ellipse` (or `circle`) | Ellipse/circle |
| `line` | Horizontal line |

```pastel
shape separator { type = line, width = fill, height = 1, fill = #E0E0E0 }

shape circle-badge { type = ellipse, width = 40, height = 40, fill = #0066FF }

shape rounded-box {
    type = rectangle, width = 200, height = 100
    fill = #FFFFFF, radius = 12
    stroke = [1, #DDD], shadow = [0, 1, 4, #0000000F]
}
```

## Image Properties

| Property | Type | Values |
|----------|------|--------|
| `width` | dimension | number, `fill`, `hug` |
| `height` | dimension | number, `fill`, `hug` |
| `radius` | number or array | Corner rounding |
| `shadow` | array | Drop shadow |
| `opacity` | number | 0.0 to 1.0 |
| `fit` | keyword | `cover`, `contain`, `fill`, `none` |

```pastel
asset hero-img = image("./hero.jpg")

image hero-img {
    width   = fill
    height  = 300
    fit     = cover
    radius  = 8
}
```
