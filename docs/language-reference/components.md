# Components

Components are reusable design blocks with parameters. They are expanded at compile time like macros.

## Declaration Syntax

```pastel
component name(param1, param2 = default_value) {
    // body: a single root node
}
```

- **name** — component identifier, used with `use`
- **params** — positional parameters; optional defaults after `=`
- **body** — exactly one root node (typically a `frame`)

### Example

```pastel
component button(label, color = #0066FF) {
    frame {
        padding = [10, 24]
        fill    = color
        radius  = 8

        text label { size = 14, weight = medium, color = #FFFFFF }
    }
}
```

## Instantiation with `use`

```pastel
use component_name(args...)
```

Positional arguments match param order. Named arguments override by name.

```pastel
// Positional only
use button("Sign Up")

// Positional + named override
use button("Learn More", color = #333333)
```

## How Expansion Works

Components are **compile-time macros**. The semantic analyzer:

1. Looks up the component definition by name
2. Matches positional and named arguments to parameters
3. Applies default values for missing parameters
4. Rewrites the component body by substituting param references
5. Builds the rewritten body as a normal node tree

After expansion, the IR contains no component references — only concrete frame/text/image/shape nodes.

### Before expansion (source)

```pastel
component badge(label, color = #0066FF) {
    frame {
        padding = [4, 12]
        fill    = color
        radius  = 12
        text label { size = 12, weight = medium, color = #FFFFFF }
    }
}

use badge("New")
use badge("Hot", color = #FF4444)
```

### After expansion (IR)

```json
[
  {
    "type": "frame",
    "padding": [4, 12, 4, 12],
    "fill": { "type": "solid", "color": "#0066FF" },
    "corner_radius": [12, 12, 12, 12],
    "children": [
      { "type": "text", "content": "New", "font_size": 12.0 }
    ]
  },
  {
    "type": "frame",
    "padding": [4, 12, 4, 12],
    "fill": { "type": "solid", "color": "#FF4444" },
    "corner_radius": [12, 12, 12, 12],
    "children": [
      { "type": "text", "content": "Hot", "font_size": 12.0 }
    ]
  }
]
```

## Parameter Substitution Rules

- **Text labels**: if a text node uses a param name as its label or name, the param value is substituted as the text content
- **Attribute values**: if an attribute references a param by `Ident`, the param value replaces it
- **Nested substitution**: children are rewritten recursively

```pastel
component card(title, subtitle, accent = #0066FF) {
    frame {
        padding = [16, 20]
        fill    = #FFFFFF
        radius  = 8
        layout  = vertical
        gap     = 4

        text title { size = 18, weight = bold, color = #111 }
        text subtitle { size = 14, color = #666 }

        shape accent-line {
            type = line, width = fill, height = 2, fill = accent
        }
    }
}

use card("Revenue", "$48K")
use card("Users", "12,345", accent = #52C41A)
```

## Limitations

- **Non-Turing-complete**: no loops, conditionals, or recursion
- **No nested components**: a component body cannot contain `use` of another component (single-level expansion)
- **Single root node**: the body must be exactly one node
- **No computed expressions**: param values are literals or variable references, not arithmetic
- Components cannot modify the canvas declaration
