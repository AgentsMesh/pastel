mod resolve;
mod resolve_extra;
mod resolve_fill;
mod builder;
mod builder_leaf;
mod expand;
mod validate;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::Program;
use crate::error::{ErrorKind, PastelError};
use crate::ir::{IrAsset, IrCanvas, IrDocument, IrPage, IrTokenGroup, IrTokenEntry, IrTokenValue};
use crate::lexer::Lexer;
use crate::parser::Parser;

pub use resolve::{PropertyResolver, VariableResolver};
pub use builder::IrBuilder;

/// Top-level semantic analysis entry point.
pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, program: &Program) -> Result<IrDocument, PastelError> {
        self.analyze_with_base(program, None)
    }

    pub fn analyze_with_base(
        &self,
        program: &Program,
        base_dir: Option<&Path>,
    ) -> Result<IrDocument, PastelError> {
        let mut merged = program.clone();

        // 1. Process includes (merge variables, assets, components)
        if !program.includes.is_empty() {
            let base = base_dir.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
            let mut visited = HashSet::new();
            self.process_includes(&mut merged, &base, &mut visited)?;
        }

        // 2. Register variables
        let mut vars = VariableResolver::new();
        for var in &merged.variables {
            vars.register(var.name.clone(), var.value.clone());
        }

        // 2b. Register token blocks as flattened variables (e.g. "colors.primary")
        let mut ir_tokens = Vec::new();
        for block in &merged.token_blocks {
            let mut ir_entries = Vec::new();
            for entry in &block.entries {
                let flat_key = format!("{}.{}", block.name, entry.key);
                vars.register(flat_key, entry.value.clone());
                ir_entries.push(IrTokenEntry {
                    key: entry.key.clone(),
                    value: Self::expr_to_token_value(&entry.value),
                });
            }
            ir_tokens.push(IrTokenGroup {
                name: block.name.clone(),
                entries: ir_entries,
            });
        }

        // 3. Register assets
        let mut assets = HashMap::new();
        for asset in &merged.assets {
            assets.insert(
                asset.name.clone(),
                IrAsset {
                    id: asset.name.clone(),
                    kind: asset.kind.clone(),
                    path: asset.path.clone(),
                },
            );
        }

        // 4. Resolve canvas
        let canvas = self.resolve_canvas(&merged, &vars)?;

        // 5. Build IR nodes (with components for expansion)
        let mut builder = IrBuilder::new(vars, assets.clone(), merged.components.clone());
        let mut nodes = Vec::new();
        for node in &merged.nodes {
            nodes.push(builder.build_node(node)?);
        }

        // 6. Build pages (if any)
        let mut pages = Vec::new();
        for page in &merged.pages {
            let mut page_nodes = Vec::new();
            for node in &page.nodes {
                page_nodes.push(builder.build_node(node)?);
            }
            pages.push(IrPage { name: page.name.clone(), nodes: page_nodes });
        }

        Ok(IrDocument {
            version: 1,
            canvas,
            assets: assets.into_values().collect(),
            tokens: ir_tokens,
            nodes,
            pages,
        })
    }

    fn process_includes(
        &self,
        program: &mut Program,
        base_dir: &Path,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<(), PastelError> {
        let includes: Vec<_> = program.includes.drain(..).collect();

        for inc in includes {
            let resolved = base_dir.join(&inc.path);
            let canonical = resolved.canonicalize().map_err(|e| {
                PastelError::new(
                    ErrorKind::IncludeError,
                    format!("cannot resolve include '{}': {}", inc.path, e),
                )
                .with_span(inc.span)
            })?;

            if !visited.insert(canonical.clone()) {
                return Err(PastelError::new(
                    ErrorKind::CircularInclude,
                    format!("circular include detected: '{}'", inc.path),
                )
                .with_span(inc.span));
            }

            let source = std::fs::read_to_string(&canonical).map_err(|e| {
                PastelError::new(
                    ErrorKind::IncludeError,
                    format!("cannot read '{}': {}", inc.path, e),
                )
                .with_span(inc.span)
            })?;

            let tokens = Lexer::new(&source).tokenize()?;
            let mut included = Parser::new(tokens).parse()?;

            let inc_dir = canonical.parent().unwrap_or(base_dir);
            self.process_includes(&mut included, inc_dir, visited)?;

            if let Some(ns) = &inc.namespace {
                // Namespaced include: prefix all names with "ns."
                // Also keep originals for internal component references
                let orig_vars = included.variables.clone();
                let orig_tokens = included.token_blocks.clone();
                for var in &mut included.variables {
                    var.name = format!("{}.{}", ns, var.name);
                }
                for asset in &mut included.assets {
                    asset.name = format!("{}.{}", ns, asset.name);
                }
                for comp in &mut included.components {
                    comp.name = format!("{}.{}", ns, comp.name);
                }
                for block in &mut included.token_blocks {
                    block.name = format!("{}.{}", ns, block.name);
                }
                // Add both prefixed (for external use) and originals (for component internals)
                program.variables.extend(included.variables);
                program.variables.extend(orig_vars);
                program.assets.extend(included.assets);
                program.components.extend(included.components);
                program.token_blocks.extend(included.token_blocks);
                program.token_blocks.extend(orig_tokens);
            } else {
                // Bare include: merge with conflict detection
                self.merge_with_conflict_check(program, &included, &inc)?;
            }
        }

        Ok(())
    }

    fn merge_with_conflict_check(
        &self, target: &mut Program, source: &Program, inc: &crate::ast::IncludeDecl,
    ) -> Result<(), PastelError> {
        // Check variable conflicts
        for var in &source.variables {
            if target.variables.iter().any(|v| v.name == var.name) {
                return Err(PastelError::new(
                    ErrorKind::DuplicateId,
                    format!("variable '{}' already defined (conflict from include '{}')", var.name, inc.path),
                ).with_span(inc.span)
                .with_hint(format!("use: include \"{}\" as <namespace>", inc.path)));
            }
        }
        // Check component conflicts
        for comp in &source.components {
            if target.components.iter().any(|c| c.name == comp.name) {
                return Err(PastelError::new(
                    ErrorKind::DuplicateId,
                    format!("component '{}' already defined (conflict from include '{}')", comp.name, inc.path),
                ).with_span(inc.span)
                .with_hint(format!("use: include \"{}\" as <namespace>", inc.path)));
            }
        }
        // Check token block conflicts
        for block in &source.token_blocks {
            if target.token_blocks.iter().any(|b| b.name == block.name) {
                return Err(PastelError::new(
                    ErrorKind::DuplicateId,
                    format!("token block '{}' already defined (conflict from include '{}')", block.name, inc.path),
                ).with_span(inc.span)
                .with_hint(format!("use: include \"{}\" as <namespace>", inc.path)));
            }
        }
        target.variables.extend(source.variables.clone());
        target.assets.extend(source.assets.clone());
        target.components.extend(source.components.clone());
        target.token_blocks.extend(source.token_blocks.clone());
        Ok(())
    }

    fn resolve_canvas(
        &self,
        program: &Program,
        vars: &VariableResolver,
    ) -> Result<IrCanvas, PastelError> {
        let p = PropertyResolver::new(vars);

        if let Some(c) = &program.canvas {
            let mut width = 1440u32;
            let mut height = 900u32;
            let mut background = None;

            for attr in &c.attrs {
                match attr.key.as_str() {
                    "width" => width = p.resolve_u32(&attr.value).map_err(|e| e.with_span(attr.span))?,
                    "height" => height = p.resolve_u32(&attr.value).map_err(|e| e.with_span(attr.span))?,
                    "background" => background = Some(p.resolve_color(&attr.value).map_err(|e| e.with_span(attr.span))?),
                    _ => {}
                }
            }

            Ok(IrCanvas {
                name: c.name.clone(),
                width,
                height,
                background,
            })
        } else {
            Ok(IrCanvas {
                name: "untitled".into(),
                width: 1440,
                height: 900,
                background: None,
            })
        }
    }

    /// Convert an AST expression to an IR token value for downstream consumption.
    fn expr_to_token_value(expr: &crate::ast::Expression) -> IrTokenValue {
        use crate::ast::Expression;
        match expr {
            Expression::Integer(n) => IrTokenValue::Number(*n as f64),
            Expression::Float(n) => IrTokenValue::Number(*n),
            Expression::Color(c) => IrTokenValue::Color(format!("#{c}")),
            Expression::String(s) => IrTokenValue::String(s.clone()),
            Expression::Bool(b) => IrTokenValue::Bool(*b),
            Expression::Ident(s) => IrTokenValue::String(s.clone()),
            Expression::Array(items) => {
                IrTokenValue::Array(items.iter().map(Self::expr_to_token_value).collect())
            }
            Expression::Object(entries) => {
                IrTokenValue::Object(
                    entries.iter()
                        .map(|(k, v)| (k.clone(), Self::expr_to_token_value(v)))
                        .collect()
                )
            }
            Expression::FunctionCall { name, args } => {
                let parts: Vec<String> = args.iter().map(|_| "...".into()).collect();
                IrTokenValue::String(format!("{}({})", name, parts.join(", ")))
            }
        }
    }
}
