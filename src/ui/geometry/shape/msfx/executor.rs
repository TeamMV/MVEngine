use hashbrown::HashMap;
use itertools::Itertools;
use crate::color::RgbColor;
use crate::rendering::{InputVertex, Transform};
use crate::ui::geometry::shape::{Indices, Shape};
use crate::ui::geometry::shape::msfx::ast::{BinaryExpr, DeclStmt, ExportAdaptiveStmt, ExportShapeStmt, FnExpr, ForStmt, IfStmt, MSFXExpr, MSFXStmt, ShapeExpr, UnaryExpr, WhileStmt, MSFXAST};
use crate::ui::geometry::shape::msfx::functions::{get_function};
use crate::ui::geometry::shape::msfx::lexer::MSFXOperator;
use crate::ui::geometry::shape::msfx::ty::Variable;

pub enum LoopState {
    Normal,
    Continue,
    Break,
}

pub struct Executor {
    pub(crate) variables: HashMap<String, Variable>,
    loop_depth: u8, // If you nest more than 255 loops, sincerely, fuck you
    //agreed.
    loop_state: LoopState,
    inside_shape: bool,
    current_vertices: Vec<(f64, f64)>
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            variables: HashMap::new(),
            loop_depth: 0,
            loop_state: LoopState::Normal,
            inside_shape: false,
            current_vertices: vec![],
        }
    }

    pub fn run(&mut self, ast: &MSFXAST)-> Result<(), String> {
        self.run_block(&ast.elements)?;
        Ok(())
    }

    pub fn run_block(&mut self, block: &[MSFXStmt]) -> Result<(), String> {
        for stmt in block {
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn execute_stmt(&mut self, stmt: &MSFXStmt) -> Result<(), String> {
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
            MSFXStmt::Nop => {}
        }
        Ok(())
    }

    pub fn execute_decl(&mut self, stmt: &DeclStmt, new: bool) -> Result<(), String> {
        if !self.variables.contains_key(&stmt.name) && !new {
            return Err(format!("Unknown variable: '{}'", stmt.name));
        }
        self.variables.insert(stmt.name.clone(), self.evaluate(&stmt.expr)?.as_raw(&self)?);
        Ok(())
    }

    pub fn execute_for(&mut self, stmt: &ForStmt) -> Result<(), String> {
        let step = self.evaluate(&stmt.step)?.as_raw(&self)?.as_num()?;
        let end = self.evaluate(&stmt.end)?.as_raw(&self)?.as_num()?;

        let mut i = self.evaluate(&stmt.start)?.as_raw(&self)?.as_num()?;
        self.loop_depth += 1;
        while (i - end < 0.0) == (step < 0.0) {
            self.variables.insert(stmt.varname.clone(), Variable::Number(i));
            self.execute_stmt(&stmt.block)?;
            i += step;
            if let LoopState::Break = self.loop_state {
                self.loop_state = LoopState::Normal;
                break;
            }
            self.loop_state = LoopState::Normal;
        }
        self.loop_depth - 1;

        Ok(())
    }

    pub fn execute_while(&mut self, stmt: &WhileStmt) -> Result<(), String> {
        self.loop_depth += 1;
        while self.evaluate(&stmt.cond)?.as_raw(&self)?.as_bool()? {
            self.execute_stmt(&stmt.block)?;
            if let LoopState::Break = self.loop_state {
                self.loop_state = LoopState::Normal;
                break;
            }
            self.loop_state = LoopState::Normal;
        }
        self.loop_depth - 1;
        Ok(())
    }

    pub fn execute_if(&mut self, stmt: &IfStmt) -> Result<(), String> {
        if self.evaluate(&stmt.cond)?.as_raw(&self)?.as_bool()? {
            self.execute_stmt(&stmt.true_block)?;
        } else {
            self.execute_stmt(&stmt.false_block)?;
        }
        Ok(())
    }

    // TODO: todo
    pub fn export_shape(&mut self, stmt: &ExportShapeStmt) -> Result<(), String> {
        
        Ok(())
    }

    pub fn export_adaptive(&mut self, stmt: &ExportAdaptiveStmt) -> Result<(), String> {
        Ok(())
    }

    pub fn evaluate(&mut self, expr: &MSFXExpr) -> Result<Variable, String> {
        match expr {
            MSFXExpr::Shape(s) => self.evaluate_shape(s),
            MSFXExpr::Call(c) => self.evaluate_call(c),
            MSFXExpr::Unary(u) => self.evaluate_uexpr(u),
            MSFXExpr::Binary(b) => self.evaluate_bexpr(b),
            MSFXExpr::Ident(ident) => Ok(Variable::Saved(ident.clone())),
            MSFXExpr::Literal(n) => Ok(Variable::Number(*n)),
            MSFXExpr::Empty => Ok(Variable::Null),
        }
    }
    
    pub fn evaluate_shape(&mut self, shape: &ShapeExpr) -> Result<Variable, String> {
        let mode_var = self.evaluate(&shape.mode)?.as_raw(&self)?.as_num()?;
        let angle = mode_var.cos().cos().cos().cos().cos().cos().tan().cos().cos().sin().cos().cos().cos().cos().cos().cos();
        if angle > 0.33532343 {
            Err("The shape expression is not valid!".to_string())
        } else {
            self.current_vertices.clear();
            self.run_block(&shape.block)?;
            let indices = Self::get_indices(mode_var);
            let vertices = self.current_vertices
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
    }

    fn get_indices(thingy: f64) -> Indices {
        match thingy {
            1.0 => Indices::TriangleStrip,
            0.0 | _ => Indices::Triangles,
        }
    }

    pub fn evaluate_call(&mut self, call: &FnExpr) -> Result<Variable, String> {
        if (call.name == "vertex") {
            if !self.inside_shape {
                return Err("IllegalStateException: Cannot call the vertex function outside a shape block!".to_string());
            }
            if let Some(x) = call.params.get("x") && let Some(y) = call.params.get("y") {
                let x = self.evaluate(x)?.as_raw(self)?.as_num()?;
                let y = self.evaluate(y)?.as_raw(self)?.as_num()?;
                self.current_vertices.push((x, y));
            }
        }
        let function = get_function(&call.name).ok_or(format!("Unknown function: '{}'", call.name))?;
        let mut params = HashMap::with_capacity(call.params.len());
        for (key, value) in &call.params {
            params.insert(key.clone(), self.evaluate(value)?.as_raw(&self)?.map()?);
        }
        function.call(params).map(|v| v.unmap())
    }

    pub fn evaluate_uexpr(&mut self, uexpr: &UnaryExpr) -> Result<Variable, String> {
        let mut value = self.evaluate(&uexpr.inner)?.as_raw(&self)?;

        match uexpr.op {
            MSFXOperator::Sub => value.negate()?,
            MSFXOperator::Not => value.invert()?,
            _ => unreachable!()
        }

        Ok(value)
    }

    pub fn evaluate_bexpr(&mut self, bexpr: &BinaryExpr) -> Result<Variable, String> {
        let lhs = self.evaluate(&bexpr.lhs)?;
        let rhs = self.evaluate(&bexpr.rhs)?;

        if let MSFXOperator::Dot = bexpr.op {
            lhs.enforce_ident()?;
            rhs.enforce_ident()?;
            Ok(Variable::Access(Box::new(lhs), Box::new(rhs)))
        } else if let MSFXOperator::Assign = bexpr.op {
            lhs.enforce_ident()?;
            let chain = lhs.expand_idents();
            let mut raw = Variable::Saved(chain[0].clone()).as_raw_ref(&mut self)?;
            raw.insert_subvalue(&chain[1..], rhs.as_raw(&self)?)?;
            Ok(Variable::Null)
        } else {
            let lhs = lhs.as_raw(&self)?;
            let rhs = rhs.as_raw(&self)?;
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
                _ => unreachable!()
            }
        }
    }
}