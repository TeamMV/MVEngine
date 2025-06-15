use crate::color::RgbColor;
use crate::graphics::Drawable;
use crate::rendering::control::RenderController;
use crate::rendering::{InputVertex, Quad, RenderContext};
use crate::ui::context::UiContext;
use crate::ui::geometry::shape::Shape;
use crate::ui::geometry::{Rect, SimpleRect};
use mvutils::utils::PClamp;
use mvutils::Savable;

pub const EDGE_LEFT: usize = 0;
pub const EDGE_TOP: usize = 1;
pub const EDGE_RIGHT: usize = 2;
pub const EDGE_BOTTOM: usize = 3;

pub const CORNER_BL: usize = 0;
pub const CORNER_TL: usize = 1;
pub const CORNER_TR: usize = 2;
pub const CORNER_BR: usize = 3;

#[derive(Clone, Savable)]
pub struct AdaptiveShape {
    pub edges: [Option<Shape>; 4],   //l, t, r, b
    pub corners: [Option<Shape>; 4], //bl, tl, tr, br
    pub center: Option<Shape>,
}

impl AdaptiveShape {
    pub fn new(
        mut bl: Option<Shape>,
        mut l: Option<Shape>,
        mut tl: Option<Shape>,
        mut t: Option<Shape>,
        mut tr: Option<Shape>,
        mut r: Option<Shape>,
        mut br: Option<Shape>,
        mut b: Option<Shape>,
        mut c: Option<Shape>,
    ) -> Self {
        if let Some(s) = &mut bl {
            s.invalidate();
        }
        if let Some(s) = &mut l {
            s.invalidate();
        }
        if let Some(s) = &mut tl {
            s.invalidate();
        }
        if let Some(s) = &mut t {
            s.invalidate();
        }
        if let Some(s) = &mut tr {
            s.invalidate();
        }
        if let Some(s) = &mut r {
            s.invalidate();
        }
        if let Some(s) = &mut br {
            s.invalidate();
        }
        if let Some(s) = &mut b {
            s.invalidate();
        }
        if let Some(s) = &mut c {
            s.invalidate();
        }

        Self {
            edges: [l, t, r, b],
            corners: [bl, tl, tr, br],
            center: c,
        }
    }

    pub fn from_arr(parts: [Option<Shape>; 9]) -> Self {
        let mut ii = parts.into_iter();
        Self::new(
            ii.next().unwrap(),
            ii.next().unwrap(),
            ii.next().unwrap(),
            ii.next().unwrap(),
            ii.next().unwrap(),
            ii.next().unwrap(),
            ii.next().unwrap(),
            ii.next().unwrap(),
            ii.next().unwrap(),
        )
    }

