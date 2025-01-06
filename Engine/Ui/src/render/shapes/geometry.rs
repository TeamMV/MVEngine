use std::cmp::Ordering;
use mvcore::math::vec::Vec2;

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

    if (t >= 0.0 || t.is_close(0.0)) && (t <= 1.0 || t.is_close(1.0)) && (u >= 0.0 || u.is_close(0.0)) && (u <= 1.0 || u.is_close(1.0)) {
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
                if av.contains(&intersection) || bv.contains(&intersection) { continue; }
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
    if triangle.contains(&p) { return true; }
    let a = &triangle[0];
    let b = &triangle[1];
    let c = &triangle[2];

    let det = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);

    det * ((b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)) >= 0.0 &&
        det * ((c.x - b.x) * (p.y - b.y) - (c.y - b.y) * (p.x - b.x)) >= 0.0 &&
        det * ((a.x - c.x) * (p.y - c.y) - (a.y - c.y) * (p.x - c.x)) >= 0.0
}