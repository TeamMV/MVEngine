use hashbrown::HashMap;
use mvutils::utils::TetrahedronOp;
use crate::ui::geometry::shape::msfx::ast::{BinaryExpr, DeclStmt, ExportAdaptiveStmt, ExportShapeStmt, FnExpr, ForStmt, IfStmt, MSFXExpr, MSFXStmt, ShapeExpr, UnaryExpr, WhileStmt, MSFXAST};
use crate::ui::geometry::shape::msfx::functions::{get_function, MappedVariable};
use crate::ui::geometry::shape::msfx::lexer::MSFXOperator;

#[derive(Debug, Clone)]
pub enum Variable {
    Null,
    Saved(String),
    Access(Box<Variable>, Box<Variable>),
    Number(f64),
    Bool(bool),
    Shape
}

// Operations impl (I wanted to separate them from the other shit)
impl Variable {
    pub(crate) fn add(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("apply +")?;
        rhs.throw_nullptr("apply +")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Number(lhs + rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn sub(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("apply -")?;
        rhs.throw_nullptr("apply -")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Number(lhs - rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn mul(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("apply *")?;
        rhs.throw_nullptr("apply *")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Number(lhs * rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn div(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("apply /")?;
        rhs.throw_nullptr("apply /")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Number(lhs / rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn rem(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("apply %")?;
        rhs.throw_nullptr("apply %")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Number(lhs % rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn pow(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("apply ^")?;
        rhs.throw_nullptr("apply ^")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Number(lhs.powf(*rhs)))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn and(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("evaluate with and")?;
        rhs.throw_nullptr("evaluate with and")?;
        self.enforce_bools(rhs)?;
        if let Variable::Bool(lhs) = self && let Variable::Bool(rhs) = rhs {
            Ok(Variable::Bool(*lhs && *rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn or(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("evaluate with or")?;
        rhs.throw_nullptr("evaluate with or")?;
        self.enforce_bools(rhs)?;
        if let Variable::Bool(lhs) = self && let Variable::Bool(rhs) = rhs {
            Ok(Variable::Bool(*lhs || *rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn eq(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("compare with ==")?;
        rhs.throw_nullptr("compare with ==")?;
        self.enforce_cmp(rhs)?;
        if let Variable::Bool(lhs) = self && let Variable::Bool(rhs) = rhs {
            Ok(Variable::Bool(*lhs == *rhs))
        } else if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Bool(*lhs == *rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn neq(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("compare with !=")?;
        rhs.throw_nullptr("compare with !=")?;
        self.enforce_cmp(rhs)?;
        if let Variable::Bool(lhs) = self && let Variable::Bool(rhs) = rhs {
            Ok(Variable::Bool(*lhs != *rhs))
        } else if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Bool(*lhs != *rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn gt(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("compare with >")?;
        rhs.throw_nullptr("compare with >")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Bool(*lhs > *rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn gte(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("compare with >=")?;
        rhs.throw_nullptr("compare with >=")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Bool(*lhs >= *rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn lt(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("compare with <")?;
        rhs.throw_nullptr("compare with <")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Bool(*lhs < *rhs))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn lte(&self, rhs: &Variable) -> Result<Variable, String> {
        self.throw_nullptr("compare with <=")?;
        rhs.throw_nullptr("compare with <=")?;
        self.enforce_numbers(rhs)?;
        if let Variable::Number(lhs) = self && let Variable::Number(rhs) = rhs {
            Ok(Variable::Bool(*lhs <= *rhs))
        } else {
            unreachable!()
        }
    }

    pub fn invert(&mut self) -> Result<(), String> {
        self.throw_nullptr("apply ! operator")?;
        self.enforce_number_msg("! operator")?;
        let Variable::Bool(b) = self else { unreachable!() };
        *b = !*b;
        Ok(())
    }

    pub fn negate(&mut self) -> Result<(), String> {
        self.throw_nullptr("apply - operator")?;
        self.enforce_number_msg("- operator")?;
        let Variable::Number(n) = self else { unreachable!() };
        *n = -*n;
        Ok(())
    }
}

impl Variable {
    pub fn as_raw_ref(&self, ex: &mut Executor) -> Result<&mut Variable, String> {
        match self {
            Variable::Saved(ident) => ex.variables.get_mut(ident).ok_or(format!("Unknown variable: '{}'", ident)),
            Variable::Access(_, _) => {
                let chain = self.expand_idents();
                let mut raw = Variable::Saved(chain[0].clone()).as_raw(ex)?;
                raw.get_subvalue_ref(&chain[1..])
            }
            _ => Err("Dereferencing non variable".to_string()),
        }
    }

    pub fn as_raw(&self, ex: &Executor) -> Result<Variable, String> {
        match self {
            Variable::Saved(ident) => ex.variables.get(ident).cloned().ok_or(format!("Unknown variable: '{}'", ident)),
            Variable::Access(_, _) => {
                let chain = self.expand_idents();
                let raw = Variable::Saved(chain[0].clone()).as_raw(ex)?;
                raw.get_subvalue(&chain[1..])
            }
            v => Ok(v.clone()),
        }
    }

    pub fn throw_nullptr(&self, msg: &str) -> Result<(), String> {
        match self {
            Variable::Null => Err(format!("NullPointerException: Cannot {} because value is null!", msg)),
            _ => Ok(())
        }
    }

    pub fn enforce_ident(&self) -> Result<(), String> {
        match self {
            Variable::Saved(_) => Ok(()),
            Variable::Access(_, _) => Ok(()),
            v => Err(format!("Expected ident but found '{}'", v.name())),
        }
    }

    pub fn enforce_number(&self) -> Result<(), String> {
        match self {
            Variable::Number(_) => Ok(()),
            v => Err(format!("Expected number but found '{}'", v.name())),
        }
    }

    pub fn enforce_number_msg(&self, msg: &str) -> Result<(), String> {
        match self {
            Variable::Number(_) => Ok(()),
            v => Err(format!("{}: Expected number but found '{}'", msg, v.name())),
        }
    }

    pub fn enforce_bool(&self) -> Result<(), String> {
        match self {
            Variable::Bool(_) => Ok(()),
            v => Err(format!("Expected bool but found '{}'", v.name())),
        }
    }

    pub fn enforce_bool_msg(&self, msg: &str) -> Result<(), String> {
        match self {
            Variable::Bool(_) => Ok(()),
            v => Err(format!("{}: Expected bool but found '{}'", msg, v.name())),
        }
    }

    pub fn enforce_cmp(&self, rhs: &Variable) -> Result<(), String> {
        match (self, rhs) {
            (Variable::Number(_), Variable::Number(_)) => Ok(()),
            (Variable::Bool(_), Variable::Bool(_)) => Ok(()),
            (lhs, rhs) => Err(format!("Cannot apply comparison between {} and {}", lhs.name(), rhs.name())),
        }
    }

    pub fn enforce_numbers(&self, rhs: &Variable) -> Result<(), String> {
        match (self, rhs) {
            (Variable::Number(_), Variable::Number(_)) => Ok(()),
            (lhs, rhs) => Err(format!("Cannot apply numerical operation to {} and {}", lhs.name(), rhs.name())),
        }
    }

    pub fn enforce_bools(&self, rhs: &Variable) -> Result<(), String> {
        match (self, rhs) {
            (Variable::Bool(_), Variable::Bool(_)) => Ok(()),
            (lhs, rhs) => Err(format!("Cannot apply boolean operation to {} and {}", lhs.name(), rhs.name())),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Variable::Null => "null",
            Variable::Saved(_) => "ident",
            Variable::Access(_, _) => "ident+",
            Variable::Number(_) => "number",
            Variable::Bool(_) => "bool",
            Variable::Shape => "shape",
        }
    }

    pub fn expand_idents(&self) ->  Vec<String> {
        match self {
            Variable::Saved(ident) => vec![ident.clone()],
            Variable::Access(lhs, rhs) => {
                let mut lhs = lhs.expand_idents();
                lhs.append(&mut rhs.expand_idents());
                lhs
            }
            _ => unreachable!(),
        }
    }

    pub fn map(&self) -> Result<MappedVariable, String> {
        match self {
            Variable::Number(n) => Ok(MappedVariable::Number(*n)),
            Variable::Bool(b) => Ok(MappedVariable::Bool(*b)),
            Variable::Null => Ok(MappedVariable::Null),
            _ => Err(format!("{} cannot be used as an input parameter", self.name()))
        }
    }

    pub fn as_num(&self) -> Result<f64, String> {
        self.enforce_number()?;
        let Variable::Number(n) = self else { unreachable!() };
        Ok(*n)
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        self.enforce_bool()?;
        let Variable::Bool(b) = self else { unreachable!() };
        Ok(*b)
    }

    pub(crate) fn insert_subvalue(&mut self, path: &[String], value: Variable) -> Result<(), String> {
        todo!()
    }

    pub(crate) fn get_subvalue(&self, path: &[String]) -> Result<Variable, String> {
        todo!()
    }

    pub(crate) fn get_subvalue_ref(&mut self, path: &[String]) -> Result<&mut Variable, String> {
        todo!()
    }
}

pub enum LoopState {
    Normal,
    Continue,
    Break,
}

pub struct Executor {
    variables: HashMap<String, Variable>,
    loop_depth: u8, // If you nest more than 255 loops, sincerely, fuck you
    loop_state: LoopState,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            variables: HashMap::new(),
            loop_depth: 0,
            loop_state: LoopState::Normal,
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
        if self.variables.contains_key(&stmt.name) == new {
            return Err(format!("{} '{}'", new.yn("Cannot redefine variable", "Unknown variable:"), stmt.name));
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

    // TODO: todo (todo)
    pub fn evaluate_shape(&mut self, shape: &ShapeExpr) -> Result<Variable, String> {
        Ok(Variable::Shape)
    }

    pub fn evaluate_call(&mut self, call: &FnExpr) -> Result<Variable, String> {
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
                MSFXOperator::Add => lhs.add(&rhs)?,
                MSFXOperator::Sub => lhs.sub(&rhs)?,
                MSFXOperator::Mul => lhs.mul(&rhs)?,
                MSFXOperator::Div => lhs.div(&rhs)?,
                MSFXOperator::Mod => lhs.rem(&rhs)?,
                MSFXOperator::Pow => lhs.pow(&rhs)?,
                MSFXOperator::And => lhs.and(&rhs)?,
                MSFXOperator::Or => lhs.or(&rhs)?,
                MSFXOperator::Eq => lhs.eq(&rhs)?,
                MSFXOperator::Neq => lhs.neq(&rhs)?,
                MSFXOperator::Gt => lhs.gt(&rhs)?,
                MSFXOperator::Gte => lhs.gte(&rhs)?,
                MSFXOperator::Lt => lhs.lt(&rhs)?,
                MSFXOperator::Lte => lhs.lte(&rhs)?,
                _ => unreachable!()
            }
            todo!()
        }
    }
}