    pub fn draw(
        &self,
        ctx: &mut impl RenderContext,
        rect: &Rect,
        fill: AdaptiveFill,
        context: &UiContext,
        crop_area: &SimpleRect,
    ) {
        let controller = ctx.controller();
        let bl = &self.corners[0];
        let tl = &self.corners[1];
        let tr = &self.corners[2];
        let br = &self.corners[3];
        let l = &self.edges[0];
        let t = &self.edges[1];
        let r = &self.edges[2];
        let b = &self.edges[3];

        let mut left_rem = rect.height();
        let mut top_rem = rect.width();
        let mut right_rem = rect.height();
        let mut bottom_rem = rect.width();
        let mut bottom_x = 0;
        let mut left_y = 0;
        let mut top_x = 0;
        let mut right_y = 0;

        if let Some(bl) = bl {
            left_rem -= bl.extent.1;
            bottom_rem -= bl.extent.0;
            left_y = bl.extent.1;
            bottom_x = bl.extent.0;
            Self::draw_shape(
                bl,
                controller,
                (rect.x(), rect.y()),
                &fill,
                rect,
                context,
                crop_area,
            );
        }
        if let Some(br) = br {
            right_rem -= br.extent.1;
            bottom_rem -= br.extent.0;
            right_y = br.extent.1;
            Self::draw_shape(
                br,
                controller,
                (rect.x() + rect.width() - br.extent.0, rect.y()),
                &fill,
                rect,
                context,
                crop_area,
            );
        }
        if let Some(tl) = tl {
            left_rem -= tl.extent.1;
            top_rem -= tl.extent.0;
            top_x = tl.extent.0;
            Self::draw_shape(
                tl,
                controller,
                (rect.x(), rect.y() + rect.height() - tl.extent.1),
                &fill,
                rect,
                context,
                crop_area,
            );
        }
        if let Some(tr) = tr {
            top_rem -= tr.extent.0;
            right_rem -= tr.extent.1;
            Self::draw_shape(
                tr,
                controller,
                (
                    rect.x() + rect.width() - tr.extent.0,
                    rect.y() + rect.height() - tr.extent.1,
                ),
                &fill,
                rect,
                context,
                crop_area,
            );
        }

        if let Some(l) = l {
            let h = l.extent.1;
            let y_scale = left_rem as f32 / h as f32;
            Self::draw_shape_scale(
                l,
                controller,
                (rect.x(), rect.y() + left_y),
                (1.0, y_scale),
                &fill,
                rect,
                context,
                crop_area,
            );
        }
        if let Some(t) = t {
            let w = t.extent.0;
            let x_scale = top_rem as f32 / w as f32;
            Self::draw_shape_scale(
                t,
                controller,
                (rect.x() + top_x, rect.y() + rect.height() - t.extent.1),
                (x_scale, 1.0),
                &fill,
                rect,
                context,
                crop_area,
            );
        }
        if let Some(r) = r {
            let h = r.extent.1;
            let y_scale = right_rem as f32 / h as f32;
            Self::draw_shape_scale(
                r,
                controller,
                (rect.x() + rect.width() - r.extent.0, rect.y() + right_y),
                (1.0, y_scale),
                &fill,
                rect,
                context,
                crop_area,
            );
        }
        if let Some(b) = b {
            let w = b.extent.0;
            let x_scale = bottom_rem as f32 / w as f32;
            Self::draw_shape_scale(
                b,
                controller,
                (rect.x() + bottom_x, rect.y()),
                (x_scale, 1.0),
                &fill,
                rect,
                context,
                crop_area,
            );
        }
        if let Some(center) = &self.center {
            let x = rect.x() + bottom_x;
            let y = rect.y() + left_y;
            let w = bottom_rem;
            let h = left_rem;
            let x_scale = w as f32 / center.extent.0 as f32;
            let y_scale = h as f32 / center.extent.1 as f32;
            Self::draw_shape_scale(
                center,
                controller,
                (x, y),
                (x_scale, y_scale),
                &fill,
                rect,
                context,
                crop_area,
            );
        }
    }

    fn draw_shape(
        shape: &Shape,
        ctx: &mut RenderController,
        pos: (i32, i32),
        fill: &AdaptiveFill,
        rect: &Rect,
        context: &UiContext,
        crop_area: &SimpleRect,
    ) {
        let pt_fn = |point: &mut InputVertex| {
            point.pos.0 += pos.0 as f32;
            point.pos.1 += pos.1 as f32;

            point.pos.0 = point
                .pos
                .0
                .p_clamp(crop_area.x as f32, (crop_area.x + crop_area.width) as f32);
            point.pos.1 = point
                .pos
                .1
                .p_clamp(crop_area.y as f32, (crop_area.y + crop_area.height) as f32);
            
            point.transform.origin.x = rect.origin().0 as f32;
            point.transform.origin.y = rect.origin().1 as f32;
            point.transform.rotation = rect.rotation().to_radians();
            point.pos.2 = 90.0;
            match fill {
                AdaptiveFill::Color(color) => {
                    point.color = color.as_vec4();
                    point.has_texture = 0.0;
                }
                AdaptiveFill::Drawable(draw) => {
                    if let Drawable::Color(c) = draw {
                        if let Some(color) = context.resources.resolve_color(*c) {
                            point.color = color.as_vec4();
                            point.has_texture = 0.0;
                        }
                    } else {
                        if let Some((tex, uv)) = draw.get_texture(context.resources) {
                            point.has_texture = 1.0;
                            point.texture = tex.id;
                            point.color = RgbColor::transparent().as_vec4();
                            point.uv = (
                                uv.x + uv.z * (point.pos.0 - rect.x() as f32) / rect.width() as f32,
                                uv.y + uv.w * (point.pos.1 - rect.y() as f32)
                                    / rect.height() as f32,
                            )
                        }
                    }
                }
            }
        };
        
        if shape.is_quad { 
            if shape.triangles.len() >= 2 {
                let mut p1 = shape.triangles[0].points[0].clone();
                let mut p2 = shape.triangles[0].points[1].clone();
                let mut p3 = shape.triangles[0].points[2].clone();
                let mut p4 = shape.triangles[1].points[2].clone();
                
                pt_fn(&mut p1);
                pt_fn(&mut p2);
                pt_fn(&mut p3);
                pt_fn(&mut p4);
                
                ctx.push_quad(Quad { points: [p1, p2, p3, p4] });
            }
        } else {
            for triangle in &shape.triangles {
                let mut triangle = triangle.clone();
                for point in &mut triangle.points {                    
                    pt_fn(point);
                }
                ctx.push_triangle(triangle);
            }
        }
    }

