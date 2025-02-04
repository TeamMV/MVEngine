use std::fmt::{Debug, Formatter, Write};
use hashbrown::HashMap;
use itertools::Itertools;
use crate::math::vec::Vec2;
use crate::ui::geometry::SimpleRect;
use crate::ui::rendering::ctx::DrawShape;
use crate::ui::rendering::shapes::geometry;

type Point = (i32, i32);
type Edge = (Point, Point);

#[derive(Clone)]
pub struct Intersection {
    pub point: Vec2,
    pub visited: bool,
}

impl Intersection {
    fn new(point: Vec2) -> Self {
        Intersection {
            point,
            visited: false,
        }
    }
}

#[derive(Clone)]
pub struct Polygon {
    pub vertices: Vec<Vec2>
}

impl Debug for Intersection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pt = self.point;
        f.write_char('(')?;
        f.write_str(format!("{}, {}", pt.x, pt.y).as_str())?;
        f.write_char(')')?;
        Ok(())
    }
}

impl Polygon {
    pub fn bounding_box(&self) -> SimpleRect {
        let mut min = (i32::MAX, i32::MAX);
        let mut max = (i32::MIN, i32::MIN);
        for vertex in &self.vertices {
            if vertex.x < min.0 as f32 {
                min.0 = vertex.x as i32;
            }
            if vertex.y < min.1 as f32 {
                min.1 = vertex.y as i32;
            }
            if vertex.x > max.0 as f32 {
                max.0 = vertex.x as i32;
            }
            if vertex.y > max.1 as f32 {
                max.1 = vertex.y as i32;
            }
        }
        SimpleRect::new(min.0, min.1, max.0 - min.0, max.1 - min.1)
    }

    fn get_slope(v1: Vec2, v2: Vec2) -> f32 {
        (v2.y - v1.y) / (v2.x - v1.x)
    }

    pub fn point_inside(&self, pt: Vec2) -> bool {
        let bounding = self.bounding_box();
        if !bounding.inside(pt.x as i32, pt.y as i32) { return false; }

        let mut seen = Vec::new();

        let ray_start = (bounding.x - 1, bounding.y - 1);
        let mut intersections = 0;
        for i in 0..self.vertices.len() {
            let start = self.vertices[i];
            let end = if i + 1 < self.vertices.len() { self.vertices[i + 1] } else { self.vertices[0] };

            let result = geometry::lines_intersection(Vec2::new(ray_start.0 as f32, ray_start.1 as f32), pt, start, end);
            if let Some(intersection) = result {
                intersections += 1;
                if self.vertices.iter().any(|v| *v == intersection) {
                    if let Some((_, other_start)) = seen.iter().find(|(i, _)| *i == intersection) {
                        let verify_line_start = *other_start;
                        let verify_line_end = if start == intersection { end } else { start };
                        if let None = geometry::lines_intersection(verify_line_start, verify_line_end, Vec2::new(ray_start.0 as f32, ray_start.1 as f32), pt) {
                            intersections -= 1;
                        }
                        intersections -= 1;
                    } else {
                        seen.push((intersection, if start == intersection { end } else { start }));
                    }
                }
            }
        }
        intersections & 1 == 1
    }

    pub fn calculate_centroid(&self) -> Vec2 {
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;

        for vertex in &self.vertices {
            sum_x += vertex.x;
            sum_y += vertex.y;
        }

        let n = self.vertices.len() as f32;
        Vec2 { x: sum_x / n, y: sum_y / n }
    }

    pub fn sort_vertices_by_angle(&mut self) {
        let centroid = self.calculate_centroid();

        self.vertices.sort_by(|a, b| {
            let angle_a = (a.y - centroid.y).atan2(a.x - centroid.x);
            let angle_b = (b.y - centroid.y).atan2(b.x - centroid.x);

            angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn detriangulate(shape: &DrawShape) -> Vec<Self> {
        let edges = Self::collect_edges(shape);
        let boundary_edges = Self::find_boundary_edges(&edges);

        let mut polygons = Vec::new();

        todo!();

        polygons
    }

    fn collect_edges(shape: &DrawShape) -> HashMap<Edge, usize> {
        let mut edge_count = HashMap::new();
        for triangle in shape.triangles.iter() {
            for edge in [
                (triangle.points[0].pos, triangle.points[1].pos),
                (triangle.points[1].pos, triangle.points[2].pos),
                (triangle.points[2].pos, triangle.points[0].pos),
            ].map(|x| ((x.0.0 as i32, x.0.1 as i32), (x.1.0 as i32, x.1.1 as i32))) {
                let normalized_edge = if edge.0 < edge.1 { edge } else { (edge.1, edge.0) };
                *edge_count.entry(normalized_edge).or_insert(0) += 1;
            }
        }
        edge_count
    }

    fn find_boundary_edges(edge_count: &HashMap<Edge, usize>) -> Vec<Edge> {
        edge_count
            .iter()
            .filter_map(|(edge, &count)| if count == 1 { Some(*edge) } else { None })
            .collect()
    }
}

impl Debug for Polygon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let len = self.vertices.len();
        for i in 0..len {
            let vec = self.vertices[i];
            f.write_char('(')?;
            f.write_str(format!("{}, {}", vec.x, vec.y).as_str())?;
            f.write_char(')')?;
            if i != len - 1 {
                f.write_str(", ")?
            }
        }
        Ok(())
    }
}