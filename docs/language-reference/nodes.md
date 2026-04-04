# Nodes

Nodes are the visual elements in a Pastel design. Every visible element is a node.

## Node Types

| Type | Description | Can have children |
|------|-------------|:-:|
| `frame` | Container with optional layout | Yes |
| `text` | Text content | No |
| `image` | Image asset reference | No |
| `shape` | Basic geometry (rect, ellipse, line) | No |

## Syntax

### Frame

```pastel
frame name {
    // properties
    // child nodes
}
```

A frame is a container. It can hold other nodes and define layout rules.

```pastel
frame sidebar {
    width   = 240
    height  = fill
    fill    = #FFFFFF
    layout  = vertical
    gap     = 8

    text "Menu" { size = 18, weight = bold, color = #111 }
    text "Home" { size = 14, color = #333 }
}
```

### Text

```pastel
text "content string" {
    size   = 16
    weight = medium
    color  = #333333
}
```

The quoted string after `text` is the content. Text nodes are always leaf nodes.

```pastel
// Inline form (comma-separated)
text "Hello" { size = 24, weight = bold, color = #111 }

// Block form
text "A longer description of something" {
    size        = 14
    color       = #666666
    font        = "Inter"
    align       = center
    line-height = 1.5
}
```

### Image

```pastel
asset logo = image("./assets/logo.svg")

image logo {
    width  = 120
    height = 32
}
```

The identifier after `image` references a declared `asset`.

### Shape

```pastel
shape divider {
    type   = line
    width  = fill
    height = 1
    fill   = #E0E0E0
}

shape avatar-bg {
    type   = ellipse
    width  = 48
    height = 48
    fill   = #0066FF
}
```

## Name and Label Rules

Nodes have an optional **name** (identifier) and an optional **label** (quoted string).

```pastel
frame sidebar { ... }           // name = "sidebar", no label
text "Hello" { ... }            // no name, label = "Hello"
frame hero-section { ... }      // name = "hero-section"
```

**Names:**
- Must be valid identifiers: letters, digits, hyphens, underscores
- Used as the node's `id` in the IR
- Should be unique within siblings (duplicates generate auto-ids)

**Labels:**
- Quoted strings, only meaningful for `text` nodes
- For text nodes, the label becomes the rendered content

## Nesting Rules

- `frame` can contain any node type (including other frames)
- `text`, `image`, `shape` are leaf nodes (no children)
- Top-level nodes sit directly in the document (no implicit root frame)
- Nesting depth is unlimited

```pastel
frame outer {
    layout = vertical

    frame inner {
        layout = horizontal

        text "Left" { size = 14, color = #333 }
        text "Right" { size = 14, color = #333 }
    }

    shape divider {
        type = line, width = fill, height = 1, fill = #DDD
    }
}
```
