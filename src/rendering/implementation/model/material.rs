use mvengine_proc_macro::chash;
use crate::utils::hashable::{Float, Vec4};

#[chash]
#[derive(Clone, Eq, PartialEq)]
pub struct Material {
    color: Vec4,
    refraction_index: Float
}