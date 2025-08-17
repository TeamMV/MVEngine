use crate::math::vec::Vec2;
use crate::ui::geometry::SimpleRect;
use std::cmp::Ordering;
use std::f32::consts::{FRAC_1_PI, FRAC_2_PI, PI, TAU};
use num_traits::Zero;
//////////////////////////////////////////////////////////////////////////////////////////////////////
// This module is a collection of useful math stuff from math websites without css or stackoverflow //
//////////////////////////////////////////////////////////////////////////////////////////////////////

trait Closeable {
    const EPSILON: Self;
    fn is_close(self, rhs: Self) -> bool
    where
        Self: std::ops::Sub<Output = Self> + PartialOrd + Sized,
    {
        let ord = if let Some(ord) = self.partial_cmp(&rhs) {
            ord
        } else {
            Ordering::Equal
        };

        let diff = match ord {
            Ordering::Equal => return true,
            Ordering::Greater => self - rhs,
            Ordering::Less => rhs - self,
        };

        if let Some(Ordering::Greater) = diff.partial_cmp(&Self::EPSILON) {
            return false;
        }
        true
    }
}

impl Closeable for f32 {
    const EPSILON: Self = f32::EPSILON;
}

pub fn lines_intersection(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2) -> Option<Vec2> {
    let denom = (v1.x - v2.x) * (v3.y - v4.y) - (v1.y - v2.y) * (v3.x - v4.x);
    if denom.is_close(0.0) {
        return None;
    }

    let t = ((v1.x - v3.x) * (v3.y - v4.y) - (v1.y - v3.y) * (v3.x - v4.x)) / denom;
    let u = ((v1.x - v3.x) * (v1.y - v2.y) - (v1.y - v3.y) * (v1.x - v2.x)) / denom;

    if (t >= 0.0 || t.is_close(0.0))
        && (t <= 1.0 || t.is_close(1.0))
        && (u >= 0.0 || u.is_close(0.0))
        && (u <= 1.0 || u.is_close(1.0))
    {
        let x = v1.x + t * (v2.x - v1.x);
        let y = v1.y + t * (v2.y - v1.y);
        Some(Vec2::new(x, y))
    } else {
        None
    }
}

pub fn distance(v1: Vec2, v2: Vec2) -> f32 {
    ((v1.x - v2.x) * (v1.x - v2.x) + (v1.y - v2.y) * (v1.y - v2.y)).sqrt()
}

pub fn is_point_on_line(pt: Vec2, line_v1: Vec2, line_v2: Vec2) -> bool {
    (distance(line_v1, pt) + distance(line_v2, pt) - distance(line_v1, line_v2)).abs() < 0.001
}

pub fn get_triangle_intersections(av: &[Vec2; 3], bv: &[Vec2; 3]) -> Option<Vec<Vec2>> {
    let mut intersections = Vec::new();

    for line_a in [(av[0], av[1]), (av[1], av[2]), (av[2], av[0])] {
        for line_b in [(bv[0], bv[1]), (bv[1], bv[2]), (bv[2], bv[0])] {
            if let Some(intersection) = lines_intersection(line_a.0, line_a.1, line_b.0, line_b.1) {
                if av.contains(&intersection) || bv.contains(&intersection) {
                    continue;
                }
                intersections.push(intersection);
            }
        }
    }

    if intersections.is_empty() {
        None
    } else {
        Some(intersections)
    }
}

pub fn is_point_in_triangle(triangle: &[Vec2; 3], p: Vec2) -> bool {
    if triangle.contains(&p) {
        return true;
    }
    let a = &triangle[0];
    let b = &triangle[1];
    let c = &triangle[2];

    let det = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);

    det * ((b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)) >= 0.0
        && det * ((c.x - b.x) * (p.y - b.y) - (c.y - b.y) * (p.x - b.x)) >= 0.0
        && det * ((a.x - c.x) * (p.y - c.y) - (a.y - c.y) * (p.x - c.x)) >= 0.0
}

pub fn get_slope(v1: Vec2, v2: Vec2) -> f32 {
    (v2.y - v1.y) / (v2.x - v1.x)
}

// it might get used in the future?
#[allow(unused)]
pub(crate) fn lerp(p1: Vec2, p2: Vec2, p: f32) -> Vec2 {
    let x = p1.x + (p2.x - p1.x) * p;
    let y = p1.y + (p2.y - p1.y) * p;
    Vec2::new(x, y)
}

