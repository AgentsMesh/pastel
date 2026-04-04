use std::path::Path;

use pastel_lang::formatter::Formatter;
use pastel_lang::lexer::Lexer;
use pastel_lang::parser::Parser;

pub fn run(file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(file)?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let formatted = Formatter::new().format(&program);

    std::fs::write(file, &formatted)?;
    println!("formatted {}", file.display());
    Ok(())
}
