use std::collections::HashMap;

use crate::ast::{Expression, NodeDecl};
use crate::error::{ErrorKind, PastelError};
use crate::ir::node::IrNode;

use super::builder::IrBuilder;

impl IrBuilder {
    /// Expand a `use` node by substituting component params into body.
    pub(super) fn expand_component(&mut self, use_node: &NodeDecl) -> Result<IrNode, PastelError> {
        let component_name = use_node.name.as_deref().unwrap_or("");

        let comp = self
            .components
            .get(component_name)
            .cloned()
            .ok_or_else(|| {
                PastelError::new(
                    ErrorKind::UndefinedVariable,
                    format!("undefined component '{}'", component_name),
                )
                .with_span(use_node.span)
                .with_hint("define with: component name(params) { ... }")
            })?;

        // Build param → value mapping
        let mut param_values: HashMap<String, Expression> = HashMap::new();
        let mut positional_idx = 0;
        for attr in &use_node.attrs {
            if attr.key.starts_with("__arg_") {
                if positional_idx < comp.params.len() {
                    param_values
                        .insert(comp.params[positional_idx].name.clone(), attr.value.clone());
                }
                positional_idx += 1;
            } else {
                param_values.insert(attr.key.clone(), attr.value.clone());
            }
        }

        // Apply defaults for missing params
        for param in &comp.params {
            if !param_values.contains_key(&param.name) {
                if let Some(default) = &param.default {
                    param_values.insert(param.name.clone(), default.clone());
                }
            }
        }

        // Clone and rewrite the component body with substituted params
        let rewritten_body = self.rewrite_node(&comp.body, &param_values);

        // Register params as variables for property resolution
        for (name, value) in &param_values {
            self.vars.register(name.clone(), value.clone());
        }

        self.build_node(&rewritten_body)
    }

    /// Rewrite a NodeDecl by substituting param references in labels and attrs.
    fn rewrite_node(&self, node: &NodeDecl, params: &HashMap<String, Expression>) -> NodeDecl {
        let mut rewritten = node.clone();

        // Substitute label if it's a param reference (e.g. text "label" → text "Sign Up")
        if let Some(label) = &node.label {
            if let Some(value) = params.get(label) {
                if let Expression::String(s) = &self.vars.resolve(value) {
                    rewritten.label = Some(s.clone());
                }
            }
        }

        // If name is a param reference and the node is a text node,
        // convert name to label (e.g. `text label { ... }` → `text "Sign Up" { ... }`)
        if node.label.is_none() {
            if let Some(name) = &node.name {
                if let Some(value) = params.get(name) {
                    let resolved = self.vars.resolve(value);
                    if let Expression::String(s) = &resolved {
                        rewritten.label = Some(s.clone());
                        rewritten.name = None;
                    }
                }
            }
        }

        // Substitute attribute values that reference params
        for attr in &mut rewritten.attrs {
            if let Expression::Ident(ref name) = attr.value {
                if let Some(value) = params.get(name) {
                    attr.value = value.clone();
                }
            }
        }

        // Recurse into children
        rewritten.children = node
            .children
            .iter()
            .map(|c| self.rewrite_node(c, params))
            .collect();

        rewritten
    }
}
