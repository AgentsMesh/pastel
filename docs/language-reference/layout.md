# Layout

Frames can define layout rules that control how their children are arranged.

## Layout Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `layout` | `horizontal` \| `vertical` | `vertical` | Direction of child arrangement |
| `gap` | number | `0` | Space between children (px) |
| `padding` | number or array | `0` | Inner spacing |
| `align` | enum | `start` | Cross-axis alignment |
| `justify` | enum | `start` | Main-axis distribution |

## Layout Mode

```pastel
frame row { layout = horizontal, gap = 16 }
frame col { layout = vertical, gap = 8 }
```

- `horizontal` — children placed left to right
- `vertical` — children placed top to bottom

If any layout-related property is set (`layout`, `gap`, `align`, `justify`), the frame activates layout. Default mode is `vertical`.

## Gap

Space in pixels between adjacent children.

```pastel
frame spaced { layout = vertical, gap = 24 }
```

## Padding

Inner spacing between the frame edge and its children.

```pastel
// Uniform padding
padding = 16                    // 16px all sides

// Vertical + horizontal
padding = [12, 24]              // 12px top/bottom, 24px left/right

// All four sides
padding = [16, 24, 16, 24]     // [top, right, bottom, left]
```

## Align (Cross-Axis)

Controls alignment perpendicular to the layout direction.

| Value | Description |
|-------|-------------|
| `start` | Align to start (left for vertical, top for horizontal) |
| `center` | Center along cross-axis |
| `end` | Align to end |
| `stretch` | Stretch to fill cross-axis |

```pastel
frame centered {
    layout = vertical
    align  = center    // children centered horizontally

    text "Centered" { size = 16, color = #333 }
}
```

## Justify (Main-Axis)

Controls distribution along the layout direction.

| Value | Description |
|-------|-------------|
| `start` | Pack children at the start |
| `center` | Center children |
| `end` | Pack children at the end |
| `space-between` | Equal space between children |
| `space-around` | Equal space around children |

```pastel
frame navbar {
    layout  = horizontal
    justify = space-between
    align   = center
    padding = [0, 40]

    text "Logo" { size = 18, weight = bold, color = #111 }
    text "Menu" { size = 14, color = #333 }
}
```

## Dimension Keywords

Width and height accept numbers or keywords.

| Value | Description |
|-------|-------------|
| `fill` | Expand to fill available space in parent |
| `hug` | Shrink to fit content |
| number | Fixed pixel size |

```pastel
frame sidebar {
    width  = 240     // fixed 240px
    height = fill    // fill parent height
}

frame card {
    width  = fill    // fill parent width
    height = hug     // fit content
    padding = [16, 20]
}
```

## Full Example

```pastel
frame page {
    width = fill, height = fill, layout = vertical

    frame header {
        width = fill, height = 64, padding = [0, 24]
        layout = horizontal, align = center, justify = space-between
        fill = #FFFFFF

        text "App" { size = 18, weight = bold, color = #111 }
        text "Settings" { size = 14, color = #666 }
    }

    frame content {
        width = fill, height = fill, padding = [24, 24]
        layout = vertical, gap = 16

        text "Welcome" { size = 24, weight = bold, color = #111 }
    }
}
```
