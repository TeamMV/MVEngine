use crate::ui::geometry::shape::msfx::ast::{BinaryExpr, DeclStmt, ExportAdaptiveStmt, ExportShapeStmt, FnExpr, ForStmt, Function, IfStmt, InputStmt, MSFXExpr, MSFXStmt, ShapeExpr, TyExpr, UnaryExpr, WhileStmt, MSFXAST};
use crate::ui::geometry::shape::msfx::functions::get_function;
use hashbrown::hash_map::Entry;
use hashbrown::HashMap;
use mvutils::utils::key;

pub struct MSFXMinifier {
    n: u32,
    f: u32,
    map: HashMap<String, String>,
    f_map: HashMap<String, String>,
}

impl MSFXMinifier {
    pub fn new() -> Self {
        MSFXMinifier {
            n: 0,
            f: 0,
            map: HashMap::new(),
            f_map: HashMap::new(),
        }
    }

    fn find_mapping(&mut self, variable: String) -> String {
        if variable == "_" {
            return variable;
        }
        match self.map.entry(variable) {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(v) => {
                let res = v.insert(key(self.n));
                self.n += 1;
                res.clone()
            }
        }
    }

    fn should_map(&self, function: &str) -> bool {
        !get_function(function).is_some()
    }

    fn find_fn_mapping(&mut self, function: String) -> String {
        if !self.should_map(&function) {
            function
        } else {
            match self.f_map.entry(function) {
                Entry::Occupied(o) => o.get().clone(),
                Entry::Vacant(v) => {
                    let res = v.insert(key(self.f));
                    self.f += 1;
                    res.clone()
                }
            }
        }
    }

    pub fn minify(&mut self, ast: MSFXAST) -> MSFXAST {
        self.n = 0;
        self.f = 0;
        self.map.clear();
        self.f_map.clear();
        let mut functions = HashMap::with_capacity(ast.functions.len());
        for (name, function) in ast.functions {
            functions.insert(self.find_fn_mapping(name), self.map_fn(function));
        }

        MSFXAST {
            elements: ast.elements.into_iter().map(|s| self.map_stmt(s)).collect(),
            functions,
        }
    }

    fn map_fn(&mut self, function: Function) -> Function {
        let mut params = HashMap::with_capacity(function.params.len());
        for (name, ty) in function.params {
            params.insert(self.find_mapping(name), ty);
        }
        Function {
            name: self.find_fn_mapping(function.name.clone()),
            locals: function.locals.into_iter().map(|l| self.find_mapping(l)).collect(),
            params,
            body: self.map_stmt(function.body),
        }
    }

    fn map_stmt(&mut self, stmt: MSFXStmt) -> MSFXStmt {
        match stmt {
            MSFXStmt::Input(i) => MSFXStmt::Input(InputStmt {
                name: self.find_mapping(i.name),
                ty: i.ty,
                default: i.default.map(|e| self.map_expr(e)),
            }),
            MSFXStmt::Block(b) => MSFXStmt::Block(b.into_iter().map(|s| self.map_stmt(s)).collect()),
            MSFXStmt::Let(l) => MSFXStmt::Let(DeclStmt {
                name: self.find_mapping(l.name),
                expr: self.map_expr(l.expr),
            }),
            MSFXStmt::Assign(a) => MSFXStmt::Assign(DeclStmt {
                name: self.find_mapping(a.name),
                expr: self.map_expr(a.expr),
            }),
            MSFXStmt::For(f) => MSFXStmt::For(ForStmt {
                varname: self.find_mapping(f.varname),
                start: self.map_expr(f.start),
                end: self.map_expr(f.end),
                step: self.map_expr(f.step),
                block: Box::new(self.map_stmt(*f.block)),
            }),
            MSFXStmt::While(w) => MSFXStmt::While(WhileStmt {
                cond: self.map_expr(w.cond),
                block: Box::new(self.map_stmt(*w.block)),
            }),
            MSFXStmt::If(i) => MSFXStmt::If(IfStmt {
                cond: self.map_expr(i.cond),
                true_block: Box::new(self.map_stmt(*i.true_block)),
                false_block: Box::new(self.map_stmt(*i.false_block)),
            }),
            MSFXStmt::ExportShape(e) => MSFXStmt::ExportShape(ExportShapeStmt {
                shape: self.map_expr(e.shape),
            }),
            MSFXStmt::ExportAdaptive(e) => MSFXStmt::ExportAdaptive(ExportAdaptiveStmt {
                parts: e.parts.map(|e| self.map_expr(e)),
            }),
            MSFXStmt::Break => MSFXStmt::Break,
            MSFXStmt::Continue => MSFXStmt::Continue,
            MSFXStmt::Return(r) => MSFXStmt::Return(self.map_expr(r)),
            MSFXStmt::Expr(e) => MSFXStmt::Expr(self.map_expr(e)),
            MSFXStmt::Nop => MSFXStmt::Nop,
        }
    }

    fn map_expr(&mut self, expr: MSFXExpr) -> MSFXExpr {
        match expr {
            MSFXExpr::Shape(s) => MSFXExpr::Shape(ShapeExpr {
                mode: Box::new(self.map_expr(*s.mode)),
                block: s.block.into_iter().map(|s| self.map_stmt(s)).collect(),
            }),
            MSFXExpr::Call(c) => {
                let mut params = HashMap::with_capacity(c.params.len());
                let mut order = Vec::with_capacity(c.order.len());
                if self.should_map(&c.name) {
                    for (name, expr) in c.params {
                        params.insert(self.find_mapping(name), self.map_expr(expr));
                    }
                    for name in c.order {
                        order.push(self.find_mapping(name));
                    }
                } else {
                    for (name, expr) in c.params {
                        params.insert(name, self.map_expr(expr));
                    }
                    order = c.order;
                }
                MSFXExpr::Call(FnExpr {
                    name: self.find_fn_mapping(c.name),
                    params,
                    order,
                })
            },
            MSFXExpr::Unary(u) => MSFXExpr::Unary(UnaryExpr {
                op: u.op,
                inner: Box::new(self.map_expr(*u.inner)),
            }),
            MSFXExpr::Binary(b) => MSFXExpr::Binary(BinaryExpr {
                op: b.op,
                lhs: Box::new(self.map_expr(*b.lhs)),
                rhs: Box::new(self.map_expr(*b.rhs)),
            }),
            MSFXExpr::Ty(t) => MSFXExpr::Ty(TyExpr {
                expr: Box::new(self.map_expr(*t.expr)),
                ty: t.ty,
            }),
            MSFXExpr::Ident(i) => MSFXExpr::Ident(self.find_mapping(i)),
            MSFXExpr::Literal(l) => MSFXExpr::Literal(l),
            MSFXExpr::Bool(b) => MSFXExpr::Bool(b),
            MSFXExpr::Empty => MSFXExpr::Empty,
        }
    }
}