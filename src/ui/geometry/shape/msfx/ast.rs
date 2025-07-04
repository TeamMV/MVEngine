use crate::ui::geometry::shape::msfx::lexer::MSFXOperator;
use crate::ui::geometry::shape::msfx::ty::MSFXType;
use hashbrown::HashMap;

#[derive(Debug)]
pub struct MSFXAST {
    pub elements: Vec<MSFXStmt>,
}

// TODO: tell max he should implement functions or else...
#[derive(Debug, Clone)]
pub enum MSFXStmt {
    Input(InputStmt),
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
pub struct InputStmt {
    pub name: String,
    pub ty: MSFXType,
    pub default: Option<MSFXExpr>,
}

#[derive(Debug, Clone)]
pub struct DeclStmt {
    pub name: String,
    pub expr: MSFXExpr,
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

#[derive(Debug, Clone)]
pub struct ExportShapeStmt {
    pub shape: MSFXExpr,
}

#[derive(Debug, Clone)]
pub struct ExportAdaptiveStmt {
    pub parts: [MSFXExpr; 9],
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
    pub params: HashMap<String, MSFXExpr>,
    pub order: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: MSFXOperator,
    pub inner: Box<MSFXExpr>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: MSFXOperator,
    pub lhs: Box<MSFXExpr>,
    pub rhs: Box<MSFXExpr>,
}
