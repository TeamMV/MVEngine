use crate::color::RgbColor;
use crate::graphics::Drawable;
use crate::rendering::RenderContext;
use crate::ui::context::UiContext;
use crate::ui::geometry::shape::Shape;
use crate::ui::geometry::{shape, SimpleRect};
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
            s.recompute();
        }
        if let Some(s) = &mut l {
            s.recompute();
        }
        if let Some(s) = &mut tl {
            s.recompute();
        }
        if let Some(s) = &mut t {
            s.recompute();
        }
        if let Some(s) = &mut tr {
            s.recompute();
        }
        if let Some(s) = &mut r {
            s.recompute();
        }
        if let Some(s) = &mut br {
            s.recompute();
        }
        if let Some(s) = &mut b {
            s.recompute();
        }
        if let Some(s) = &mut c {
            s.recompute();
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
        rect: &SimpleRect,
        fill: AdaptiveFill,
        context: &UiContext,
    ) {
        let bl = &self.corners[0];
        let tl = &self.corners[1];
        let tr = &self.corners[2];
        let br = &self.corners[3];

        let l = &self.edges[0];
        let t = &self.edges[1];
        let r = &self.edges[2];
        let b = &self.edges[3];

        let x = rect.x;
        let y = rect.y;
        let w = rect.width;
        let h = rect.height;
        
        let (tlw, tlh) = tl.as_ref().map(|s| (s.extent.width, s.extent.height)).unwrap_or((0, 0));
        let (trw, trh) = tr.as_ref().map(|s| (s.extent.width, s.extent.height)).unwrap_or((0, 0));
        let (blw, blh) = bl.as_ref().map(|s| (s.extent.width, s.extent.height)).unwrap_or((0, 0));
        let (brw, brh) = br.as_ref().map(|s| (s.extent.width, s.extent.height)).unwrap_or((0, 0));
        
        if let Some(shape) = tl {
            let r = SimpleRect { x, y, width: tlw, height: tlh };
            Self::draw_shape(shape, ctx, &r, fill.clone(), context);
        }
        if let Some(shape) = tr {
            let r = SimpleRect { x: x + w - trw, y, width: trw, height: trh };
            Self::draw_shape(shape, ctx, &r, fill.clone(), context);
        }
        if let Some(shape) = bl {
            let r = SimpleRect { x, y: y + h - blh, width: blw, height: blh };
            Self::draw_shape(shape, ctx, &r, fill.clone(), context);
        }
        if let Some(shape) = br {
            let r = SimpleRect { x: x + w - brw, y: y + h - brh, width: brw, height: brh };
            Self::draw_shape(shape, ctx, &r, fill.clone(), context);
        }
        
        if let Some(shape) = t {
            let edge_h = shape.extent.height;
            let r = SimpleRect {
                x: x + tlw,
                y,
                width: w - tlw - trw,
                height: edge_h,
            };
            Self::draw_shape(shape, ctx, &r, fill.clone(), context);
        }
        if let Some(shape) = b {
            let edge_h = shape.extent.height;
            let r = SimpleRect {
                x: x + blw,
                y: y + h - edge_h,
                width: w - blw - brw,
                height: edge_h,
            };
            Self::draw_shape(shape, ctx, &r, fill.clone(), context);
        }
        if let Some(shape) = l {
            let edge_w = shape.extent.width;
            let r = SimpleRect {
                x,
                y: y + tlh,
                width: edge_w,
                height: h - tlh - blh,
            };
            Self::draw_shape(shape, ctx, &r, fill.clone(), context);
        }
        if let Some(shape) = r {
            let edge_w = shape.extent.width;
            let r = SimpleRect {
                x: x + w - edge_w,
                y: y + trh,
                width: edge_w,
                height: h - trh - brh,
            };
            Self::draw_shape(shape, ctx, &r, fill.clone(), context);
        }
        
        if let Some(shape) = &self.center {
            let r = SimpleRect {
                x: x + tlw,
                y: y + tlh,
                width: w - tlw - trw,
                height: h - tlh - blh,
            };
            Self::draw_shape(shape, ctx, &r, fill, context);
        }        
    }
    
    fn draw_shape(
        shape: &Shape,
        ctx: &mut impl RenderContext,
        rect: &SimpleRect,
        fill: AdaptiveFill,
        context: &UiContext) {
        
        match fill {
            AdaptiveFill::Color(col) => {
                shape::utils::draw_shape_color(ctx, shape, col, rect);
            }
            AdaptiveFill::Drawable(draw) => {
                let (tex, uv) = draw.get_texture_or_default(context.resources);
                shape::utils::draw_shape_textured(ctx, shape, tex, uv, rect);
            }
        }
    }
}

#[derive(Clone)]
pub enum AdaptiveFill {
    Color(RgbColor),
    Drawable(Drawable),
}
