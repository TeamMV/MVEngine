use crate::ui::geometry::shape::msfx::executor::MSFXExecutor;
use crate::ui::geometry::shape::Shape;
use crate::utils::{pointee_mut, pointer};
use std::fmt::{Debug, Display, Formatter, Write};
use std::marker::PhantomData;
use std::str::FromStr;
use crate::ui::parse::{parse_2xf64, parse_num};

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub(in crate::ui::geometry::shape::msfx) struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn splat(val: f64) -> Self {
        Self { x: val, y: val }
    }

    pub fn as_mvengine(&self) -> crate::math::vec::Vec2 {
        crate::math::vec::Vec2::new(self.x as f32, self.y as f32)
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;
        f.write_str(&format!("{}, {}", self.x, self.y))?;
        f.write_char(')')
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MSFXType {
    Bool,
    Number,
    Vec2,
}

#[derive(Debug, Clone)]
pub enum SavedDebugVariable {
    Null,
    Number(f64),
    Bool(bool),
    Vec2(crate::math::vec::Vec2),
    Shape(Shape)
}

impl From<Variable> for SavedDebugVariable {
    fn from(value: Variable) -> Self {
        match value {
            Variable::Null => SavedDebugVariable::Null,
            Variable::Number(n) => SavedDebugVariable::Number(n),
            Variable::Bool(b) => SavedDebugVariable::Bool(b),
            Variable::Shape(s) => SavedDebugVariable::Shape(s),
            Variable::Vec2(v) => SavedDebugVariable::Vec2(v.as_mvengine()),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InputVariable {
    Number(f64),
    Bool(bool),
    Vec2(Vec2),
}

impl FromStr for InputVariable {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = parse_num(s) {
            Ok(InputVariable::Number(num))
        } else if let Ok(b) = s.parse::<bool>() {
            Ok(InputVariable::Bool(b))
        } else if let Ok(v) = parse_2xf64(s) {
            Ok(InputVariable::Vec2(Vec2::new(v[0], v[1])))
        } else {
            Err(format!("Cannot parse '{s}' as either a number, a boolean or a vec2!"))
        }
    }
}

impl InputVariable {
    pub(crate) fn ty(&self) -> MSFXType {
        match self {
            InputVariable::Number(_) => MSFXType::Number,
            InputVariable::Bool(_) => MSFXType::Bool,
            InputVariable::Vec2(_) => MSFXType::Vec2,
        }
    }
}

impl From<f64> for InputVariable {
    fn from(value: f64) -> Self {
        InputVariable::Number(value)
    }
}

impl From<bool> for InputVariable {
    fn from(value: bool) -> Self {
        InputVariable::Bool(value)
    }
}

impl From<crate::math::vec::Vec2> for InputVariable {
    fn from(value: crate::math::vec::Vec2) -> Self {
        InputVariable::Vec2(Vec2::new(value.x as f64, value.y as f64))
    }
}

#[derive(Debug, Clone)]
pub enum MappedVariable {
    Number(f64),
    Bool(bool),
    Shape(Shape),
    Vec2(Vec2),
    Null,
}

impl MappedVariable {
    pub fn as_f64(&self) -> Result<f64, String> {
        match self {
            MappedVariable::Number(n) => Ok(*n),
            MappedVariable::Bool(_) => {
                Err("Invalid argument: Expected number but found bool!".to_string())
            }
            MappedVariable::Vec2(_) => {
                Err("Invalid argument: Expected number but found vec2!".to_string())
            }
            MappedVariable::Shape(_) => {
                Err("Invalid argument: Expected number but found shape!".to_string())
            }
            MappedVariable::Null => {
                Err("Invalid argument: Expected number but found null!".to_string())
            }
        }
    }

    pub fn as_vec2(&self) -> Result<Vec2, String> {
        match self {
            MappedVariable::Vec2(n) => Ok(*n),
            MappedVariable::Bool(_) => {
                Err("Invalid argument: Expected vec2 but found bool!".to_string())
            }
            MappedVariable::Number(_) => {
                Err("Invalid argument: Expected vec2 but found number!".to_string())
            }
            MappedVariable::Shape(_) => {
                Err("Invalid argument: Expected vec2 but found shape!".to_string())
            }
            MappedVariable::Null => {
                Err("Invalid argument: Expected vec2 but found null!".to_string())
            }
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            MappedVariable::Bool(b) => Ok(*b),
            MappedVariable::Number(_) => {
                Err("Invalid argument: Expected bool but found number!".to_string())
            }
            MappedVariable::Vec2(_) => {
                Err("Invalid argument: Expected number but found vec2!".to_string())
            }
            MappedVariable::Shape(_) => {
                Err("Invalid argument: Expected bool but found shape!".to_string())
            }
            MappedVariable::Null => {
                Err("Invalid argument: Expected bool but found null!".to_string())
            }
        }
    }

    pub fn as_shape(&self) -> Result<Shape, String> {
        match self {
            MappedVariable::Shape(s) => Ok(s.clone()),
            MappedVariable::Number(_) => {
                Err("Invalid argument: Expected shape but found number!".to_string())
            }
            MappedVariable::Vec2(_) => {
                Err("Invalid argument: Expected number but found vec2!".to_string())
            }
            MappedVariable::Bool(_) => {
                Err("Invalid argument: Expected shape but found bool!".to_string())
            }
            MappedVariable::Null => {
                Err("Invalid argument: Expected shape but found null!".to_string())
            }
        }
    }

    pub fn as_f64_nullable(&self) -> Result<Option<f64>, String> {
        match self {
            MappedVariable::Number(n) => Ok(Some(*n)),
            MappedVariable::Bool(_) => {
                Err("Invalid argument: Expected number but found bool!".to_string())
            }
            MappedVariable::Vec2(_) => {
                Err("Invalid argument: Expected number but found vec2!".to_string())
            }
            MappedVariable::Shape(_) => {
                Err("Invalid argument: Expected number but found shape!".to_string())
            }
            MappedVariable::Null => Ok(None),
        }
    }

    pub fn as_vec2_nullable(&self) -> Result<Option<Vec2>, String> {
        match self {
            MappedVariable::Vec2(n) => Ok(Some(*n)),
            MappedVariable::Bool(_) => {
                Err("Invalid argument: Expected vec2 but found bool!".to_string())
            }
            MappedVariable::Number(_) => {
                Err("Invalid argument: Expected vec2 but found number!".to_string())
            }
            MappedVariable::Shape(_) => {
                Err("Invalid argument: Expected vec2 but found shape!".to_string())
            }
            MappedVariable::Null => Ok(None),
        }
    }

    pub fn as_bool_nullable(&self) -> Result<Option<bool>, String> {
        match self {
            MappedVariable::Bool(b) => Ok(Some(*b)),
            MappedVariable::Number(_) => {
                Err("Invalid argument: Expected bool but found number!".to_string())
            }
            MappedVariable::Vec2(_) => {
                Err("Invalid argument: Expected number but found vec2!".to_string())
            }
            MappedVariable::Shape(_) => {
                Err("Invalid argument: Expected bool but found shape!".to_string())
            }
            MappedVariable::Null => Ok(None),
        }
    }

    pub fn as_shape_nullable(&self) -> Result<Option<Shape>, String> {
        match self {
            MappedVariable::Shape(s) => Ok(Some(s.clone())),
            MappedVariable::Number(_) => {
                Err("Invalid argument: Expected shape but found number!".to_string())
            }
            MappedVariable::Vec2(_) => {
                Err("Invalid argument: Expected number but found vec2!".to_string())
            }
            MappedVariable::Bool(_) => {
                Err("Invalid argument: Expected shape but found bool!".to_string())
            }
            MappedVariable::Null => Ok(None),
        }
    }

    pub fn unmap(&self) -> Variable {
        match self {
            MappedVariable::Number(n) => Variable::Number(*n),
            MappedVariable::Bool(b) => Variable::Bool(*b),
            MappedVariable::Shape(s) => Variable::Shape(s.clone()),
            MappedVariable::Vec2(v) => Variable::Vec2(*v),
            MappedVariable::Null => Variable::Null,
        }
    }

    pub fn apply<F: Fn(&dyn ApplyBrain) -> Result<Variable, String>>(
        &self,
        f: F,
    ) -> Result<Variable, String> {
        match self {
            MappedVariable::Number(n) => f(n),
            MappedVariable::Bool(b) => f(b),
            MappedVariable::Shape(s) => f(s),
            MappedVariable::Vec2(v) => f(v),
            MappedVariable::Null => unreachable!(),
        }
    }
}

impl Display for MappedVariable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MappedVariable::Number(n) => Display::fmt(n, f),
            MappedVariable::Bool(b) => Display::fmt(b, f),
            MappedVariable::Shape(s) => s.fmt(f),
            MappedVariable::Vec2(v) => Display::fmt(v, f),
            MappedVariable::Null => f.write_str("null"),
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

impl From<Vec2> for MappedVariable {
    fn from(value: Vec2) -> Self {
        MappedVariable::Vec2(value)
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
}

impl ApplyBrain for f64 {
    fn add(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Number(self + other.as_num()?))
    }

    fn sub(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Number(self - other.as_num()?))
    }

    fn mul(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Number(self * other.as_num()?))
    }

    fn div(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Number(self / other.as_num()?))
    }

    fn rem(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Number(self % other.as_num()?))
    }

    fn pow(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Number(self.powf(other.as_num()?)))
    }

    fn and(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply and to number!".to_string())
    }

    fn or(&self, other: &Variable) -> Result<Variable, String> {
        Err("Cannot apply or to number!".to_string())
    }

    fn eq(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self == other.as_num()?))
    }

    fn neq(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self != other.as_num()?))
    }

    fn gt(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self > other.as_num()?))
    }

    fn gte(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self >= other.as_num()?))
    }

    fn lt(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self < other.as_num()?))
    }

    fn lte(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self <= other.as_num()?))
    }
}

