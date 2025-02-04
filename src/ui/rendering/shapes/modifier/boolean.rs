use std::cmp::Ordering;
use hashbrown::{HashMap, HashSet};
use itertools::Itertools;
use log::warn;
use crate::math::vec::Vec2;
use crate::ui::rendering::ctx;
use crate::ui::rendering::ctx::DrawShape;
use crate::ui::rendering::shapes::{geometry, Param};

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

pub(crate) fn compute(input: &DrawShape, params: Vec<Param>, shapes: &HashMap<String, DrawShape>) -> Result<DrawShape, String> {
    let mode = params[0].as_str();
    let other_shape_name = params[1].as_str();
    if let Some(other_shape) = shapes.get(other_shape_name) {
        match mode.as_str() {
            "union" => compute_union(input, other_shape),
            "intersect" => compute_intersect(input, other_shape),
            "difference" => compute_difference(input, other_shape),
            _ => Err(format!("Invalid Boolean mode {mode}"))
        }
    } else {
        Err(format!("{} is not defined!", other_shape_name))
    }
}

fn compute_union(input: &DrawShape, other: &DrawShape) -> Result<DrawShape, String> {
    todo!()
}

pub fn compute_intersect(input: &DrawShape, clipping: &DrawShape) -> Result<DrawShape, String> {
    let mut end_shape = DrawShape { triangles: vec![], extent: (0, 0) };

    for input_triangle in &input.triangles {
        let input_vertices = input_triangle.vec2s();
        for clipping_triangle in &clipping.triangles {
            let clipping_vertices = clipping_triangle.vec2s();
            if let Some(mut intersection_points) = geometry::get_triangle_intersections(&input_vertices, &clipping_vertices) {
                let mut res_vertices = Vec::with_capacity(intersection_points.len() + 6);
                res_vertices.extend_from_slice(&input_vertices);
                res_vertices.extend_from_slice(&clipping_vertices);

                res_vertices.retain(|vertex| {
                    geometry::is_point_in_triangle(&input_vertices, *vertex) &&
                    geometry::is_point_in_triangle(&clipping_vertices, *vertex)
                });

                res_vertices.clear_duplicates();

                res_vertices.extend(intersection_points);
                if res_vertices.len() == 4 {
                    let centroid = Vec2::new(
                        res_vertices.iter().map(|v| v.x).sum::<f32>() / res_vertices.len() as f32,
                        res_vertices.iter().map(|v| v.y).sum::<f32>() / res_vertices.len() as f32,
                    );

                    res_vertices.sort_by(|a, b| {
                        let angle_a = (a.y - centroid.y).atan2(a.x - centroid.x);
                        let angle_b = (b.y - centroid.y).atan2(b.x - centroid.x);
                        angle_a.partial_cmp(&angle_b).unwrap()
                    });

                    for i in 1..3 {
                        let tri = ctx::triangle()
                            .point((res_vertices[0].x as i32, res_vertices[0].y as i32), None)
                            .point((res_vertices[i].x as i32, res_vertices[i].y as i32), None)
                            .point((res_vertices[i + 1].x as i32, res_vertices[i + 1].y as i32), None)
                            .create();
                        end_shape.combine(&tri);
                    }
                } else if res_vertices.len() == 3 {
                    let tri = ctx::triangle()
                        .point((res_vertices[0].x as i32, res_vertices[0].y as i32), None)
                        .point((res_vertices[1].x as i32, res_vertices[1].y as i32), None)
                        .point((res_vertices[2].x as i32, res_vertices[2].y as i32), None)
                        .create();
                    end_shape.combine(&tri);
                } else if res_vertices.len() == 5 {
                    let centroid = Vec2::new(
                        res_vertices.iter().map(|v| v.x).sum::<f32>() / res_vertices.len() as f32,
                        res_vertices.iter().map(|v| v.y).sum::<f32>() / res_vertices.len() as f32,
                    );

                    res_vertices.sort_by(|a, b| {
                        let angle_a = (a.y - centroid.y).atan2(a.x - centroid.x);
                        let angle_b = (b.y - centroid.y).atan2(b.x - centroid.x);
                        angle_a.partial_cmp(&angle_b).unwrap()
                    });

                    for i in 1..4 {
                        let tri = ctx::triangle()
                            .point((res_vertices[0].x as i32, res_vertices[0].y as i32), None)
                            .point((res_vertices[i].x as i32, res_vertices[i].y as i32), None)
                            .point((res_vertices[i + 1].x as i32, res_vertices[i + 1].y as i32), None)
                            .create();
                        end_shape.combine(&tri);
                    }
                } else {
                    warn!("Illegal amount of vertices ({}) left for triangulation!", res_vertices.len());
                }
            } else {
                if geometry::is_point_in_triangle(&clipping_vertices, input_vertices[0]) &&
                    geometry::is_point_in_triangle(&clipping_vertices, input_vertices[1]) &&
                    geometry::is_point_in_triangle(&clipping_vertices, input_vertices[2]) {

                    let tri = ctx::triangle()
                        .point((input_vertices[0].x as i32, input_vertices[0].y as i32), None)
                        .point((input_vertices[1].x as i32, input_vertices[1].y as i32), None)
                        .point((input_vertices[2].x as i32, input_vertices[2].y as i32), None)
                        .create();
                    end_shape.combine(&tri);
                } else if geometry::is_point_in_triangle(&input_vertices, clipping_vertices[0]) &&
                    geometry::is_point_in_triangle(&input_vertices, clipping_vertices[1]) &&
                    geometry::is_point_in_triangle(&input_vertices, clipping_vertices[2]) {

                    println!("still addded");

                    let tri = ctx::triangle()
                        .point((clipping_vertices[0].x as i32, clipping_vertices[0].y as i32), None)
                        .point((clipping_vertices[1].x as i32, clipping_vertices[1].y as i32), None)
                        .point((clipping_vertices[2].x as i32, clipping_vertices[2].y as i32), None)
                        .create();
                    end_shape.combine(&tri);
                }
            }
        }
    }

    Ok(end_shape)
}

fn compute_difference(input: &DrawShape, other: &DrawShape) -> Result<DrawShape, String> {
    todo!()
}