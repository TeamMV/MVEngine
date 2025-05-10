use crate::color::RgbColor;
use crate::rendering::{Transform, Triangle};
use crate::ui::geometry::geom;
use crate::ui::geometry::morph::Morph;
use crate::ui::geometry::polygon::Polygon;
use crate::ui::rendering::ctx::TextureCtx;
use ahash::{HashSet, HashSetExt};
use itertools::Itertools;
use std::fmt::{Debug, Formatter, Write};
use mvutils::Savable;

#[derive(Clone, Savable)]
pub struct Shape {
    pub triangles: Vec<Triangle>,
    pub extent: (i32, i32),
    outline: ShapeOutline,
}

impl Debug for Shape {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{\n")?;
        for triangle in &self.triangles {
            triangle.vec2s().fmt(f)?;
            f.write_char('\n')?;
        }
        f.write_char('}')
    }
}

impl Shape {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self {
            triangles,
            extent: (0, 0),
            outline: ShapeOutline { points: vec![] },
        }
    }

    pub fn new_with_extent(triangles: Vec<Triangle>, extent: (i32, i32)) -> Self {
        Self {
            triangles,
            extent,
            outline: ShapeOutline { points: vec![] },
        }
    }

    pub fn apply_transformations(&mut self) {
        for triangle in self.triangles.iter_mut() {
            for vertex in &mut triangle.points {
                let transform = &vertex.transform;
                let after = transform.apply_for_point((vertex.pos.0 as i32, vertex.pos.1 as i32));
                vertex.pos.0 = after.0 as f32;
                vertex.pos.1 = after.1 as f32;
                vertex.transform = Transform::new();
            }
        }
    }

    pub fn invalidate(&mut self) {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        self.outline.points.clear();
        let mut poly = Polygon::detriangulate(self);
        // create the convex shape
        poly.sort_vertices_by_angle();
        let mut seen = Vec::with_capacity(poly.vertices.len() / 2);
        for vertex in &poly.vertices {
            if seen.contains(vertex) {
                continue;
            }
            seen.push(*vertex);
        }
        poly.vertices = seen;

        if let Some(first) = poly.vertices.first() {
            poly.vertices.push(*first);
        }
        let length = poly.outline_length();
        let mut walking_dist = 0.0;
        for (v1, v2) in poly.vertices.iter().tuple_windows() {
            let progress = walking_dist / length;
            self.outline.points.push(PathPoint {
                progress,
                coords: (v1.x as i32, v1.y as i32),
            });
            walking_dist += geom::distance(*v1, *v2);
        }

        for triangle in &self.triangles {
            for (x, y, _) in triangle.points.iter().map(|v| v.pos) {
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }

        let width = max_x - min_x;
        let height = max_y - min_y;
        self.extent = (width as i32, height as i32);
    }

    fn set_z(&mut self, z: f32) {
        for tri in &mut self.triangles {
            tri.points.iter_mut().for_each(|v| v.pos.2 = z);
        }
    }

    pub fn combine(&mut self, other: &Shape) {
        self.triangles.extend(other.triangles.iter().cloned());
    }

    pub fn recenter(&mut self) {
        let mut total_x = 0;
        let mut total_y = 0;
        for triangle in self.triangles.iter() {
            let center = triangle.center();
            total_x += center.0;
            total_y += center.1;
        }
        let new_center = (
            total_x as f32 / self.triangles.len() as f32,
            total_y as f32 / self.triangles.len() as f32,
        );
        for triangle in self.triangles.iter_mut() {
            triangle
                .points
                .iter_mut()
                .for_each(|v| v.transform.origin.x = new_center.0);
            triangle
                .points
                .iter_mut()
                .for_each(|v| v.transform.origin.y = new_center.1);
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        for triangle in self.triangles.iter_mut() {
            triangle
                .points
                .iter_mut()
                .for_each(|v| v.transform = transform.clone());
        }
    }

    pub fn modify_transform<F: FnMut(&mut Transform)>(&mut self, mut transformation: F) {
        for triangle in self.triangles.iter_mut() {
            triangle
                .points
                .iter_mut()
                .for_each(|v| transformation(&mut v.transform));
        }
    }

    pub fn set_translate(&mut self, x: i32, y: i32) {
        self.modify_transform(|t| {
            t.translation.x = x as f32;
            t.translation.y = y as f32;
        });
    }

    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.modify_transform(|t| {
            t.scale.x = x;
            t.scale.y = y;
        });
    }

    pub fn set_origin(&mut self, x: i32, y: i32) {
        self.modify_transform(|t| {
            t.origin.x = x as f32;
            t.origin.y = y as f32;
        });
    }

    pub fn set_rotation(&mut self, rot: f32) {
        self.modify_transform(|t| {
            t.rotation = rot.to_radians();
        });
    }

    pub fn translated(mut self, x: i32, y: i32) -> Self {
        self.modify_transform(|t| {
            t.translation.x = x as f32;
            t.translation.y = y as f32;
        });
        self
    }

    pub fn rotated(mut self, r: f32) -> Self {
        self.modify_transform(|t| {
            t.rotation = r;
        });
        self
    }

    pub fn scaled(mut self, x: f32, y: f32) -> Self {
        self.modify_transform(|t| {
            t.scale.x = x;
            t.scale.y = y;
        });
        self
    }

    pub fn set_texture(&mut self, texture: TextureCtx) {
        if let Some(tex) = texture.texture {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;

            for triangle in &self.triangles {
                for (x, y) in triangle.points.iter().map(|v| (v.pos.0, v.pos.1)) {
                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }
            }

            let width = (max_x - min_x) as f32;
            let height = (max_y - min_y) as f32;

            let uv_bounds = tex.get_uv();
            let (uv_tl, uv_tr, uv_br, uv_bl) = (
                uv_bounds[0], // Top-left
                uv_bounds[1], // Top-right
                uv_bounds[2], // Bottom-right
                uv_bounds[3], // Bottom-left
            );

            for triangle in self.triangles.iter_mut() {
                for vertex in triangle.points.iter_mut() {
                    let x = vertex.pos.0;
                    let y = vertex.pos.1;

                    let normalized_u = (x - min_x) / width;
                    let normalized_v = 1.0 - (y - min_y) / height;

                    let u = uv_tl.0 + (uv_tr.0 - uv_tl.0) * normalized_u;
                    let v = uv_tl.1 + (uv_bl.1 - uv_tl.1) * normalized_v;

                    vertex.uv = (u, v);
                    vertex.texture = tex.id;
                    vertex.has_texture = 1.0;
                }
            }
        }
    }

    pub fn set_color(&mut self, color: RgbColor) {
        for triangle in self.triangles.iter_mut() {
            triangle
                .points
                .iter_mut()
                .for_each(|v| v.color = color.as_vec4());
        }
    }

    pub fn get_outline(&self) -> &ShapeOutline {
        &self.outline
    }

    pub fn create_morph(&self, to: &Shape) -> Morph {
        let mut morph = Morph::new(self, to);
        morph
    }

    pub fn print_points(&self) {
        for triangle in &self.triangles {
            for (x, y, _) in triangle.points.iter().map(|p| p.pos) {
                print!("({x}, {y}),");
            }
        }
        println!();
    }
}

#[derive(Clone, Savable)]
pub struct ShapeOutline {
    points: Vec<PathPoint>,
}

impl ShapeOutline {
    pub fn points(&self) -> &[PathPoint] {
        &self.points
    }
}

#[derive(Clone, Savable)]
pub struct PathPoint {
    progress: f32,
    coords: (i32, i32),
}

impl PathPoint {
    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn coords(&self) -> (i32, i32) {
        self.coords
    }
}
