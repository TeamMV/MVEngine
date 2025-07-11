use crate::math::vec::{Vec2, Vec4};
use crate::rendering::{InputVertex, Transform, Triangle};
use crate::ui::geometry::SimpleRect;
use crate::ui::geometry::shape::{Indices, Shape};
use gl::types::GLuint;
use std::f32::consts::PI;

pub fn vertex0(x: i32, y: i32) -> InputVertex {
    InputVertex {
        transform: Transform::new(),
        pos: (x as f32, y as f32, 0.0),
        color: Vec4::default(),
        uv: (0.0, 0.0),
        texture: 0,
        has_texture: 0.0,
    }
}

pub fn vertex1(x: i32, y: i32, tex: GLuint, uv: (f32, f32)) -> InputVertex {
    InputVertex {
        transform: Transform::new(),
        pos: (x as f32, y as f32, 0.0),
        color: Vec4::default(),
        uv,
        texture: tex,
        has_texture: 1.0,
    }
}

pub fn vertex2(x: f32, y: f32) -> InputVertex {
    InputVertex {
        transform: Transform::new(),
        pos: (x, y, 0.0),
        color: Vec4::default(),
        uv: (0.0, 0.0),
        texture: 0,
        has_texture: 0.0,
    }
}

pub fn vertex3(x: f32, y: f32, tex: GLuint, uv: (f32, f32)) -> InputVertex {
    InputVertex {
        transform: Transform::new(),
        pos: (x, y, 0.0),
        color: Vec4::default(),
        uv,
        texture: tex,
        has_texture: 1.0,
    }
}

pub fn rectangle0(x: i32, y: i32, w: i32, h: i32) -> Shape {
    Shape::new_with_extent(
        vec![
            vertex0(x, y),
            vertex0(x, y + h),
            vertex0(x + w, y),
            vertex0(x + w, y + h),
        ],
        Indices::TriangleStrip,
        SimpleRect::new(x, y, w, h),
    )
}

pub fn rectangle1(x1: i32, y1: i32, x2: i32, y2: i32) -> Shape {
    Shape::new_with_extent(
        vec![
            vertex0(x1, y1),
            vertex0(x1, y2),
            vertex0(x2, y1),
            vertex0(x2, y2),
        ],
        Indices::TriangleStrip,
        SimpleRect::new(x1, y1, x2 - x1, y2 - y1),
    )
}

pub fn rectangle2(rect: SimpleRect) -> Shape {
    Shape::new_with_extent(
        vec![
            vertex0(rect.x, rect.y),
            vertex0(rect.x, rect.y + rect.height),
            vertex0(rect.x + rect.width, rect.y),
            vertex0(rect.x + rect.width, rect.y + rect.height),
        ],
        Indices::TriangleStrip,
        rect,
    )
}

pub fn arc0(cx: i32, cy: i32, radius: i32, offset: f32, range: f32, tri_count: i32) -> Shape {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    vertices.push(vertex0(cx, cy));

    let step = range / tri_count as f32;

    for i in 0..=tri_count {
        let angle = offset + i as f32 * step;
        let x = cx + (angle.cos() * radius as f32).round() as i32;
        let y = cy + (angle.sin() * radius as f32).round() as i32;
        vertices.push(vertex0(x, y));
    }

    for i in 1..=tri_count as usize {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }

    Shape::new(vertices, Indices::Manual(indices))
}

pub fn arc1(
    cx: i32,
    cy: i32,
    radius_x: i32,
    radius_y: i32,
    offset: f32,
    range: f32,
    tri_count: i32,
) -> Shape {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    vertices.push(vertex0(cx, cy));

    let step = range / tri_count as f32;

    for i in 0..=tri_count {
        let angle = offset + i as f32 * step;
        let x = cx + (angle.cos() * radius_x as f32).round() as i32;
        let y = cy + (angle.sin() * radius_y as f32).round() as i32;
        vertices.push(vertex0(x, y));
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

pub fn triangle0(x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32) -> Shape {
    let v1 = vertex0(x1, y1);
    let v2 = vertex0(x2, y2);
    let v3 = vertex0(x3, y3);
    Shape::new(vec![v1, v2, v3], Indices::Triangles)
}

pub fn triangle1(p1: (i32, i32), p2: (i32, i32), p3: (i32, i32)) -> Shape {
    let v1 = vertex0(p1.0, p1.1);
    let v2 = vertex0(p2.0, p2.1);
    let v3 = vertex0(p3.0, p3.1);
    Shape::new(vec![v1, v2, v3], Indices::Triangles)
}

pub fn triangle2(v1: Vec2, v2: Vec2, v3: Vec2) -> Shape {
    let (x1, y1) = v1.as_i32_tuple();
    let (x2, y2) = v2.as_i32_tuple();
    let (x3, y3) = v3.as_i32_tuple();
    triangle0(x1, y1, x2, y2, x3, y3)
}

pub fn triangle3(triangle: Triangle) -> Shape {
    Shape::new(triangle.points.to_vec(), Indices::Triangles)
}

pub fn clipped_rectangle(outer: SimpleRect, inner: SimpleRect) -> Shape {
    let mut shape = rectangle1(outer.x, outer.y, outer.x + outer.width, inner.y); // Top

    shape.combine(&rectangle1(
        outer.x,
        inner.y,
        inner.x,
        inner.y + inner.height,
    )); // Left

    shape.combine(&rectangle1(
        inner.x + inner.width,
        inner.y,
        outer.x + outer.width,
        inner.y + inner.height,
    )); // Right

    shape.combine(&rectangle1(
        outer.x,
        inner.y + inner.height,
        outer.x + outer.width,
        outer.y + outer.height,
    )); // Bottom

    shape
}

pub fn void_rectangle0(x: i32, y: i32, width: i32, height: i32, thickness: i32) -> Shape {
    let mut s = rectangle0(x, y, thickness, height);
    s.combine(&rectangle0(x + thickness, y, width - thickness * 2, thickness));
    s.combine(&rectangle0(x + thickness, y + height - thickness, width - thickness * 2, thickness));
    s.combine(&rectangle0(x + width - thickness, y, thickness, height));
    s
}