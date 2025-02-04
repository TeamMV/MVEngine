pub mod boolean;

use hashbrown::HashMap;
use mvutils::lazy;
use crate::ui::rendering::ctx::DrawShape;
use crate::ui::rendering::shapes::Param;

lazy! {
    pub static MODIFIER_BOOLEAN: Modifier = Modifier::new("Boolean", vec![ParamType::Str, ParamType::Str], boolean::compute);
}

#[derive(Debug)]
pub enum ParamType {
    Str,
    Struct
}

pub type ModifierFunction = fn(&DrawShape, Vec<Param>, &HashMap<String, DrawShape>) -> Result<DrawShape, String>;

pub struct Modifier {
    name: &'static str,
    function: ModifierFunction,
    types: Vec<ParamType>
}

impl Modifier {
    pub const fn new(name: &'static str, types: Vec<ParamType>, function: ModifierFunction) -> Self {
        Self {
            name,
            function,
            types,
        }
    }

    fn illegal_params(&self) -> Result<DrawShape, String> {
        Err(format!("Illegal combination of parameters, expected {:?}", self.types))
    }

    pub fn run(&self, input: &DrawShape, params: Vec<Param>, shapes: &HashMap<String, DrawShape>) -> Result<DrawShape, String> {
        if params.len() != self.types.len() { return Err(format!("Illegal amount of parameters given to modifier {}", self.name)); }
        for (idx, param) in params.iter().enumerate() {
            let ty = &self.types[idx];
            match ty {
                ParamType::Str => { if let Param::Str(_) = param {} else { return self.illegal_params() } }
                ParamType::Struct => { if let Param::Struct(_) = param {} else { return self.illegal_params() } }
            }
        }

        (self.function)(input, params, shapes)
    }
}