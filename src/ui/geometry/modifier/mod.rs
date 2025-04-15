pub mod boolean;

use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::shapes::Param;
use hashbrown::HashMap;
use mvutils::lazy;

lazy! {
    pub static MODIFIER_BOOLEAN: Modifier = Modifier::new("Boolean", vec![ParamType::Str, ParamType::Str], boolean::compute);
}

#[derive(Debug)]
pub enum ParamType {
    Str,
    Struct,
}

pub type ModifierFunction =
    fn(&Shape, Vec<Param>, &HashMap<String, Shape>) -> Result<Shape, String>;

pub struct Modifier {
    name: &'static str,
    function: ModifierFunction,
    types: Vec<ParamType>,
}

impl Modifier {
    pub const fn new(
        name: &'static str,
        types: Vec<ParamType>,
        function: ModifierFunction,
    ) -> Self {
        Self {
            name,
            function,
            types,
        }
    }

    fn illegal_params(&self) -> Result<Shape, String> {
        Err(format!(
            "Illegal combination of parameters, expected {:?}",
            self.types
        ))
    }

    pub fn run(
        &self,
        input: &Shape,
        params: Vec<Param>,
        shapes: &HashMap<String, Shape>,
    ) -> Result<Shape, String> {
        if params.len() != self.types.len() {
            return Err(format!(
                "Illegal amount of parameters given to modifier {}",
                self.name
            ));
        }
        for (idx, param) in params.iter().enumerate() {
            let ty = &self.types[idx];
            match ty {
                ParamType::Str => {
                    if let Param::Str(_) = param {
                    } else {
                        return self.illegal_params();
                    }
                }
                ParamType::Struct => {
                    if let Param::Struct(_) = param {
                    } else {
                        return self.illegal_params();
                    }
                }
            }
        }

        (self.function)(input, params, shapes)
    }
}
