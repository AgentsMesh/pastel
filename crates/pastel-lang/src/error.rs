use crate::token::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct PastelError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    // Lexer errors
    UnexpectedChar,
    UnterminatedString,
    InvalidColor,
    InvalidNumber,

    // Parser errors
    UnexpectedToken,
    ExpectedToken,
    ExpectedDeclaration,

    // Semantic errors
    UndefinedVariable,
    UndefinedAsset,
    DuplicateId,
    TypeMismatch,
    InvalidValue,
    CircularInclude,
    IncludeError,
}

impl fmt::Display for PastelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error[{:?}]: {}", self.kind, self.message)?;
        if let Some(span) = &self.span {
            write!(f, "\n  --> line {}:{}", span.line, span.col)?;
        }
        if let Some(hint) = &self.hint {
            write!(f, "\n  = help: {}", hint)?;
        }
        Ok(())
    }
}

impl std::error::Error for PastelError {}

impl PastelError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            span: None,
            hint: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}
