use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub file: String,
    pub node: String,
    pub rule: String,
    pub value: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LintReport {
    pub violations: Vec<Violation>,
}

impl LintReport {
    pub fn new() -> Self {
        Self { violations: Vec::new() }
    }

    pub fn add(&mut self, v: Violation) {
        self.violations.push(v);
    }

    pub fn format_text(&self) -> String {
        let mut out = String::new();
        for v in &self.violations {
            out.push_str(&format!("{}  [{}] {} ({})\n", v.file, v.node, v.message, v.rule));
            if let Some(s) = &v.suggestion {
                out.push_str(&format!("  suggestion: {}\n", s));
            }
        }
        if self.violations.is_empty() {
            out.push_str("  All design values match token definitions\n");
        } else {
            out.push_str(&format!("\n{} violation(s) found\n", self.violations.len()));
        }
        out
    }

    pub fn format_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}