// this one too
#[allow(unused)]
pub(crate) fn is_convex(a: Vec2, b: Vec2, c: Vec2) -> bool {
    let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    cross < 0.0 // Counter-clockwise = convex
}

pub(crate) fn cross_product(p1: &Vec2, p2: &Vec2, p3: &Vec2) -> f32 {
    (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x)
}

fn get_distance_to_side(from: &Vec2, x1: f32, y1: f32, x2: f32, y2: f32, side_width: f32) -> f32 {
    ((y2 - y1) * from.x - (x2 - x1) * from.y + x2 * y1 - y2 * x1) / side_width
}

pub fn get_distance_to_triangle(from: &Vec2, triangle: &[Vec2; 3]) -> f32 {
    let ab_width = (triangle[1].x - triangle[0].x) * (triangle[1].x - triangle[0].x)
        + (triangle[1].y - triangle[0].y) * (triangle[1].y - triangle[0].y);

    let bc_width = (triangle[2].x - triangle[1].x) * (triangle[2].x - triangle[1].x)
        + (triangle[2].y - triangle[1].y) * (triangle[2].y - triangle[1].y);

    let ca_width = (triangle[0].x - triangle[2].x) * (triangle[0].x - triangle[2].x)
        + (triangle[0].y - triangle[2].y) * (triangle[0].y - triangle[2].y);

    let ab_distance = get_distance_to_side(
        from,
        triangle[0].x,
        triangle[0].y,
        triangle[1].x,
        triangle[1].y,
        ab_width,
    );
    let bc_distance = get_distance_to_side(
        from,
        triangle[1].x,
        triangle[1].y,
        triangle[2].x,
        triangle[2].y,
        bc_width,
    );
    let ca_distance = get_distance_to_side(
        from,
        triangle[2].x,
        triangle[2].y,
        triangle[0].x,
        triangle[0].y,
        ca_width,
    );

    ab_distance.max(bc_distance).max(ca_distance)
}

pub fn get_distance_to_line(l1: Vec2, l2: Vec2, p: Vec2) -> f32 {
    let y2d = l2.y - l1.y;
    let x2d = l2.x - l1.x;

    (y2d * p.x - x2d * p.y + l2.x * l1.y - l2.y * l1.x).abs() / (y2d * y2d + x2d * x2d).sqrt()
}

pub fn length(v: Vec2) -> f32 {
    (v.x * v.x + v.y * v.y).sqrt()
}

pub fn normalize(v: Vec2) -> Vec2 {
    let len = length(v);
    if len == 0.0 {
        v
    } else {
        Vec2 {
            x: v.x / len,
            y: v.y / len,
        }
    }
}

pub fn perpendicular(v: Vec2) -> Vec2 {
    Vec2 { x: -v.y, y: v.x }
}

pub fn add(a: Vec2, b: Vec2) -> Vec2 {
    Vec2 {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}

pub fn sub(a: Vec2, b: Vec2) -> Vec2 {
    Vec2 {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

pub fn scale(v: Vec2, factor: f32) -> Vec2 {
    Vec2 {
        x: v.x * factor,
        y: v.y * factor,
    }
}

pub fn scale_xy(v: Vec2, x: f32, y: f32) -> Vec2 {
    Vec2 {
        x: v.x * x,
        y: v.y * y,
    }
}

pub fn remap_point(pt: Vec2, from: &SimpleRect, to: &SimpleRect) -> Vec2 {
    let norm_x = (pt.x - from.x as f32) / from.width as f32;
    let norm_y = (pt.y - from.y as f32) / from.height as f32;

    Vec2 {
        x: to.x as f32 + norm_x * to.width as f32,
        y: to.y as f32 + norm_y * to.height as f32,
    }
}

pub fn angle_between_points(a: Vec2, b: Vec2) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;

    let angle = dy.atan2(dx) - PI / 2.0;

    angle
}

pub fn vector_angle(v: Vec2) -> f32 {
    let angle = v.y.atan2(v.x) - std::f32::consts::FRAC_PI_2;

    (angle + TAU) % TAU
}

pub fn rotate_vector(v: Vec2, angle: f32) -> Vec2 {
    let cos_theta = angle.cos();
    let sin_theta = angle.sin();

    Vec2 {
        x: v.x * cos_theta - v.y * sin_theta,
        y: v.x * sin_theta + v.y * cos_theta,
    }
}

pub fn invert(v: Vec2) -> Vec2 {
    Vec2 { x: -v.x, y: -v.y }
}

pub fn is_vec_zero(v: Vec2) -> bool {
    v.x.is_zero() && v.y.is_zero()
}