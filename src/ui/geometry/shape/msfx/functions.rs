use hashbrown::HashMap;
use mvengine_proc_macro::msfx_fn;
use crate::ui::geometry::shape::msfx::ty::MappedVariable;

pub trait MSFXFunction {
    fn call_ordered(&self, arguments: HashMap<String, MappedVariable>, order: &[String]) -> Result<MappedVariable, String> {
        self.call(arguments)
    }
    fn call(&self, arguments: HashMap<String, MappedVariable>) -> Result<MappedVariable, String>;
}

fn get_named(arguments: &HashMap<String, MappedVariable>, name: &str) -> MappedVariable {
    arguments.get(name).cloned().unwrap_or(MappedVariable::Null)
}

fn get_unnamed(arguments: &HashMap<String, MappedVariable>, name: &str) -> MappedVariable {
    let mut value = arguments.get(name);
    if value.is_none() {
        value = arguments.get("_");
    }
    value.cloned().unwrap_or(MappedVariable::Null)
}

struct Print;

impl MSFXFunction for Print {
    fn call_ordered(&self, arguments: HashMap<String, MappedVariable>, order: &[String]) -> Result<MappedVariable, String> {
        let mut buf = String::new();
        for name in order {
            let value = arguments.get(name).expect("This shouldn't happen... (MSFXParser must have a critical bug)");
            if name.starts_with("_") {
                buf.push_str(&format!("{}, ", value));
            } else {
                buf.push_str(&format!("{}: {}, ", name, value));
            }
        }
        buf.pop();
        buf.pop();
        println!("{buf}");
        Ok(MappedVariable::Null)
    }

    fn call(&self, _: HashMap<String, MappedVariable>) -> Result<MappedVariable, String> {
        unreachable!()
    }
}

#[msfx_fn]
fn sin(value: f64) -> f64 {
    value.sin()
}

pub fn get_function(name: &str) -> Option<Box<dyn MSFXFunction>> {
    match name {
        "print" => Some(Box::new(Print)),
        "sin" => Some(Box::new(Sin)),
        _ => None,
    }
}