impl ApplyBrain for bool {
    fn add(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply + to boolean!".to_string())
    }

    fn sub(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply - to boolean!".to_string())
    }

    fn mul(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply * to boolean!".to_string())
    }

    fn div(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply / to boolean!".to_string())
    }

    fn rem(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply % to boolean!".to_string())
    }

    fn pow(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply ^ to boolean (it's pow, not xor)!".to_string())
    }

    fn and(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self && other.as_bool()?))
    }

    fn or(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self || other.as_bool()?))
    }

    fn eq(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self == other.as_bool()?))
    }

    fn neq(&self, other: &Variable) -> Result<Variable, String> {
        Ok(Variable::Bool(*self != other.as_bool()?))
    }

    fn gt(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply > to boolean!".to_string())
    }

    fn gte(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply >= to boolean!".to_string())
    }

    fn lt(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply < to boolean!".to_string())
    }

    fn lte(&self, _: &Variable) -> Result<Variable, String> {
        Err("Cannot apply <= to boolean!".to_string())
    }
}

impl ApplyBrain for Vec2 {
    fn add(&self, other: &Variable) -> Result<Variable, String> {
        match other {
            Variable::Number(n) => Ok(Variable::Vec2(Vec2::new(self.x + *n, self.y + *n))),
            Variable::Vec2(v) => Ok(Variable::Vec2(Vec2::new(self.x + v.x, self.y + v.y))),
            o => Err(format!("Cannot apply `+`, expected number or vec2 but found {}", o.name())),
        }
    }

