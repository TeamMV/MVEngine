use crate::color::RgbColor;
use crate::rendering::{InputVertex, Transform};
use crate::ui::geometry::shape::msfx::ast::{BinaryExpr, DeclStmt, ExportAdaptiveStmt, ExportShapeStmt, FnExpr, ForStmt, IfStmt, MSFXAST, MSFXExpr, MSFXStmt, ShapeExpr, UnaryExpr, WhileStmt, Function};
use crate::ui::geometry::shape::msfx::functions::get_function;
use crate::ui::geometry::shape::msfx::lexer::MSFXOperator;
use crate::ui::geometry::shape::msfx::ty::Variable;
use crate::ui::geometry::shape::{Indices, Shape};
use crate::ui::rendering::adaptive::AdaptiveShape;
use hashbrown::HashMap;
use itertools::Itertools;
use std::array;
use std::fmt::format;
use log::trace;
pub use crate::ui::geometry::shape::msfx::ty::{InputVariable, SavedDebugVariable};

pub enum LoopState {
    Normal,
    Continue,
    Break,
}

pub enum Return {
    Shape(Shape),
    Adaptive(AdaptiveShape),
}

pub struct MSFXExecutor {
    pub(crate) variables: HashMap<String, Variable>,
    inputs: HashMap<String, InputVariable>,
    loop_depth: u8, // If you nest more than 255 loops, sincerely, fuck you
    //agreed.
    loop_state: LoopState,
    inside_shape: bool,
    halt: bool,
    the_return: Option<Return>,
    current_vertices: Vec<(f64, f64)>,
    last_ret: Option<Variable>,
    functions: HashMap<String, Function>,
}

impl MSFXExecutor {
    pub fn new() -> MSFXExecutor {
        MSFXExecutor {
            variables: HashMap::new(),
            inputs: HashMap::new(),
            loop_depth: 0,
            loop_state: LoopState::Normal,
            inside_shape: false,
            halt: false,
            the_return: None,
            current_vertices: vec![],
            last_ret: None,
            functions: HashMap::new(),
        }
    }

    pub fn run(
        &mut self,
        ast: &MSFXAST,
        inputs: HashMap<String, InputVariable>,
    ) -> Result<Return, String> {
        self.run_debug(ast, inputs).map(|r| r.0).map_err(|r| r.0)
    }

    pub fn run_debug(
        &mut self,
        ast: &MSFXAST,
        inputs: HashMap<String, InputVariable>,
    ) -> Result<(Return, HashMap<String, SavedDebugVariable>), (String, HashMap<String, SavedDebugVariable>)> {
        self.inputs = inputs;
        self.functions = ast.functions.clone();
        let result = self.run_block(&ast.elements);
        self.loop_state = LoopState::Normal;
        self.loop_depth = 0;
        self.inside_shape = false;
        self.current_vertices = vec![];
        self.functions.clear();
        let vars = self.variables.drain().map(|(n, v)| (unscope(&n), v.into())).collect();
        if let Err(err) = result {
            Err((err, vars))
        } else if let Some(ret) = self.the_return.take() {
            self.halt = false;
            Ok((ret, vars))
        } else {
            Err(("MSFX missing return, you must call export at the end of your code!".to_string(), vars))
        }
    }

