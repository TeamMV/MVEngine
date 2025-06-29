use hashbrown::HashMap;
use mvutils::TryFromString;
use crate::ui::geometry::shape::msfx::lexer::{MSFXKeyword, MSFXOperator};

pub struct MSFXAST {
    pub elements: Vec<MSFXStmt>
}

#[derive(Debug, Clone)]
pub enum MSFXStmt {
    Block(Vec<MSFXStmt>),
    Let(DeclStmt),
    Assign(DeclStmt),
    For(ForStmt),
    While(WhileStmt),
    If(IfStmt),
    ExportShape(ExportShapeStmt),
    ExportAdaptive(ExportAdaptiveStmt),
    Break,
    Continue,
    Expr(MSFXExpr),
    Nop,
}

#[derive(Debug, Clone)]
pub struct DeclStmt {
    pub name: String,
    pub expr: MSFXExpr
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub varname: String,
    pub start: MSFXExpr,
    pub end: MSFXExpr,
    pub step: MSFXExpr,
    pub block: Box<MSFXStmt>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub cond: MSFXExpr,
    pub block: Box<MSFXStmt>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub cond: MSFXExpr,
    pub true_block: Box<MSFXStmt>,
    pub false_block: Box<MSFXStmt>,
}

#[derive(TryFromString, Debug, Default, Clone)]
pub enum ExportTarget {
    #[default]
    All,
    Bl,
    Br,
    Tl,
    Tr,
    C
}

impl ExportTarget {
    pub fn from_keyword(keyword: MSFXKeyword) -> Result<Self, String> {
        match keyword {
            MSFXKeyword::All => Ok(Self::All),
            MSFXKeyword::Bl => Ok(Self::Bl),
            MSFXKeyword::Br => Ok(Self::Br),
            MSFXKeyword::Tl => Ok(Self::Tl),
            MSFXKeyword::Tr => Ok(Self::Tr),
            MSFXKeyword::C => Ok(Self::C),
            _ => Err(format!("Illegal keyword '{keyword:?}' for export target!"))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportShapeStmt {
    pub target: ExportTarget,
    pub shape: MSFXExpr
}

#[derive(Debug, Clone)]
pub struct ExportAdaptiveStmt {
    pub parts: [MSFXExpr; 9]
}

#[derive(Debug, Clone)]
pub enum MSFXExpr {
    Shape(ShapeExpr),
    Call(FnExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Ident(String),
    Literal(f64),
    Empty,
}

#[derive(Debug, Clone)]
pub struct ShapeExpr {
    pub mode: Box<MSFXExpr>,
    pub block: Vec<MSFXStmt>,
}

#[derive(Debug, Clone)]
pub struct FnExpr {
    pub name: String,
    pub params: HashMap<String, MSFXExpr>
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: MSFXOperator,
    pub inner: Box<MSFXExpr>
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: MSFXOperator,
    pub lhs: Box<MSFXExpr>,
    pub rhs: Box<MSFXExpr>
}
