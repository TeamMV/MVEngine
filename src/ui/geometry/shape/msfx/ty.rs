use crate::ui::geometry::shape::msfx::executor::Executor;
use crate::ui::geometry::shape::Shape;
use crate::ui::geometry::SimpleRect;

#[derive(Debug, Clone)]
pub enum MappedVariable {
    Number(f64),
    Bool(bool),
    Shape(Shape),
    Null,
}

impl MappedVariable {
    pub fn as_f64(&self) -> Result<f64, String> {
        match self {
            MappedVariable::Number(n) => Ok(*n),
            MappedVariable::Bool(_) => Err("Invalid argument: Expected number but found bool!".to_string()),
            MappedVariable::Shape(_) => Err("Invalid argument: Expected number but found shape!".to_string()),
            MappedVariable::Null => Err("Invalid argument: Expected number but found null!".to_string()),
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            MappedVariable::Bool(b) => Ok(*b),
            MappedVariable::Number(_) => Err("Invalid argument: Expected bool but found number!".to_string()),
            MappedVariable::Shape(_) => Err("Invalid argument: Expected bool but found shape!".to_string()),
            MappedVariable::Null => Err("Invalid argument: Expected bool but found null!".to_string()),
        }
    }

    pub fn as_shape(&self) -> Result<Shape, String> {
        match self {
            MappedVariable::Shape(s) => Ok(s.clone()),
            MappedVariable::Number(_) => Err("Invalid argument: Expected shape but found number!".to_string()),
            MappedVariable::Bool(_) => Err("Invalid argument: Expected shape but found bool!".to_string()),
            MappedVariable::Null => Err("Invalid argument: Expected shape but found null!".to_string()),
        }
    }

    pub fn as_f64_nullable(&self) -> Result<Option<f64>, String> {
        match self {
            MappedVariable::Number(n) => Ok(Some(*n)),
            MappedVariable::Bool(_) => Err("Invalid argument: Expected number but found bool!".to_string()),
            MappedVariable::Shape(_) => Err("Invalid argument: Expected number but found shape!".to_string()),
            MappedVariable::Null => Ok(None),
        }
    }

    pub fn as_bool_nullable(&self) -> Result<Option<bool>, String> {
        match self {
            MappedVariable::Bool(b) => Ok(Some(*b)),
            MappedVariable::Number(_) => Err("Invalid argument: Expected bool but found number!".to_string()),
            MappedVariable::Shape(_) => Err("Invalid argument: Expected bool but found shape!".to_string()),
            MappedVariable::Null => Ok(None),
        }
    }

    pub fn as_shape_nullable(&self) -> Result<Option<Shape>, String> {
        match self {
            MappedVariable::Shape(s) => Ok(Some(s.clone())),
            MappedVariable::Number(_) => Err("Invalid argument: Expected shape but found number!".to_string()),
            MappedVariable::Bool(_) => Err("Invalid argument: Expected shape but found bool!".to_string()),
            MappedVariable::Null => Ok(None),
        }
    }

    pub fn unmap(&self) -> Variable {
        match self {
            MappedVariable::Number(n) => Variable::Number(*n),
            MappedVariable::Bool(b) => Variable::Bool(*b),
            MappedVariable::Shape(s) => Variable::Shape(s.clone()),
            MappedVariable::Null => Variable::Null
        }
    }

    pub fn apply<F: Fn(&dyn ApplyBrain) -> Result<Variable, String>>(&self, f: F) -> Result<Variable, String> {
        match self {
            MappedVariable::Number(n) => f(n),
            MappedVariable::Bool(b) => f(b),
            MappedVariable::Shape(s) => f(s),
            MappedVariable::Null => unreachable!(),
        }
    }
}

impl From<f64> for MappedVariable {
    fn from(value: f64) -> Self {
        MappedVariable::Number(value)
    }
}

impl From<bool> for MappedVariable {
    fn from(value: bool) -> Self {
        MappedVariable::Bool(value)
    }
}

impl From<Shape> for MappedVariable {
    fn from(value: Shape) -> Self {
        MappedVariable::Shape(value)
    }
}

impl From<()> for MappedVariable {
    fn from(_: ()) -> Self {
        MappedVariable::Null
    }
}

pub trait ApplyBrain {
    fn add(&self, other: &Variable) -> Result<Variable, String>;
    fn sub(&self, other: &Variable) -> Result<Variable, String>;
    fn mul(&self, other: &Variable) -> Result<Variable, String>;
    fn div(&self, other: &Variable) -> Result<Variable, String>;
    fn rem(&self, other: &Variable) -> Result<Variable, String>;
    fn pow(&self, other: &Variable) -> Result<Variable, String>;
    fn and(&self, other: &Variable) -> Result<Variable, String>;
    fn or(&self, other: &Variable) -> Result<Variable, String>;
    fn eq(&self, other: &Variable) -> Result<Variable, String>;
    fn neq(&self, other: &Variable) -> Result<Variable, String>;
    fn gt(&self, other: &Variable) -> Result<Variable, String>;
    fn gte(&self, other: &Variable) -> Result<Variable, String>;
    fn lt(&self, other: &Variable) -> Result<Variable, String>;
    fn lte(&self, other: &Variable) -> Result<Variable, String>;
    fn invert(&self, other: &Variable) -> Result<Variable, String>;
    fn negate(&self, other: &Variable) -> Result<Variable, String>;
}

impl ApplyBrain for f64 {
    
}

impl ApplyBrain for bool {
    
}

impl ApplyBrain for Shape {
    
}

#[derive(Debug, Clone)]
pub enum Variable {
    Null,
    Saved(String),
    Access(Box<Variable>, Box<Variable>),
    Number(f64),
    Bool(bool),
    Shape(Shape),
}

macro_rules! op_fn {
    ($name:ident) => {
        pub fn $name(&self, rhs: &Variable) -> Result<Variable, String> {
            self.throw_nullptr(&format!("apply {}", stringify!($name)))?;
            rhs.throw_nullptr(&format!("apply {}", stringify!($name)))?;
            let eval = self.map()?;
            eval.apply(|lhs| lhs.$name(rhs))
        }
    };
}

impl Variable {
    op_fn!(add);
    op_fn!(sub);
    op_fn!(mul);
    op_fn!(div);
    op_fn!(rem);
    op_fn!(pow);
    op_fn!(and);
    op_fn!(or);
    op_fn!(eq);
    op_fn!(neq);
    op_fn!(lt);
    op_fn!(lte);
    op_fn!(gt);
    op_fn!(gte);

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

    pub fn as_raw_ref(&self, ex: &mut Executor) -> Result<&mut Variable, String> {
        match self {
            Variable::Saved(ident) => ex.variables.get_mut(ident).ok_or(format!("Unknown variable: '{}'", ident)),
            // Variable::Access(_, _) => {
            //     let chain = self.expand_idents();
            //     let mut raw = Variable::Saved(chain[0].clone()).as_raw(ex)?;
            //     raw.get_subvalue_ref(&chain[1..])
            // }
            _ => Err("Dereferencing non single-chain variable".to_string()),
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

    pub fn name(&self) -> &str {
        match self {
            Variable::Null => "null",
            Variable::Saved(_) => "ident",
            Variable::Access(_, _) => "ident+",
            Variable::Number(_) => "number",
            Variable::Bool(_) => "bool",
            Variable::Shape(_) => "shape",
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
            Variable::Shape(s) => Ok(MappedVariable::Shape(s.clone())),
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
}