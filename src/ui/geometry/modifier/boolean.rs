use crate::math::vec::Vec2;
use crate::ui::geometry::geom;
use crate::ui::geometry::shape::Shape;
use crate::ui::geometry::shape::msf::Param;
use hashbrown::HashMap;
use log::warn;

trait Dedup<T: PartialEq + Clone> {
    fn clear_duplicates(&mut self);
}

impl<T: PartialEq + Clone> Dedup<T> for Vec<T> {
    fn clear_duplicates(&mut self) {
        let mut already_seen = Vec::with_capacity(self.len());
        self.retain(|item| match already_seen.contains(item) {
            true => false,
            _ => {
                already_seen.push(item.clone());
                true
            }
        })
    }
}

pub(crate) fn compute(
    input: &Shape,
    params: Vec<Param>,
    shapes: &HashMap<String, Shape>,
) -> Result<Shape, String> {
    let mode = params[0].as_str();
    let other_shape_name = params[1].as_str();
    if let Some(other_shape) = shapes.get(other_shape_name) {
        match mode.as_str() {
            "union" => compute_union(input, other_shape),
            "intersect" => compute_intersect(input, other_shape),
            "difference" => compute_difference(input, other_shape),
            _ => Err(format!("Invalid Boolean mode {mode}")),
        }
    } else {
        Err(format!("{} is not defined!", other_shape_name))
    }
}

fn compute_union(_: &Shape, _: &Shape) -> Result<Shape, String> {
    // we do indeed compute the union here
    todo!()
}

pub fn compute_intersect(input: &Shape, clipping: &Shape) -> Result<Shape, String> {
    // we do indeed compute the intersection here
    todo!()
}

fn compute_difference(_: &Shape, _: &Shape) -> Result<Shape, String> {
    // we also compute the difference here
    todo!()
}
