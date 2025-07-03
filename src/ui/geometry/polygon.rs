use crate::math::vec::Vec2;
use crate::ui::geometry::geom;
use crate::ui::geometry::shape::Shape;
use crate::ui::geometry::SimpleRect;
use hashbrown::HashMap;
use itertools::Itertools;
use std::fmt::{Debug, Formatter, Write};

type Point = (i32, i32);
type Edge = (Point, Point);

#[derive(Clone)]
pub struct Intersection {
    pub point: Vec2,
    pub visited: bool,
}

impl Intersection {
    pub fn new(point: Vec2) -> Self {
        Intersection {
            point,
            visited: false,
        }
    }
}

#[derive(Clone)]
pub struct Polygon {
    pub vertices: Vec<Vec2>,
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

    pub fn outline_length(&self) -> f32 {
        let mut len = 0.0;
        for (a, b) in self.vertices.iter().tuple_windows() {
            len += geom::distance(*a, *b);
        }
        len
    }

    pub fn point_inside(&self, pt: Vec2) -> bool {
        let bounding = self.bounding_box();
        if !bounding.inside(pt.x as i32, pt.y as i32) {
            return false;
        }

        let mut seen = Vec::new();

        let ray_start = (bounding.x - 1, bounding.y - 1);
        let mut intersections = 0;
        for i in 0..self.vertices.len() {
            let start = self.vertices[i];
            let end = if i + 1 < self.vertices.len() {
                self.vertices[i + 1]
            } else {
                self.vertices[0]
            };

            let result = geom::lines_intersection(
                Vec2::new(ray_start.0 as f32, ray_start.1 as f32),
                pt,
                start,
                end,
            );
            if let Some(intersection) = result {
                intersections += 1;
                if self.vertices.iter().any(|v| *v == intersection) {
                    if let Some((_, other_start)) = seen.iter().find(|(i, _)| *i == intersection) {
                        let verify_line_start = *other_start;
                        let verify_line_end = if start == intersection { end } else { start };
                        if let None = geom::lines_intersection(
                            verify_line_start,
                            verify_line_end,
                            Vec2::new(ray_start.0 as f32, ray_start.1 as f32),
                            pt,
                        ) {
                            intersections -= 1;
                        }
                        intersections -= 1;
                    } else {
                        seen.push((
                            intersection,
                            if start == intersection { end } else { start },
                        ));
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
        Vec2 {
            x: sum_x / n,
            y: sum_y / n,
        }
    }

    pub fn sort_vertices_by_angle(&mut self) {
        let centroid = self.calculate_centroid();

        self.vertices.sort_by(|a, b| {
            let angle_a = (a.y - centroid.y).atan2(a.x - centroid.x);
            let angle_b = (b.y - centroid.y).atan2(b.x - centroid.x);

            angle_a
                .partial_cmp(&angle_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn sort_vertices_for_convex(&mut self) {
        self.sort_vertices_by_angle();
        self.vertices = self.convex_hull(&self.vertices);
    }

    fn convex_hull(&self, points: &Vec<Vec2>) -> Vec<Vec2> {
        if points.len() <= 1 {
            return points.clone();
        }

        // Sort points lexicographically (first by x, then by y)
        let mut sorted_points = points.clone();
        sorted_points.sort_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
        });

        // Build the lower hull
        let mut lower = Vec::new();
        for p in &sorted_points {
            while lower.len() >= 2
                && geom::cross_product(&lower[lower.len() - 2], &lower[lower.len() - 1], p) <= 0.0
            {
                lower.pop();
            }
            lower.push(*p);
        }

        // Build the upper hull
        let mut upper = Vec::new();
        for p in sorted_points.iter().rev() {
            while upper.len() >= 2
                && geom::cross_product(&upper[upper.len() - 2], &upper[upper.len() - 1], p) <= 0.0
            {
                upper.pop();
            }
            upper.push(*p);
        }

        // Remove the last point of each half because it's repeated at the beginning of the other half
        lower.pop();
        upper.pop();

        // Concatenate lower and upper hull to get the convex hull
        lower.extend(upper);
        lower
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
