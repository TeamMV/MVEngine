use hashbrown::HashMap;
use itertools::Itertools;
use mvcore::math::vec::Vec2;
use crate::geometry::SimpleRect;
use crate::render::ctx::DrawShape;

type Point = (i32, i32);
type Edge = (Point, Point);

const NO: i32 = 0;
const YES: i32 = 1;
const COLLINEAR: i32 = 2;

fn lines_intersecting(
    v1x1: f32,
    v1y1: f32,
    v1x2: f32,
    v1y2: f32,
    v2x1: f32,
    v2y1: f32,
    v2x2: f32,
    v2y2: f32,
) -> i32 {
    let mut d1: f32;
    let mut d2: f32;
    let a1: f32;
    let b1: f32;
    let c1: f32;
    let a2: f32;
    let b2: f32;
    let c2: f32;

    a1 = v1y2 - v1y1;
    b1 = v1x1 - v1x2;
    c1 = (v1x2 * v1y1) - (v1x1 * v1y2);

    d1 = (a1 * v2x1) + (b1 * v2y1) + c1;
    d2 = (a1 * v2x2) + (b1 * v2y2) + c1;

    if d1 > 0.0 && d2 > 0.0 {
        return NO;
    }
    if d1 < 0.0 && d2 < 0.0 {
        return NO;
    }

    a2 = v2y2 - v2y1;
    b2 = v2x1 - v2x2;
    c2 = (v2x2 * v2y1) - (v2x1 * v2y2);

    d1 = (a2 * v1x1) + (b2 * v1y1) + c2;
    d2 = (a2 * v1x2) + (b2 * v1y2) + c2;

    if d1 > 0.0 && d2 > 0.0 {
        return NO;
    }
    if d1 < 0.0 && d2 < 0.0 {
        return NO;
    }

    if (a1 * b2) - (a2 * b1) == 0.0 {
        return COLLINEAR;
    }

    YES
}

#[derive(Debug, Clone)]
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

            let result = Self::are_intersecting(Vec2::new(ray_start.0 as f32, ray_start.1 as f32), pt, start, end);
            if let Some(intersection) = result {
                intersections += 1;
                if self.vertices.iter().any(|v| *v == intersection) {
                    if let Some((_, other_start)) = seen.iter().find(|(i, _)| *i == intersection) {
                        let verify_line_start = *other_start;
                        let verify_line_end = if start == intersection { end } else { start };
                        if let None = Self::are_intersecting(verify_line_start, verify_line_end, Vec2::new(ray_start.0 as f32, ray_start.1 as f32), pt) {
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

    pub fn are_intersecting(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2) -> Option<Vec2> {
        let denom = (v1.x - v2.x) * (v3.y - v4.y) - (v1.y - v2.y) * (v3.x - v4.x);
        if denom == 0.0 {
            return None;
        }

        let t = ((v1.x - v3.x) * (v3.y - v4.y) - (v1.y - v3.y) * (v3.x - v4.x)) / denom;
        let u = ((v1.x - v3.x) * (v1.y - v2.y) - (v1.y - v3.y) * (v1.x - v2.x)) / denom;

        if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
            let x = v1.x + t * (v2.x - v1.x);
            let y = v1.y + t * (v2.y - v1.y);
            Some(Vec2::new(x, y))
        } else {
            None
        }
    }

    pub fn get_intersections(&self, other: &Polygon) -> Vec<Intersection> {
        let mut intersections = Vec::new();

        for i in 0..self.vertices.len() {
            let v1 = self.vertices[i];
            let v2 = if i + 1 < self.vertices.len() { self.vertices[i + 1] } else { self.vertices[0] };

            for j in 0..other.vertices.len() {
                let v3 = other.vertices[j];
                let v4 = if j + 1 < other.vertices.len() { other.vertices[j + 1] } else { other.vertices[0] };

                if let Some(intersection) = Self::are_intersecting(v1, v2, v3, v4) {
                    if !intersections.iter().any(|i: &Intersection| i.point == intersection) {
                        intersections.push(Intersection::new(intersection));
                    }
                }
                //if other.point_inside(v1) {
                //    if !intersections.iter().any(|i| i.point == v1) {
                //        intersections.push(Intersection::new(v1));
                //    }
                //}
                //if self.point_inside(v3) {
                //    if !intersections.iter().any(|i| i.point == v3) {
                //        intersections.push(Intersection::new(v3));
                //    }
                //}
            }
        }
        intersections
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
        let mut remaining_edges = boundary_edges;

        while !remaining_edges.is_empty() {
            let polygon = Self::order_edges(remaining_edges.clone());
            polygons.push(polygon.clone());

            remaining_edges.retain(|edge| !polygon.contains(&edge.0) && !polygon.contains(&edge.1));
        }

        polygons.into_iter().map(|p| {
            Polygon {
                vertices: p.into_iter().map(|v| Vec2::new(v.0 as f32, v.1 as f32)).collect_vec(),
            }
        }).collect_vec()
    }

    fn collect_edges(shape: &DrawShape) -> HashMap<Edge, usize> {
        let mut edge_count = HashMap::new();
        for triangle in shape.triangles.iter() {
            for edge in [
                (triangle.points[0], triangle.points[1]),
                (triangle.points[1], triangle.points[2]),
                (triangle.points[2], triangle.points[0]),
            ] {
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

    fn order_edges(boundary_edges: Vec<Edge>) -> Vec<Point> {
        let mut ordered_polygon = Vec::new();
        let mut edge_map: HashMap<Point, Point> = HashMap::new();

        for (start, end) in &boundary_edges {
            edge_map.insert(*start, *end);
        }

        let mut start_point = boundary_edges[0].0;
        ordered_polygon.push(start_point);

        while let Some(&next_point) = edge_map.get(&start_point) {
            ordered_polygon.push(next_point);
            start_point = next_point;
            if start_point == ordered_polygon[0] {
                break;
            }
        }

        ordered_polygon
    }
}