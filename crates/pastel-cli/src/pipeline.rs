use std::path::Path;

use pastel_lang::error::PastelError;
use pastel_lang::ir::IrDocument;
use pastel_lang::lexer::Lexer;
use pastel_lang::parser::Parser;
use pastel_lang::semantic::SemanticAnalyzer;

/// Compile a .pastel source string into IR (used by tests).
#[allow(dead_code)]
pub fn compile(source: &str) -> Result<IrDocument, PastelError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let analyzer = SemanticAnalyzer::new();
    let ir = analyzer.analyze(&program)?;

    Ok(ir)
}

/// Read a .pastel file and compile it.
pub fn compile_file(path: &Path) -> Result<IrDocument, PastelError> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        PastelError::new(
            pastel_lang::error::ErrorKind::UnexpectedChar, // TODO: add IO error kind
            format!("failed to read file '{}': {}", path.display(), e),
        )
    })?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let base_dir = path.parent().unwrap_or(Path::new("."));
    let analyzer = SemanticAnalyzer::new();
    let ir = analyzer.analyze_with_base(&program, Some(base_dir))?;

    Ok(ir)
}
