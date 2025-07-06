use crate::ui::geometry::shape::msfx::lexer::MSFXOperator;
use crate::ui::geometry::shape::msfx::ty::MSFXType;
use hashbrown::HashMap;
use mvutils::Savable;
use mvutils::save::{Loader, Savable, Saver};
use mvutils::save::custom::{string8_load, string8_save, varint_load, varint_save};

fn hashmap_save<T: Savable>(saver: &mut impl Saver, value: &HashMap<String, T>) {
    varint_save(saver, &(value.len() as u64));
    for (key, value) in value {
        string8_save(saver, key);
        value.save(saver);
    }
}

fn hashmap_load<T: Savable>(loader: &mut impl Loader) -> Result<HashMap<String, T>, String> {
    let len = varint_load(loader)?;
    let mut map = HashMap::with_capacity(len as usize);
    for _ in 0..len {
        let name = string8_load(loader)?;
        let value = T::load(loader)?;
        map.insert(name, value);
    }
    Ok(map)
}

fn varvec_save<T: Savable>(saver: &mut impl Saver, vec: &Vec<T>) {
    varint_save(saver, &(vec.len() as u64));
    for t in vec {
        t.save(saver);
    }
}

fn varvec_load<T: Savable>(loader: &mut impl Loader) -> Result<Vec<T>, String> {
    let len = varint_load(loader)?;
    let mut vec = Vec::with_capacity(len as usize);
    for _ in 0..len {
        vec.push(T::load(loader)?);
    }
    Ok(vec)
}

#[derive(Debug, Savable)]
pub struct MSFXAST {
    #[custom(save = varvec_save, load = varvec_load)]
    pub elements: Vec<MSFXStmt>,
    #[custom(save = hashmap_save, load = hashmap_load)]
    pub functions: HashMap<String, Function>
}

#[derive(Debug, Clone, Savable)]
pub struct Function {
    #[custom(save = string8_save, load = string8_load)]
    pub name: String,
    #[custom(save = varvec_save, load = varvec_load)]
    pub locals: Vec<String>,
    #[custom(save = hashmap_save, load = hashmap_load)]
    pub params: HashMap<String, MSFXType>,
    pub body: MSFXStmt,
}

#[derive(Debug, Clone, Savable)]
pub enum MSFXStmt {
    Input(InputStmt),
    Block(#[custom(save = varvec_save, load = varvec_load)] Vec<MSFXStmt>),
    Let(DeclStmt),
    Assign(DeclStmt),
    For(ForStmt),
    While(WhileStmt),
    If(IfStmt),
    ExportShape(ExportShapeStmt),
    ExportAdaptive(ExportAdaptiveStmt),
    Break,
    Continue,
    Return(MSFXExpr),
    Expr(MSFXExpr),
    Nop,
}

#[derive(Debug, Clone, Savable)]
pub struct InputStmt {
    #[custom(save = string8_save, load = string8_load)]
    pub name: String,
    pub ty: MSFXType,
    pub default: Option<MSFXExpr>,
}

#[derive(Debug, Clone, Savable)]
pub struct DeclStmt {
    #[custom(save = string8_save, load = string8_load)]
    pub name: String,
    pub expr: MSFXExpr,
}

#[derive(Debug, Clone, Savable)]
pub struct ForStmt {
    #[custom(save = string8_save, load = string8_load)]
    pub varname: String,
    pub start: MSFXExpr,
    pub end: MSFXExpr,
    pub step: MSFXExpr,
    pub block: Box<MSFXStmt>,
}

#[derive(Debug, Clone, Savable)]
pub struct WhileStmt {
    pub cond: MSFXExpr,
    pub block: Box<MSFXStmt>,
}

#[derive(Debug, Clone, Savable)]
pub struct IfStmt {
    pub cond: MSFXExpr,
    pub true_block: Box<MSFXStmt>,
    pub false_block: Box<MSFXStmt>,
}

#[derive(Debug, Clone, Savable)]
pub struct ExportShapeStmt {
    pub shape: MSFXExpr,
}

#[derive(Debug, Clone, Savable)]
pub struct ExportAdaptiveStmt {
    pub parts: [MSFXExpr; 9],
}

#[derive(Debug, Clone, Savable)]
pub enum MSFXExpr {
    Shape(ShapeExpr),
    Call(FnExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Ty(TyExpr),
    Ident(#[custom(save = string8_save, load = string8_load)] String),
    Literal(f64),
    Bool(bool),
    Empty,
}

#[derive(Debug, Clone, Savable)]
pub struct ShapeExpr {
    pub mode: Box<MSFXExpr>,
    #[custom(save = varvec_save, load = varvec_load)]
    pub block: Vec<MSFXStmt>,
}

#[derive(Debug, Clone)]
pub struct FnExpr {
    pub name: String,
    pub params: HashMap<String, MSFXExpr>,
    pub order: Vec<String>,
}

impl Savable for FnExpr {
    fn save(&self, saver: &mut impl Saver) {
        string8_save(saver, &self.name);
        saver.push_u8(self.order.len() as u8);
        for name in &self.order {
            string8_save(saver, name);
            self.params.get(name).unwrap().save(saver);
        }
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let name = string8_load(loader)?;
        let count = u8::load(loader)?;
        let mut order = Vec::with_capacity(count as usize);
        let mut params = HashMap::with_capacity(count as usize);
        for _ in 0..count {
            let p_name = string8_load(loader)?;
            let expr = MSFXExpr::load(loader)?;
            order.push(p_name.clone());
            params.insert(p_name, expr);
        }
        Ok(Self {
            name,
            params,
            order,
        })
    }
}

#[derive(Debug, Clone, Savable)]
pub struct UnaryExpr {
    pub op: MSFXOperator,
    pub inner: Box<MSFXExpr>,
}

#[derive(Debug, Clone, Savable)]
pub struct BinaryExpr {
    pub op: MSFXOperator,
    pub lhs: Box<MSFXExpr>,
    pub rhs: Box<MSFXExpr>,
}

#[derive(Debug, Clone, Savable)]
pub struct TyExpr {
    pub expr: Box<MSFXExpr>,
    pub ty: MSFXType,
}
