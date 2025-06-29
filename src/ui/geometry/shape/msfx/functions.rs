use hashbrown::HashMap;
use mvengine_proc_macro::msfx_fn;
use crate::ui::geometry::shape::msfx::executor::Variable;

#[derive(Debug, Clone)]
pub enum MappedVariable {
    Number(f64),
    Bool(bool),
    Null,
}

impl MappedVariable {
    pub fn as_f64(&self) -> Result<f64, String> {
        match self {
            MappedVariable::Number(n) => Ok(*n),
            MappedVariable::Bool(_) => Err("Invalid argument: Expected number but found bool!".to_string()),
            MappedVariable::Null => Err("Invalid argument: Expected number but found null!".to_string()),
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            MappedVariable::Bool(b) => Ok(*b),
            MappedVariable::Number(_) => Err("Invalid argument: Expected bool but found number!".to_string()),
            MappedVariable::Null => Err("Invalid argument: Expected bool but found null!".to_string()),
        }
    }

    pub fn as_f64_nullable(&self) -> Result<Option<f64>, String> {
        match self {
            MappedVariable::Number(n) => Ok(Some(*n)),
            MappedVariable::Bool(_) => Err("Invalid argument: Expected number but found bool!".to_string()),
            MappedVariable::Null => Ok(None),
        }
    }

    pub fn as_bool_nullable(&self) -> Result<Option<bool>, String> {
        match self {
            MappedVariable::Bool(b) => Ok(Some(*b)),
            MappedVariable::Number(_) => Err("Invalid argument: Expected bool but found number!".to_string()),
            MappedVariable::Null => Ok(None),
        }
    }

    pub fn unmap(&self) -> Variable {
        match self {
            MappedVariable::Number(n) => Variable::Number(*n),
            MappedVariable::Bool(b) => Variable::Bool(*b),
            MappedVariable::Null => Variable::Null
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

impl From<()> for MappedVariable {
    fn from(_: ()) -> Self {
        MappedVariable::Null
    }
}

pub trait MSFXFunction {
    fn call(&self, arguments: HashMap<String, MappedVariable>) -> Result<MappedVariable, String>;
}

fn get_named(arguments: &HashMap<String, MappedVariable>, name: String) -> MappedVariable {
    arguments.get(&name).cloned().unwrap_or(MappedVariable::Null)
}

fn get_unnamed(arguments: &HashMap<String, MappedVariable>, name: String) -> MappedVariable {
    let mut value = arguments.get(&name);
    if value.is_none() {
        value = arguments.get("_");
    }
    value.cloned().unwrap_or(MappedVariable::Null)
}

#[msfx_fn]
fn sin(value: f64) -> f64 {
    value.sin()
}

pub fn get_function(name: &str) -> Option<Box<dyn MSFXFunction>> {
    match name {
        "sin" => Some(Box::new(Sin)),
        _ => None,
    }
}