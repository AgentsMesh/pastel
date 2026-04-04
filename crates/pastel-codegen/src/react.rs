use pastel_lang::ir::IrDocument;

/// Generate React component + tokens CSS from IR.
pub fn generate_react(ir: &IrDocument) -> (String, String) {
    let tokens_css = crate::tokens::generate_css(&ir.tokens);
    let html = crate::html::generate_html(ir);

    // Wrap HTML body content into a React component
    let body_start = html.find("<body>").map(|i| i + 7).unwrap_or(0);
    let body_end = html.find("</body>").unwrap_or(html.len());
    let body = &html[body_start..body_end];

    let name = pascal_case(&ir.canvas.name);
    let component = format!(
        r#"import './tokens.css';

export function {name}() {{
  return (
    <>
{body}    </>
  );
}}
"#
    );

    (component, tokens_css)
}

fn pascal_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_' || c == ' ')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase(),
            }
        })
        .collect()
}