    fn sub(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn mul(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn div(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn rem(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn pow(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn and(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn or(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn eq(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn neq(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn gt(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn gte(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn lt(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }

    fn lte(&self, other: &Variable) -> Result<Variable, String> {
        todo!()
    }
}

impl ApplyBrain for Shape {
    fn add(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn sub(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn mul(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn div(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn rem(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn pow(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn and(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn or(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn eq(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn neq(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn gt(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn gte(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn lt(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }

    fn lte(&self, other: &Variable) -> Result<Variable, String> {
        Err("Shape cannot be like done shit to".to_string())
    }
}

#[derive(Debug, Clone)]
pub enum Variable {
    Null,
    Saved(String),
    Access(Box<Variable>, Box<Variable>),
    Number(f64),
    Bool(bool),
    Shape(Shape),
    Vec2(Vec2),
}

#[derive(Copy, Clone)]
pub enum MutVar<'a> {
    Null(PhantomData<&'a ()>),
    Number(usize),
    Bool(usize),
    Shape(usize),
    Vec2(usize),
}

impl From<InputVariable> for Variable {
    fn from(value: InputVariable) -> Self {
        match value {
            InputVariable::Number(n) => Variable::Number(n),
            InputVariable::Bool(b) => Variable::Bool(b),
            InputVariable::Vec2(v) => Variable::Vec2(v),
        }
    }
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
        self.enforce_bool_msg("! operator")?;
        let Variable::Bool(b) = self else {
            unreachable!()
        };
        *b = !*b;
        Ok(())
    }

    pub fn negate(&mut self) -> Result<(), String> {
        self.throw_nullptr("apply - unary operator")?;
        self.enforce_number_or_vec_msg("- unary operator")?;
        if let Variable::Number(n) = self {
            *n = -*n;
        } else if let Variable::Vec2(v) = self {
            v.x = -v.x;
            v.y = -v.y;
        } else {
            unreachable!()
        };
        Ok(())
    }

    pub fn as_ref(&mut self) -> MutVar {
        match self {
            Variable::Null => MutVar::Null(PhantomData::default()),
            Variable::Number(n) => MutVar::Number(pointer(n)),
            Variable::Bool(b) => MutVar::Bool(pointer(b)),
            Variable::Shape(s) => MutVar::Shape(pointer(s)),
            Variable::Vec2(v) => MutVar::Vec2(pointer(v)),
            _ => unreachable!(),
        }
    }

    pub fn as_raw_ref<'a>(&'a mut self, ex: &'a mut MSFXExecutor) -> Result<MutVar<'a>, String> {
        match self {
            Variable::Saved(ident) => {
                let var = ex
                    .variables
                    .get_mut(ident)
                    .ok_or(format!("Unknown variable: '{}'", ident))?;
                Ok(var.as_ref())
            },
            // This is now suddenly needed
            Variable::Access(base, f) => {
                let mut base_ref: MutVar<'a> = base.as_raw_ref(ex)?;
                let field = f.as_ident()?;
                let field_ref = base_ref.get_subvalue_ref(&field)?;
                Ok(field_ref)
            }
            _ => Err("Dereferencing non single-chain variable".to_string()),
        }
    }

    pub fn as_raw(&mut self, ex: &mut MSFXExecutor) -> Result<Variable, String> {
        match self {
            Variable::Saved(ident) => ex
                .variables
                .get(ident)
                .cloned()
                .ok_or(format!("Unknown variable: '{}'", ident)),
            Variable::Access(base, f) => {
                let mut base_ref = base.as_raw_ref(ex)?;
                let field = f.as_ident()?;
                Ok(base_ref.get_subvalue(&field)?)
            }
            v => Ok(v.clone()),
        }
    }

    pub fn throw_nullptr(&self, msg: &str) -> Result<(), String> {
        match self {
            Variable::Null => Err(format!(
                "NullPointerException: Cannot {} because value is null!",
                msg
            )),
            _ => Ok(()),
        }
    }

    pub fn enforce_ident(&self) -> Result<(), String> {
        match self {
            Variable::Saved(_) => Ok(()),
            v => Err(format!("Expected ident but found '{}'", v.name())),
        }
    }

    pub fn enforce_number(&self) -> Result<(), String> {
        match self {
            Variable::Number(_) => Ok(()),
            v => Err(format!("Expected number but found '{}'", v.name())),
        }
    }

    pub fn enforce_vec2(&self) -> Result<(), String> {
        match self {
            Variable::Vec2(_) => Ok(()),
            v => Err(format!("Expected number but found '{}'", v.name())),
        }
    }

    pub fn enforce_number_or_vec_msg(&self, msg: &str) -> Result<(), String> {
        match self {
            Variable::Number(_) => Ok(()),
            Variable::Vec2(_) => Ok(()),
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
            Variable::Vec2(_) => "vec2",
        }
    }

    pub fn as_ident(&self) -> Result<String, String> {
        match self {
            Variable::Saved(ident) => Ok(ident.clone()),
            v => Err(format!("Expected ident but found {}", v.name())),
        }
    }

    pub fn map(&self) -> Result<MappedVariable, String> {
        match self {
            Variable::Number(n) => Ok(MappedVariable::Number(*n)),
            Variable::Bool(b) => Ok(MappedVariable::Bool(*b)),
            Variable::Shape(s) => Ok(MappedVariable::Shape(s.clone())),
            Variable::Vec2(v) => Ok(MappedVariable::Vec2(*v)),
            Variable::Null => Ok(MappedVariable::Null),
            _ => Err(format!(
                "{} cannot be used as an input parameter",
                self.name()
            )),
        }
    }

    pub fn as_num(&self) -> Result<f64, String> {
        self.enforce_number()?;
        let Variable::Number(n) = self else {
            unreachable!()
        };
        Ok(*n)
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        self.enforce_bool()?;
        let Variable::Bool(b) = self else {
            unreachable!()
        };
        Ok(*b)
    }

    pub fn as_vec2(&self) -> Result<Vec2, String> {
        self.enforce_vec2()?;
        let Variable::Vec2(v) = self else {
            unreachable!();
        };
        Ok(*v)
    }

    pub fn as_shape(&self) -> Result<Shape, String> {
        self.enforce_vec2()?;
        let Variable::Shape(s) = self else {
            unreachable!();
        };
        Ok(s.clone())
    }

    pub(crate) fn has_fields(&self) -> bool {
        match self {
            Variable::Vec2(_) => true,
            _ => false
        }
    }
}

impl<'a> MutVar<'a> {
    pub(crate) fn assign(&mut self, value: Variable) -> Result<(), String> {
        match self {
            MutVar::Null(_) => Err("Cannot assign value to null".to_string()),
            MutVar::Number(n) => {
                let n = pointee_mut::<'a, f64>(*n);
                *n = value.as_num()?;
                Ok(())
            }
            MutVar::Bool(b) => {
                let b = pointee_mut::<'a, bool>(*b);
                *b = value.as_bool()?;
                Ok(())
            }
            MutVar::Shape(s) => {
                let s = pointee_mut::<'a, Shape>(*s);
                *s = value.as_shape()?;
                Ok(())
            }
            MutVar::Vec2(v) => {
                let v = pointee_mut::<'a, Vec2>(*v);
                *v = value.as_vec2()?;
                Ok(())
            }
        }
    }

    pub(crate) fn insert_subvalue(
        &mut self,
        path: &str,
        value: Variable,
    ) -> Result<(), String> {
        match self {
            MutVar::Vec2(v) => {
                let v = pointee_mut::<'a, Vec2>(*v);
                match path {
                    "x" => {
                        v.x = value.as_num()?;
                    }
                    "y" => {
                        v.y = value.as_num()?;
                    }
                    _ => return Err(format!("Vec2 does not have subfield {}", path))
                }
                Ok(())
            }
            s => Err(format!("Cannot access subfield {} on parameter of type {}", path, s.name())),
        }
    }

    pub(crate) fn get_subvalue(&self, path: &str) -> Result<Variable, String> {
        match self {
            MutVar::Vec2(v) => {
                let v = pointee_mut::<'a, Vec2>(*v);
                match path {
                    "x" => Ok(Variable::Number(v.x)),
                    "y" => Ok(Variable::Number(v.y)),
                    _ => Err(format!("Vec2 does not have subfield {}", path))
                }
            }
            s => Err(format!("Cannot access subfield {} on parameter of type {}", path, s.name())),
        }
    }

    pub(crate) fn get_subvalue_ref<'b>(&'b mut self, path: &str) -> Result<MutVar<'a>, String> {
        match self {
            MutVar::Vec2(v) => {
                let v = pointee_mut::<'a, Vec2>(*v);
                match path {
                    "x" => Ok(MutVar::Number(pointer(&mut v.x))),
                    "y" => Ok(MutVar::Number(pointer(&mut v.y))),
                    _ => Err(format!("Vec2 does not have subfield {}", path))
                }
            }
            s => Err(format!("Cannot access subfield {} on parameter of type {}", path, s.name())),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            MutVar::Null(_) => "null",
            MutVar::Number(_) => "number",
            MutVar::Bool(_) => "bool",
            MutVar::Shape(_) => "shape",
            MutVar::Vec2(_) => "vec2",
        }
    }
}
