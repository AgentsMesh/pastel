# Variables and Includes

## Variables (`let`)

Variables bind a name to a value. They are resolved at compile time.

### Syntax

```pastel
let name = value
```

### Variable Types

| Type | Example |
|------|---------|
| Number (integer) | `let spacing = 16` |
| Number (float) | `let opacity = 0.8` |
| String | `let font = "Inter"` |
| Color | `let primary = #0066FF` |
| Array | `let shadow = [0, 2, 8, #00000012]` |

### Usage

Variables are referenced by their identifier name in attribute values.

```pastel
let primary = #0066FF
let radius  = 8
let gap_md  = 16

frame card {
    fill   = primary       // resolves to #0066FF
    radius = radius        // resolves to 8
    gap    = gap_md        // resolves to 16

    text "Hello" { size = 14, color = primary }
}
```

### Resolution Rules

- Variables are resolved recursively (a variable can reference another variable)
- All variables must be defined before the IR is built
- Undefined variable references produce an error
- Variables from included files are merged into the current scope

```pastel
let base = #0066FF
let primary = base         // resolves to #0066FF
```

### Scope

Variables have file-level scope. There is no block scoping — all `let` declarations are global within the compilation unit (including merged includes).

## Includes (`include`)

Include merges declarations from another `.pastel` file.

### Syntax

```pastel
include "./path/to/file.pastel"
```

The path is a quoted string, relative to the current file's directory.

### What Gets Merged

| Declaration | Merged? |
|-------------|:-------:|
| `let` variables | Yes |
| `asset` declarations | Yes |
| `component` definitions | Yes |
| `canvas` | No (ignored) |
| Top-level nodes | No (ignored) |

### Example

**shared.pastel**

```pastel
let primary   = #0066FF
let secondary = #52C41A
let radius    = 8

component badge(label, color = primary) {
    frame {
        padding = [4, 12]
        fill    = color
        radius  = 12
        text label { size = 12, weight = medium, color = #FFFFFF }
    }
}
```

**main.pastel**

```pastel
canvas "app" { width = 1440, height = 900, background = #F8F9FA }

include "./shared.pastel"

frame hero {
    width  = fill
    fill   = primary           // from shared.pastel

    use badge("New Feature")   // component from shared.pastel
}
```

### File Resolution Rules

1. Paths are relative to the **including file's directory**, not the workspace root
2. The path must be a quoted string literal (no variables or expressions)
3. The file must exist and be readable
4. The file must be valid `.pastel` syntax

### Nested Includes

Included files can themselves include other files. Resolution is recursive:

```
main.pastel
  └── include "./lib/theme.pastel"
        └── include "./colors.pastel"    (resolved relative to lib/)
```

### Circular Include Detection

The compiler tracks visited file paths (canonicalized). If a file is included a second time, the compiler reports:

```
error[CircularInclude]: circular include detected: './shared.pastel'
  --> line 3:1
```

This applies to both direct and transitive cycles (A includes B includes A).
