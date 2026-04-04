use crate::token::Span;

/// Top-level program.
#[derive(Debug, Clone)]
pub struct Program {
    pub canvas: Option<CanvasDecl>,
    pub assets: Vec<AssetDecl>,
    pub variables: Vec<LetDecl>,
    pub includes: Vec<IncludeDecl>,
    pub components: Vec<ComponentDecl>,
    pub token_blocks: Vec<TokenBlockDecl>,
    pub nodes: Vec<NodeDecl>,
    pub pages: Vec<PageDecl>,
}

#[derive(Debug, Clone)]
pub struct PageDecl {
    pub name: String,
    pub nodes: Vec<NodeDecl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CanvasDecl {
    pub name: String,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AssetDecl {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IncludeDecl {
    pub path: String,
    pub span: Span,
}

/// Reusable component definition (compile-time macro).
#[derive(Debug, Clone)]
pub struct ComponentDecl {
    pub name: String,
    pub params: Vec<ComponentParam>,
    pub body: NodeDecl,
    pub span: Span,
}

/// A `token` block defining grouped design tokens.
#[derive(Debug, Clone)]
pub struct TokenBlockDecl {
    pub name: String,
    pub entries: Vec<TokenEntry>,
    pub span: Span,
}

/// A single key = value entry inside a `token` block.
#[derive(Debug, Clone)]
pub struct TokenEntry {
    pub key: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ComponentParam {
    pub name: String,
    pub default: Option<Expression>,
}

/// A design node (frame, text, image, shape, use).
#[derive(Debug, Clone)]
pub struct NodeDecl {
    pub kind: NodeKind,
    pub name: Option<String>,
    pub label: Option<String>,
    pub attrs: Vec<Attribute>,
    pub children: Vec<NodeDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Frame,
    Text,
    Image,
    Shape,
    Use, // Component instantiation
}

/// For `use` nodes: stores the component call args.
#[derive(Debug, Clone)]
pub struct UseArgs {
    pub component_name: String,
    pub positional: Vec<Expression>,
    pub named: Vec<(String, Expression)>,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    String(String),
    Color(String),
    Bool(bool),
    Ident(String),
    Array(Vec<Expression>),
    Object(Vec<(String, Expression)>),
    FunctionCall { name: String, args: Vec<Expression> },
}