    fn draw_shape_scale(
        shape: &Shape,
        ctx: &mut RenderController,
        pos: (i32, i32),
        scale: (f32, f32),
        fill: &AdaptiveFill,
        rect: &Rect,
        context: &UiContext,
        crop_area: &SimpleRect,
    ) {
        let pt_fn = |point: &mut InputVertex| {
            point.pos.0 += pos.0 as f32;
            point.pos.1 += pos.1 as f32;

            point.pos.0 = point
                .pos
                .0
                .p_clamp(crop_area.x as f32, (crop_area.x + crop_area.width) as f32);
            point.pos.1 = point
                .pos
                .1
                .p_clamp(crop_area.y as f32, (crop_area.y + crop_area.height) as f32);
            
            point.transform.origin.x = rect.origin().0 as f32;
            point.transform.origin.y = rect.origin().1 as f32;
            point.transform.rotation = rect.rotation().to_radians();
            point.pos.2 = 90.0;
            match fill {
                AdaptiveFill::Color(color) => {
                    point.color = color.as_vec4();
                    point.has_texture = 0.0;
                }
                AdaptiveFill::Drawable(draw) => {
                    if let Drawable::Color(c) = draw {
                        if let Some(color) = context.resources.resolve_color(*c) {
                            point.color = color.as_vec4();
                            point.has_texture = 0.0;
                        }
                    } else {
                        if let Some((tex, uv)) = draw.get_texture(context.resources) {
                            point.has_texture = 1.0;
                            point.texture = tex.id;
                            point.color = RgbColor::transparent().as_vec4();
                            point.uv = (
                                uv.x + uv.z * (point.pos.0 - rect.x() as f32) / rect.width() as f32,
                                uv.y + uv.w * (point.pos.1 - rect.y() as f32)
                                    / rect.height() as f32,
                            )
                        }
                    }
                }
            }
        };
        
        if shape.is_quad {
            if shape.triangles.len() >= 2 {
                let mut p1 = shape.triangles[0].points[0].clone();
                let mut p2 = shape.triangles[0].points[1].clone();
                let mut p3 = shape.triangles[0].points[2].clone();
                let mut p4 = shape.triangles[1].points[2].clone();

                p1.pos.0 *= scale.0;
                p1.pos.1 *= scale.1;
                p2.pos.0 *= scale.0;
                p2.pos.1 *= scale.1;
                p3.pos.0 *= scale.0;
                p3.pos.1 *= scale.1;
                p4.pos.0 *= scale.0;
                p4.pos.1 *= scale.1;
                
                pt_fn(&mut p1);
                pt_fn(&mut p2);
                pt_fn(&mut p3);
                pt_fn(&mut p4);

                ctx.push_quad(Quad { points: [p1, p2, p3, p4] });
            }
        } else {
            for triangle in &shape.triangles {
                let mut triangle = triangle.clone();
                for point in &mut triangle.points {
                    point.pos.0 *= scale.0;
                    point.pos.1 *= scale.1;
                    
                    pt_fn(point);
                }
                ctx.push_triangle(triangle);
            }
        }
    }
}

pub enum AdaptiveFill {
    Color(RgbColor),
    Drawable(Drawable),
}
