# Canvas Declaration

The `canvas` block defines the document's dimensions and background. It must appear at most once per file.

## Syntax

```pastel
canvas "name" {
    width      = <number>
    height     = <number>
    background = <color>
}
```

The name is a quoted string used as the document title.

## Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `width` | integer | `1440` | Canvas width in pixels |
| `height` | integer | `900` | Canvas height in pixels |
| `background` | color | none (transparent) | Background color |

## Defaults

If no `canvas` block is present, the compiler uses:

```
name       = "untitled"
width      = 1440
height     = 900
background = (none)
```

## Examples

### Minimal

```pastel
canvas "my-design" {}
```

Uses all defaults: 1440x900, no background.

### Full specification

```pastel
canvas "hero-section" {
    width      = 1440
    height     = 600
    background = #F8F9FA
}
```

### Small artboard

```pastel
canvas "icon" {
    width  = 64
    height = 64
    background = transparent
}
```

### Comma-separated (also valid)

```pastel
canvas "card" { width = 400, height = 300, background = #FFFFFF }
```

## Notes

- Canvas dimensions are always fixed pixel values (no `fill` or `hug`).
- Only one `canvas` per file. Included files' canvas declarations are ignored.
- The canvas name appears in `pastel inspect` and `pastel plan` output.
