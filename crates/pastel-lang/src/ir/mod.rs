pub mod node;
pub mod style;
pub mod extra;

use serde::Serialize;

use self::node::IrNode;

/// Fully resolved document, ready for rendering.
#[derive(Debug, Clone, Serialize)]
pub struct IrDocument {
    pub version: u32,
    pub canvas: IrCanvas,
    pub assets: Vec<IrAsset>,
    pub nodes: Vec<IrNode>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pages: Vec<IrPage>,
}

/// A named page containing its own set of nodes.
#[derive(Debug, Clone, Serialize)]
pub struct IrPage {
    pub name: String,
    pub nodes: Vec<IrNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IrCanvas {
    pub name: String,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<style::Color>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IrAsset {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub path: String,
}
