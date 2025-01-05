use std::cmp::Ordering;
use hashbrown::HashMap;
use itertools::Itertools;
use mvcore::math::vec::Vec2;
use crate::render::ctx::DrawShape;
use crate::render::shapes::Param;
use crate::render::shapes::polygon::{Intersection, Polygon};

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
    let polygons_a = Polygon::detriangulate(input);
    let polygons_b = Polygon::detriangulate(other);
    for pa in polygons_a.iter() {
        for pb in polygons_b.iter() {
            let out = union_polygons(pa, pb);
        }
    }

    Err("idk".to_string())
}

pub fn union_polygons(a: &Polygon, b: &Polygon) -> Polygon {
    let mut intersections = a.get_intersections(b);
    walk_polygon(a, b, &mut intersections, true).clone()
}

pub fn intersect_polygons(a: &Polygon, b: &Polygon) -> Polygon {
    let mut intersections = a.get_intersections(b);
    walk_polygon(a, b, &mut intersections, false).clone()
}

fn distance(v1: Vec2, v2: Vec2) -> f32 {
    ((v1.x - v2.x) * (v1.x - v2.x) + (v1.y - v2.y) * (v1.y - v2.y)).sqrt()
}

fn is_point_on_line(pt: Vec2, line_v1: Vec2, line_v2: Vec2) -> bool {
    (distance(line_v1, pt) + distance(line_v2, pt) - distance(line_v1, line_v2)).abs() < 0.01
}

fn find_next_vertex(current_vertex: Vec2, polygon: &Polygon) -> Vec2 {
    for i in 0..polygon.vertices.len() {
        let v = polygon.vertices[i];
        if v == current_vertex {
            println!("got, index: {i}, amt: {}", polygon.vertices.len());
            return polygon.vertices[(i + 1) % polygon.vertices.len()];
        }
    }
    for i in 0..polygon.vertices.len() {
        let start = polygon.vertices[i];
        let end = polygon.vertices[(i + 1) % polygon.vertices.len()];
        println!("searching... start: {:?}, end: {:?}", start, end);
        if is_point_on_line(current_vertex, start, end) {
            return end;
        }
    }
    current_vertex
}

fn walk_polygon(subject: &Polygon, clipping: &Polygon, intersections: &mut Vec<Intersection>, union: bool) -> Polygon {
    let mut resulting_polygon = Polygon { vertices: vec![] };
    let mut current_vertex = subject.vertices[0];
    let mut vertex_count = 0;
    let mut clip_found = false;
    while vertex_count < subject.vertices.len() {
        if clipping.point_inside(current_vertex) ^ union {
            clip_found = true;
            break;
        }
        current_vertex = subject.vertices[vertex_count];
        vertex_count += 1;
    }
    if !clip_found && !union {
        if intersections.len() > 0 {
            current_vertex = intersections[0].point;
        } else {
            return clipping.clone();
        }
    }
    let mut in_clip = !union;

    if in_clip || union { resulting_polygon.vertices.push(current_vertex) }

    let mut counter = 1;
    loop {
        let next_vertex = if in_clip ^ union {
            find_next_vertex(current_vertex, subject)
        } else {
            find_next_vertex(current_vertex, clipping)
        };
        println!("current: {:?}, next: {:?}, inside: {in_clip}", current_vertex, next_vertex);
        counter += 1;

        if let Some(intersection) = intersections
            .iter_mut()
            .filter(|i| !i.visited && is_point_on_line(i.point, current_vertex, next_vertex) && i.point != current_vertex)
            .sorted_by(|i1, i2| distance(i1.point, current_vertex).partial_cmp(&distance(i2.point, current_vertex)).unwrap_or(Ordering::Equal))
            .next()
        {
            resulting_polygon.vertices.push(intersection.point);
            intersection.visited = true;
            current_vertex = intersection.point;
            in_clip = !in_clip;
            println!("switched, got intersection: {:?}", intersection.point);
        } else {
            resulting_polygon.vertices.push(next_vertex);
            current_vertex = next_vertex;
        }

        if resulting_polygon.vertices.len() > 3 && current_vertex == resulting_polygon.vertices[0] || counter > 20 {
            break;
        }
    }
    resulting_polygon.vertices.dedup();
    resulting_polygon
}


fn compute_intersect(input: &DrawShape, other: &DrawShape) -> Result<DrawShape, String> {
    todo!()
}

fn compute_difference(input: &DrawShape, other: &DrawShape) -> Result<DrawShape, String> {
    todo!()
}