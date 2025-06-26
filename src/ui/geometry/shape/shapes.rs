use std::f32::consts::PI;
use crate::math::vec::Vec4;
use crate::rendering::{InputVertex, Transform};
use crate::ui::geometry::shape::{Indices, Shape};
use crate::ui::geometry::SimpleRect;

fn vertex(x: i32, y: i32) -> InputVertex {
    InputVertex {
        transform: Transform::new(),
        pos: (x as f32, y as f32, 0.0),
        color: Vec4::default(),
        uv: (0.0, 0.0),
        texture: 0,
        has_texture: 0.0,
    }
}

pub fn rectangle0(x: i32, y: i32, w: i32, h: i32) -> Shape {
    Shape::new_with_extent(vec![
        vertex(x, y),
        vertex(x, y + h),
        vertex(x + w, y + h),
        vertex(x + w, y),
    ], Indices::TriangleStrip, SimpleRect::new(x, y, w, h))
}

pub fn rectangle1(x1: i32, y1: i32, x2: i32, y2: i32) -> Shape {
    Shape::new_with_extent(vec![
        vertex(x1, y1),
        vertex(x1, y2),
        vertex(x2, y2),
        vertex(x2, y1),
    ], Indices::TriangleStrip, SimpleRect::new(x1, y1, x2 - x1, y2 - y1))
}

pub fn rectangle2(rect: SimpleRect) -> Shape {
    Shape::new_with_extent(vec![
        vertex(rect.x, rect.y),
        vertex(rect.x, rect.y + rect.height),
        vertex(rect.x + rect.width, rect.y + rect.height),
        vertex(rect.x + rect.width, rect.y),
    ], Indices::TriangleStrip, rect)
}

pub fn arc0(cx: i32, cy: i32, radius: i32, offset: f32, range: f32, tri_count: i32) -> Shape {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    vertices.push(vertex(cx, cy));

    let step = range / tri_count as f32;
    
    for i in 0..=tri_count {
        let angle = offset + i as f32 * step;
        let x = cx + (angle.cos() * radius as f32).round() as i32;
        let y = cy + (angle.sin() * radius as f32).round() as i32;
        vertices.push(vertex(x, y));
    }
    
    for i in 1..=tri_count as usize {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }

    Shape::new(vertices, Indices::Manual(indices))
}

pub fn arc1(cx: i32, cy: i32, radius_x: i32, radius_y: i32, offset: f32, range: f32, tri_count: i32) -> Shape {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    vertices.push(vertex(cx, cy));

    let step = range / tri_count as f32;

    for i in 0..=tri_count {
        let angle = offset + i as f32 * step;
        let x = cx + (angle.cos() * radius_x as f32).round() as i32;
        let y = cy + (angle.sin() * radius_y as f32).round() as i32;
        vertices.push(vertex(x, y));
    }

    for i in 1..=tri_count as usize {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }

    Shape::new(vertices, Indices::Manual(indices))
}

pub fn circle0(cx: i32, cy: i32, radius: i32, tri_count: i32) -> Shape {
    arc0(cx, cy, radius, 0.0, PI + PI, tri_count)
}

pub fn ellipse0(cx: i32, cy: i32, radius_x: i32, radius_y: i32, tri_count: i32) -> Shape {
    arc1(cx, cy, radius_x, radius_y, 0.0, PI + PI, tri_count)
}