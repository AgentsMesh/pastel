use std::collections::BTreeMap;
use std::path::Path;

use pastel_lang::ir::{IrDocument, IrTokenGroup, IrTokenValue};

/// Generate CSS custom properties and JSON from design tokens.
pub fn generate(ir: &IrDocument, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;

    let css = generate_css(&ir.tokens);
    std::fs::write(output_dir.join("tokens.css"), css)?;

    let json = generate_json(&ir.tokens)?;
    std::fs::write(output_dir.join("tokens.json"), json)?;

    Ok(())
}

/// Build CSS custom properties string from token groups.
pub fn generate_css(groups: &[IrTokenGroup]) -> String {
    let mut lines = Vec::new();
    lines.push(":root {".to_string());

    for group in groups {
        for entry in &group.entries {
            let var_name = format!("--{}-{}", group.name, entry.key);
            if let Some(val) = token_value_to_css(&entry.value) {
                lines.push(format!("  {}: {};", var_name, val));
            }
        }
    }

    lines.push("}".to_string());
    lines.join("\n")
}

/// Build JSON representation from token groups.
pub fn generate_json(groups: &[IrTokenGroup]) -> Result<String, Box<dyn std::error::Error>> {
    let mut map = BTreeMap::new();

    for group in groups {
        let mut entries = BTreeMap::new();
        for entry in &group.entries {
            entries.insert(entry.key.clone(), token_value_to_json(&entry.value));
        }
        map.insert(
            group.name.clone(),
            serde_json::Value::Object(entries.into_iter().collect()),
        );
    }

    let json = serde_json::to_string_pretty(&map)?;
    Ok(json)
}

/// Convert an IR token value to a CSS value string.
fn token_value_to_css(val: &IrTokenValue) -> Option<String> {
    match val {
        IrTokenValue::Number(n) => {
            if *n == (*n as i64) as f64 {
                Some(format!("{}px", *n as i64))
            } else {
                Some(format!("{}px", n))
            }
        }
        IrTokenValue::Color(c) => Some(c.clone()),
        IrTokenValue::String(s) => Some(s.clone()),
        IrTokenValue::Bool(b) => Some(b.to_string()),
        IrTokenValue::Array(arr) => {
            // Shadow arrays: [x, y, blur, color]
            if arr.len() == 4 {
                shadow_array_to_css(arr)
            } else {
                None
            }
        }
        IrTokenValue::Object(_) => None, // Complex objects skip CSS
    }
}

/// Convert a shadow array [x, y, blur, color] to CSS box-shadow value.
fn shadow_array_to_css(arr: &[IrTokenValue]) -> Option<String> {
    let x = match &arr[0] {
        IrTokenValue::Number(n) => *n,
        _ => return None,
    };
    let y = match &arr[1] {
        IrTokenValue::Number(n) => *n,
        _ => return None,
    };
    let blur = match &arr[2] {
        IrTokenValue::Number(n) => *n,
        _ => return None,
    };
    let color = match &arr[3] {
        IrTokenValue::Color(c) => c.clone(),
        _ => return None,
    };
    Some(format!(
        "{}px {}px {}px {}",
        x as i64, y as i64, blur as i64, color
    ))
}

/// Convert an IR token value to a JSON value.
fn token_value_to_json(val: &IrTokenValue) -> serde_json::Value {
    match val {
        IrTokenValue::Number(n) => serde_json::json!(*n),
        IrTokenValue::Color(c) => serde_json::json!(c),
        IrTokenValue::String(s) => serde_json::json!(s),
        IrTokenValue::Bool(b) => serde_json::json!(b),
        IrTokenValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(token_value_to_json).collect())
        }
        IrTokenValue::Object(pairs) => {
            let map: serde_json::Map<String, serde_json::Value> = pairs
                .iter()
                .map(|(k, v)| (k.clone(), token_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pastel_lang::ir::{IrTokenEntry, IrTokenGroup};

    #[test]
    fn css_color_tokens() {
        let groups = vec![IrTokenGroup {
            name: "color".into(),
            entries: vec![IrTokenEntry {
                key: "primary".into(),
                value: IrTokenValue::Color("#0066FF".into()),
            }],
        }];
        let css = generate_css(&groups);
        assert!(css.contains("--color-primary: #0066FF;"));
    }

    #[test]
    fn css_spacing_tokens() {
        let groups = vec![IrTokenGroup {
            name: "spacing".into(),
            entries: vec![IrTokenEntry {
                key: "sm".into(),
                value: IrTokenValue::Number(8.0),
            }],
        }];
        let css = generate_css(&groups);
        assert!(css.contains("--spacing-sm: 8px;"));
    }

    #[test]
    fn json_output() {
        let groups = vec![IrTokenGroup {
            name: "colors".into(),
            entries: vec![IrTokenEntry {
                key: "primary".into(),
                value: IrTokenValue::Color("#0066FF".into()),
            }],
        }];
        let json = generate_json(&groups).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["colors"]["primary"], "#0066FF");
    }

    #[test]
    fn shadow_array_css() {
        let groups = vec![IrTokenGroup {
            name: "shadow".into(),
            entries: vec![IrTokenEntry {
                key: "sm".into(),
                value: IrTokenValue::Array(vec![
                    IrTokenValue::Number(0.0),
                    IrTokenValue::Number(1.0),
                    IrTokenValue::Number(3.0),
                    IrTokenValue::Color("#0000000D".into()),
                ]),
            }],
        }];
        let css = generate_css(&groups);
        assert!(css.contains("--shadow-sm: 0px 1px 3px #0000000D;"));
    }
}
