use crate::ast::*;

/// AST round-trip formatter for .pastel source files.
pub struct Formatter {
    buf: String,
}

impl Formatter {
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    pub fn format(mut self, program: &Program) -> String {
        let mut first = true;

        for inc in &program.includes {
            if !first { self.buf.push('\n'); }
            self.fmt_include(inc);
            first = false;
        }

        for asset in &program.assets {
            if !first { self.buf.push('\n'); }
            self.fmt_asset(asset);
            first = false;
        }

        for var in &program.variables {
            if !first { self.buf.push('\n'); }
            self.fmt_let(var);
            first = false;
        }

        if let Some(canvas) = &program.canvas {
            if !first { self.buf.push('\n'); }
            self.fmt_canvas(canvas);
            first = false;
        }

        for comp in &program.components {
            if !first { self.buf.push('\n'); }
            self.fmt_component(comp);
            first = false;
        }

        for node in &program.nodes {
            if !first { self.buf.push('\n'); }
            self.fmt_node(node, 0);
            first = false;
        }

        if !self.buf.is_empty() && !self.buf.ends_with('\n') {
            self.buf.push('\n');
        }
        self.buf
    }

    fn fmt_include(&mut self, inc: &IncludeDecl) {
        self.buf.push_str(&format!("include \"{}\"\n", inc.path));
    }

    fn fmt_asset(&mut self, a: &AssetDecl) {
        self.buf.push_str(&format!(
            "asset {} = {}(\"{}\")\n",
            a.name, a.kind, a.path
        ));
    }

    fn fmt_let(&mut self, v: &LetDecl) {
        self.buf.push_str(&format!("let {} = {}\n", v.name, fmt_expr(&v.value)));
    }

    fn fmt_canvas(&mut self, c: &CanvasDecl) {
        self.buf.push_str(&format!("canvas \"{}\" {{\n", c.name));
        for attr in &c.attrs {
            self.buf.push_str(&format!("    {} = {}\n", attr.key, fmt_expr(&attr.value)));
        }
        self.buf.push_str("}\n");
    }

    fn fmt_component(&mut self, comp: &ComponentDecl) {
        self.buf.push_str(&format!("component {}(", comp.name));
        for (i, p) in comp.params.iter().enumerate() {
            if i > 0 { self.buf.push_str(", "); }
            self.buf.push_str(&p.name);
            if let Some(d) = &p.default {
                self.buf.push_str(&format!(" = {}", fmt_expr(d)));
            }
        }
        self.buf.push_str(") {\n");
        self.fmt_node(&comp.body, 1);
        self.buf.push_str("}\n");
    }

    fn fmt_node(&mut self, node: &NodeDecl, depth: usize) {
        let indent = "    ".repeat(depth);
        let kind = match node.kind {
            NodeKind::Frame => "frame",
            NodeKind::Text => "text",
            NodeKind::Image => "image",
            NodeKind::Shape => "shape",
            NodeKind::Use => "use",
        };

        // Text nodes with a label and short attrs can be inline
        if node.kind == NodeKind::Text {
            if let Some(label) = &node.label {
                if node.children.is_empty() && node.attrs.len() <= 3 && !node.attrs.is_empty() {
                    let total_len: usize = node.attrs.iter()
                        .map(|a| a.key.len() + fmt_expr(&a.value).len() + 5)
                        .sum();
                    if total_len < 60 {
                        let pairs: Vec<String> = node.attrs.iter()
                            .map(|a| format!("{} = {}", a.key, fmt_expr(&a.value)))
                            .collect();
                        self.buf.push_str(&format!(
                            "{}text \"{}\" {{ {} }}\n",
                            indent, label, pairs.join(", ")
                        ));
                        return;
                    }
                }
            }
        }

        // Build the opening tag
        self.buf.push_str(&format!("{}{}", indent, kind));

        if node.kind == NodeKind::Text {
            if let Some(label) = &node.label {
                self.buf.push_str(&format!(" \"{}\"", label));
            }
        } else if let Some(name) = &node.name {
            self.buf.push_str(&format!(" {}", name));
        }

        if node.attrs.is_empty() && node.children.is_empty() {
            self.buf.push_str(" {}\n");
            return;
        }

        self.buf.push_str(" {\n");
        let inner = "    ".repeat(depth + 1);
        for attr in &node.attrs {
            self.buf.push_str(&format!("{}{} = {}\n", inner, attr.key, fmt_expr(&attr.value)));
        }
        for child in &node.children {
            self.fmt_node(child, depth + 1);
        }
        self.buf.push_str(&format!("{}}}\n", indent));
    }
}

fn fmt_expr(expr: &Expression) -> String {
    match expr {
        Expression::Integer(n) => n.to_string(),
        Expression::Float(n) => format_float(*n),
        Expression::String(s) => format!("\"{}\"", s),
        Expression::Color(c) => format!("#{}", c),
        Expression::Bool(b) => b.to_string(),
        Expression::Ident(s) => s.clone(),
        Expression::Array(items) => {
            let parts: Vec<String> = items.iter().map(fmt_expr).collect();
            format!("[{}]", parts.join(", "))
        }
    }
}

fn format_float(n: f64) -> String {
    if n == n.floor() && n.abs() < 1e15 {
        format!("{:.1}", n)
    } else {
        n.to_string()
    }
}
