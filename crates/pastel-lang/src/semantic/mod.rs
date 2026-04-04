mod resolve;
mod resolve_extra;
mod builder;
mod expand;
mod validate;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::Program;
use crate::error::{ErrorKind, PastelError};
use crate::ir::{IrAsset, IrCanvas, IrDocument};
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

        Ok(IrDocument {
            version: 1,
            canvas,
            assets: assets.into_values().collect(),
            nodes,
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

            // Recursively process includes in the included file
            let inc_dir = canonical.parent().unwrap_or(base_dir);
            self.process_includes(&mut included, inc_dir, visited)?;

            // Merge declarations into the main program
            program.variables.extend(included.variables);
            program.assets.extend(included.assets);
            program.components.extend(included.components);
        }

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
}
