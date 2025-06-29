use hashbrown::HashMap;
use mvengine_proc_macro::msfx_fn;
use crate::ui::geometry::shape::msfx::ty::MappedVariable;

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