    pub fn run_block(&mut self, block: &[MSFXStmt]) -> Result<(), String> {
        for stmt in block {
            if self.halt || self.last_ret.is_some() {
                break;
            }
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn run_function(&mut self, function: &Function, mut params: HashMap<String, Variable>) -> Result<Variable, String> {
        for (name, ty) in &function.params {
            let value = params.remove(name).ok_or(format!("Missing param '{}' from function {}", unscope(name), function.name))?;
            if value.ty() != *ty {
                return Err(format!("Incorrect type for function param '{}'", unscope(name)));
            }
            self.variables.insert(name.clone(), value);
        }

        self.execute_stmt(&function.body)?;

        for local in &function.locals {
            self.variables.remove(local);
        }

        Ok(self.last_ret.take().unwrap_or(Variable::Null))
    }

    pub fn execute_stmt(&mut self, stmt: &MSFXStmt) -> Result<(), String> {
        if self.halt || self.last_ret.is_some() {
            return Ok(());
        }
        if let LoopState::Continue = self.loop_state {
            return Ok(());
        }
        match stmt {
            MSFXStmt::Block(b) => self.run_block(b)?,
            MSFXStmt::Let(decl) => self.execute_decl(decl, true)?,
            MSFXStmt::Assign(assign) => self.execute_decl(assign, false)?,
            MSFXStmt::For(f) => self.execute_for(f)?,
            MSFXStmt::While(w) => self.execute_while(w)?,
            MSFXStmt::If(i) => self.execute_if(i)?,
            MSFXStmt::ExportShape(s) => self.export_shape(s)?,
            MSFXStmt::ExportAdaptive(a) => self.export_adaptive(a)?,
            MSFXStmt::Break => {
                if self.loop_depth > 0 {
                    self.loop_state = LoopState::Break;
                } else {
                    return Err("Cannot use 'break' outside a loop".to_string());
                }
            }
            MSFXStmt::Continue => {
                if self.loop_depth > 0 {
                    self.loop_state = LoopState::Continue;
                } else {
                    return Err("Cannot use 'continue' outside a loop".to_string());
                }
            }
            MSFXStmt::Expr(e) => {
                self.evaluate(e)?;
            }
            MSFXStmt::Input(input) => {
                match self.inputs.get(&input.name).cloned() {
                    None => {
                        if let Some(default) = &input.default {
                            let value = self.evaluate(default)?.as_raw(self)?;
                            self.variables.insert(input.name.clone(), value);
                        } else {
                            return Err(format!("Missing input parameter '{}'", input.name));
                        }
                    }
                    Some(var) => {
                        if var.ty() != input.ty {
                            return Err(format!(
                                "Mismatched input type for '{}', expected {:?} but got {:?}",
                                input.name,
                                input.ty,
                                var.ty()
                            ));
                        }
                        self.variables.insert(input.name.clone(), var.into());
                    }
                }
            }
            MSFXStmt::Return(e) => {
                let mut var = self.evaluate(e)?;
                self.last_ret = Some(var.as_raw(self)?)
            }
            MSFXStmt::Nop => {}
        }
        Ok(())
    }

    pub fn execute_decl(&mut self, stmt: &DeclStmt, new: bool) -> Result<(), String> {
        if !self.variables.contains_key(&stmt.name) && !new {
            return Err(format!("Unknown variable: '{}'", unscope(&stmt.name)));
        }
        let value = self.evaluate(&stmt.expr)?.as_raw(self)?;
        self.variables.insert(stmt.name.clone(), value);
        Ok(())
    }

    pub fn execute_for(&mut self, stmt: &ForStmt) -> Result<(), String> {
        let step = self.evaluate(&stmt.step)?.as_raw(self)?.as_num()?;
        let end = self.evaluate(&stmt.end)?.as_raw(self)?.as_num()?;

        let mut i = self.evaluate(&stmt.start)?.as_raw(self)?.as_num()?;

        fn a(i: f64, end: f64) -> bool {
            i < end
        }
        fn b(i: f64, end: f64) -> bool {
            i > end
        }

        let f = if step < 0.0 { b } else { a };

        self.loop_depth += 1;
        while f(i, end) {
            self.variables
                .insert(stmt.varname.clone(), Variable::Number(i));
            self.execute_stmt(&stmt.block)?;
            i += step;
            if let LoopState::Break = self.loop_state {
                self.loop_state = LoopState::Normal;
                break;
            }
            if self.halt || self.last_ret.is_some() {
                break;
            }
            self.loop_state = LoopState::Normal;
        }
        self.loop_depth - 1;

        Ok(())
    }

    pub fn execute_while(&mut self, stmt: &WhileStmt) -> Result<(), String> {
        self.loop_depth += 1;
        while self.evaluate(&stmt.cond)?.as_raw(self)?.as_bool()? {
            self.execute_stmt(&stmt.block)?;
            if let LoopState::Break = self.loop_state {
                self.loop_state = LoopState::Normal;
                break;
            }
            if self.halt || self.last_ret.is_some() {
                break;
            }
            self.loop_state = LoopState::Normal;
        }
        self.loop_depth - 1;
        Ok(())
    }

    pub fn execute_if(&mut self, stmt: &IfStmt) -> Result<(), String> {
        if self.evaluate(&stmt.cond)?.as_raw(self)?.as_bool()? {
            self.execute_stmt(&stmt.true_block)?;
        } else {
            self.execute_stmt(&stmt.false_block)?;
        }
        Ok(())
    }

    pub fn export_shape(&mut self, stmt: &ExportShapeStmt) -> Result<(), String> {
        let r = self.evaluate(&stmt.shape)?.as_raw(self)?;
        self.halt = true;
        match r {
            Variable::Shape(s) => {
                self.the_return = Some(Return::Shape(s));
            }
            a => {
                return Err(format!(
                    "Illeagal return {} at export! To export a shape, well, you have to export a SHAPE.",
                    a.name()
                ));
            }
        }
        Ok(())
    }

    fn expect_shape(&mut self, exp: &MSFXExpr) -> Result<Shape, String> {
        let r = self.evaluate(exp)?.as_raw(self)?;
        match r {
            Variable::Shape(s) => Ok(s),
            a => Err(format!(
                "Illeagal return {} at export! To export an adaptive shape, well, you have to export some SHAPEs in the form of a nice an even rectangle of SHAPEs (or nulls).",
                a.name()
            )),
        }
    }

    pub fn export_adaptive(&mut self, stmt: &ExportAdaptiveStmt) -> Result<(), String> {
        let mut shapes = vec![];
        for part in &stmt.parts {
            if let MSFXExpr::Empty = part {
                shapes.push(None);
                continue;
            }
            let s = self.expect_shape(part)?;
            shapes.push(Some(s));
        }
        self.halt = true;
        if shapes.len() != 9 {
            return Err("Illegal amount of shapes exported as adaptive! Please export exactly 9 parts and use the # wildcard to skip!".to_string());
        }
        let arr: [Option<Shape>; 9] = array::from_fn(|i| shapes[i].clone());
        self.the_return = Some(Return::Adaptive(AdaptiveShape::from_arr(arr)));

        Ok(())
    }

    pub fn evaluate(&mut self, expr: &MSFXExpr) -> Result<Variable, String> {
        match expr {
            MSFXExpr::Shape(s) => self.evaluate_shape(s),
            MSFXExpr::Call(c) => self.evaluate_call(c),
            MSFXExpr::Unary(u) => self.evaluate_uexpr(u),
            MSFXExpr::Binary(b) => self.evaluate_bexpr(b),
            MSFXExpr::Ty(t) => Ok(Variable::Bool(self.evaluate(&t.expr)?.as_raw(self)?.ty() == t.ty)),
            MSFXExpr::Ident(ident) => Ok(Variable::Saved(ident.clone())),
            MSFXExpr::Literal(n) => Ok(Variable::Number(*n)),
            MSFXExpr::Bool(b) => Ok(Variable::Bool(*b)),
            MSFXExpr::Empty => Ok(Variable::Null),
        }
    }

    pub fn evaluate_shape(&mut self, shape: &ShapeExpr) -> Result<Variable, String> {
        let mode_var = self.evaluate(&shape.mode)?.as_raw(self)?.as_num()?;
        self.current_vertices.clear();
        self.run_block(&shape.block)?;
        let indices = Self::get_indices(mode_var);
        let vertices = self
            .current_vertices
            .iter()
            .map(|(x, y)| InputVertex {
                transform: Transform::new(),
                pos: (*x as f32, *y as f32, 0.0),
                color: RgbColor::white().as_vec4(),
                uv: (0.0, 0.0),
                texture: 0,
                has_texture: 0.0,
            })
            .collect_vec();
        let mut shape = Shape::new(vertices, indices);
        shape.recompute();
        Ok(Variable::Shape(shape))
    }

    fn get_indices(thingy: f64) -> Indices {
        match thingy {
            1.0 => Indices::TriangleStrip,
            0.0 | _ => Indices::Triangles,
        }
    }

    pub fn evaluate_call(&mut self, call: &FnExpr) -> Result<Variable, String> {
        if (call.name == "vertex") {
            //idk never set to true so this is fine ig
            if !self.inside_shape && false {
                return Err(
                    "IllegalStateException: Cannot call the vertex function outside a shape block!"
                        .to_string(),
                );
            }
            if let Some(x) = call.params.get("vertex_x")
                && let Some(y) = call.params.get("vertex_y")
            {
                let x = self.evaluate(x)?.as_raw(self)?.as_num()?;
                let y = self.evaluate(y)?.as_raw(self)?.as_num()?;
                trace!("added vertex");
                self.current_vertices.push((x, y));
            }
            return Ok(Variable::Null);
        }
        if let Some(function) =
            get_function(&call.name) {
            let mut params = HashMap::with_capacity(call.params.len());
            for (key, value) in &call.params {
                if key.as_str() != "_" {
                    params.insert(unscope(key), self.evaluate(value)?.as_raw(self)?.map()?);
                } else {
                    params.insert("_".to_string(), self.evaluate(value)?.as_raw(self)?.map()?);
                }
            }
            function
                .call_ordered(params, &call.order)
                .map(|v| v.unmap())
        } else if let Some(function) = self.functions.get(&call.name).cloned() {
            let mut params = HashMap::with_capacity(call.params.len());
            for (key, value) in &call.params {
                params.insert(key.clone(), self.evaluate(value)?.as_raw(self)?);
            }
            self.run_function(&function, params)
        } else {
            Err(format!("Unknown function '{}'", call.name))
        }
    }

    pub fn evaluate_uexpr(&mut self, uexpr: &UnaryExpr) -> Result<Variable, String> {
        let mut value = self.evaluate(&uexpr.inner)?.as_raw(self)?;

        match uexpr.op {
            MSFXOperator::Sub => value.negate()?,
            MSFXOperator::Not => value.invert()?,
            _ => unreachable!(),
        }

        Ok(value)
    }

    pub fn evaluate_bexpr(&mut self, bexpr: &BinaryExpr) -> Result<Variable, String> {
        let mut lhs = self.evaluate(&bexpr.lhs)?;
        let mut rhs = self.evaluate(&bexpr.rhs)?;

        if let MSFXOperator::Dot = bexpr.op {
            match lhs {
                Variable::Saved(s) => {
                    rhs.enforce_ident()?;
                    Ok(Variable::Access(Box::new(Variable::Saved(s)), Box::new(rhs)))
                }
                // rip Variable::Access (2025-2025)
                // nvm welcome back (2025-unknown)
                Variable::Access(v, f) => {
                    rhs.enforce_ident()?;
                    // ok im honestly not even sure we need to impl this
                    // i honestly cant tell if it will get evaled right away or not
                    // unlucky.
                    // id rather it be right away
                    // wait...
                    // waiting.....................................................................
                    // .....................................................
                    // there
                    // done
                    // erhm buddy, v doesnt exist
                    // actually then I dont think I even need ident+
                    // ok im doing projhect search for Access
                    // ye now we never make one
                    // i can remove
                    // this comment of code will forever remain in our memories
                    // nvm welcome back Variable::Access
                    // ok now i need to figure out what to do here
                    // or more so if this can even be reached
                    // yes it can
                    // but smth else needs changed
                    Ok(Variable::Access(Box::new(Variable::Access(v, f)), Box::new(rhs)))
                }
                mut v if v.has_fields() => {
                    let ident = rhs.as_ident()?;
                    let v_ref = v.as_ref();
                    Ok(v_ref.get_subvalue(&ident)?)
                }
                v => {
                    Err(format!("Cannot access fields on object of type {} because it has none", v.name()))
                }
            }
        } else if let MSFXOperator::Assign = bexpr.op {
            let rhs_raw = rhs.as_raw(self)?;
            match lhs {
                Variable::Saved(s) => {
                    if !self.variables.contains_key(&s) {
                        return Err(format!("Unknown variable: '{}'", unscope(&s)));
                    }
                    self.variables.insert(s, rhs);
                    Ok(Variable::Null)
                }
                Variable::Access(mut v, f) => {
                    //whats the issue
                    // if I have just ident I get to set value but its set on copy of value, not on the one in the hashmap
                    // &mut exists
                    // we need Variable:: for fields
                    // as in:
                    // somehow map our thingy to a ref
                    // vec2.x => Variable::Number
                    // but i dont want to do it like that directly
                    //isnt that exactly the code below
                    // yes but look into as_raw_ref
                    // only exists if self is Saved
                    let mut raw = v.as_raw_ref(self)?;
                    raw.insert_subvalue(&f.as_ident()?, rhs_raw)?;
                    Ok(Variable::Null)
                }
                _ => Err("Cannot assign to not identifier".to_string())
            }
        } else {
            let lhs = lhs.as_raw(self)?;
            let rhs = rhs.as_raw(self)?;
            match bexpr.op {
                MSFXOperator::Add => lhs.add(&rhs),
                MSFXOperator::Sub => lhs.sub(&rhs),
                MSFXOperator::Mul => lhs.mul(&rhs),
                MSFXOperator::Div => lhs.div(&rhs),
                MSFXOperator::Mod => lhs.rem(&rhs),
                MSFXOperator::Pow => lhs.pow(&rhs),
                MSFXOperator::And => lhs.and(&rhs),
                MSFXOperator::Or => lhs.or(&rhs),
                MSFXOperator::Eq => lhs.eq(&rhs),
                MSFXOperator::Neq => lhs.neq(&rhs),
                MSFXOperator::Gt => lhs.gt(&rhs),
                MSFXOperator::Gte => lhs.gte(&rhs),
                MSFXOperator::Lt => lhs.lt(&rhs),
                MSFXOperator::Lte => lhs.lte(&rhs),
                _ => unreachable!(),
            }
        }
    }
}

pub(crate) fn unscope(value: &str) -> String {
    value.split_once("_").unwrap_or(("", value)).1.to_string()
}
