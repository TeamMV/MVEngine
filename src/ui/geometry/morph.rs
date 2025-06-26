use crate::color::RgbColor;
use crate::math::vec::{Vec2, Vec4};
use crate::rendering::{InputVertex, Transform, InputVertex};
use crate::ui::geometry::geom;
use crate::ui::geometry::polygon::Polygon;
use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::ctx;
use crate::ui::rendering::ctx::DrawContext2D;
use itertools::Itertools;
use mvutils::utils::PClamp;

const CONTOUR_THRESHOLD: f32 = 5.0;

pub struct Morph {
    from_sdf: SDF,
    to_sdf: SDF,
    current_sdf: SDF,
}

impl Morph {
    pub fn new(from: &Shape, to: &Shape) -> Self {
        let sdf_width = from.extent.0.max(to.extent.0) as u32;
        let sdf_height = from.extent.1.max(to.extent.1) as u32;

        let mut from_sdf = SDF::new(sdf_width, sdf_height);
        let mut to_sdf = SDF::new(sdf_width, sdf_height);
        let current_sdf = SDF::new(sdf_width, sdf_height);

        from_sdf.populate(from);
        to_sdf.populate(to);

        Self {
            from_sdf,
            to_sdf,
            current_sdf,
        }
    }

    pub fn animate_frame(&mut self, progress: f32) -> Shape {
        self.from_sdf
            .interpolate_into(&self.to_sdf, &mut self.current_sdf, progress);
        sdf_to_shape(&self.current_sdf)
    }

    pub fn debug_draw(&mut self, ctx: &mut DrawContext2D) {
        self.current_sdf.debug_draw(ctx);
    }
}

struct SDF {
    size: (u32, u32),
    grid: Vec<f32>,
}

impl SDF {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            size: (width, height),
            grid: vec![f32::MAX; (width * height) as usize],
        }
    }

    pub fn populate(&mut self, shape: &Shape) {
        // let center = Vec2::new(shape.extent.0 as f32, shape.extent.1 as f32);
        // let mut init = true;
        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                let p = Vec2::new(x as f32, y as f32);
                let mut min_dist = f32::MAX;

                let inside = false;
                // let outline = shape.get_outline();
                //for (a, b) in outline.points().iter().tuple_windows() {
                //    let l1 = Vec2::new(a.coords().0 as f32, a.coords().1 as f32);
                //    let l2 = Vec2::new(b.coords().0 as f32, b.coords().1 as f32);
                //    if let Some(_) = geom::lines_intersection(l1, l2, center, p) {
                //        inside = false;
                //    }
                //}

                let outline = shape.get_outline();
                for (a, b) in outline.points().iter().tuple_windows() {
                    let l1 = Vec2::new(a.coords().0 as f32, a.coords().1 as f32);
                    let l2 = Vec2::new(b.coords().0 as f32, b.coords().1 as f32);
                    let mut dist = geom::get_distance_to_line(l1, l2, p);
                    //let mut dist = geom::distance(l1, p);
                    if inside {
                        dist = -dist;
                    }
                    min_dist = min_dist.min(dist);
                }

                self.grid[(x + (y * self.size.0)) as usize] = min_dist;
            }
        }
    }

    pub fn interpolate_into(&self, to: &SDF, target: &mut SDF, progress: f32) {
        for i in 0..self.grid.len() {
            target.grid[i] = (1.0 - progress) * self.grid[i] + progress * to.grid[i];
        }
    }

    pub fn extract_contour(&self, threshold: f32) -> Vec<Vec2> {
        let mut contour = vec![];
        let width = self.size.0 as usize;
        let height = self.size.1 as usize;

        for y in 0..height {
            for x in 0..width {
                let sdf_val = self.grid[y * width + x];

                //println!("{}", sdf_val);
                if sdf_val.abs() < threshold {
                    contour.push(Vec2::new(x as f32, y as f32));
                }
            }
        }

        if let Some(first) = contour.first() {
            contour.push(*first);
        }

        contour
    }

    pub fn debug_draw(&self, ctx: &mut DrawContext2D) {
        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                let val = self.grid[(y * self.size.0 + x) as usize];
                //println!("{}", val);
                // let mapped = val.map(&(-10.0..10.0), &(0.0..255.0)).p_clamp(0.0, 255.0);
                let mapped = (val * 2.0).p_clamp(0.0, 255.0);
                let col = RgbColor::white().alpha(mapped as u8);
                let mut rect = ctx::rectangle()
                    .xywh(x as i32, y as i32, 1, 1)
                    .color(col)
                    .create();
                rect.set_translate(100, 100);
                rect.set_origin(0, 0);
                rect.set_scale(2.0, 2.0);
                ctx.shape(rect);
            }
        }
    }
}

fn triangulate_contour(contour: Vec<Vec2>) -> Vec<[Vec2; 3]> {
    let mut triangles = vec![];

    if contour.len() < 3 {
        return triangles;
    }

    let first_point = contour[0];

    for i in 1..contour.len() - 1 {
        let second_point = contour[i];
        let third_point = contour[i + 1];

        triangles.push([first_point, second_point, third_point]);
    }

    triangles
}

fn sdf_to_shape(sdf: &SDF) -> Shape {
    let contour = sdf.extract_contour(CONTOUR_THRESHOLD);
    let mut poly = Polygon { vertices: contour };
    poly.sort_vertices_by_angle();
    let contour = poly.vertices;
    //println!("{contour:?}");
    //println!();
    let triangle_vertices = triangulate_contour(contour);

    let triangles = triangle_vertices
        .into_iter()
        .map(|verts| {
            InputVertex {
                points: [
                    InputVertex {
                        transform: Transform::new(), // Default transform
                        pos: (verts[0].x, verts[0].y, f32::INFINITY),
                        color: Vec4::new(1.0, 1.0, 1.0, 1.0), // Default color
                        uv: (0.0, 0.0),
                        texture: 0,
                        has_texture: 0.0,
                    },
                    InputVertex {
                        transform: Transform::new(),
                        pos: (verts[1].x, verts[1].y, f32::INFINITY),
                        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                        uv: (0.0, 0.0),
                        texture: 0,
                        has_texture: 0.0,
                    },
                    InputVertex {
                        transform: Transform::new(),
                        pos: (verts[2].x, verts[2].y, f32::INFINITY),
                        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                        uv: (0.0, 0.0),
                        texture: 0,
                        has_texture: 0.0,
                    },
                ],
            }
        })
        .collect();

    Shape::new_with_extent(triangles, (sdf.size.0 as i32, sdf.size.1 as i32))
}
