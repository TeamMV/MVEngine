use crate::ui::elements::UiElementStub;
use crate::ui::rendering::ctx::DrawShape;

pub const EDGE_LEFT: usize = 0;
pub const EDGE_TOP: usize = 1;
pub const EDGE_RIGHT: usize = 2;
pub const EDGE_BOTTOM: usize = 3;

pub const CORNER_BL: usize = 0;
pub const CORNER_TL: usize = 1;
pub const CORNER_TR: usize = 2;
pub const CORNER_BR: usize = 3;

pub struct AdaptiveShape {
    pub edges: [Option<DrawShape>; 4], //l, t, r, b
    pub corners: [Option<DrawShape>; 4], //bl, tl, tr, br
    pub center: Option<DrawShape>
}

impl AdaptiveShape {
    pub fn new(
        bl: Option<DrawShape>,
        l: Option<DrawShape>,
        tl: Option<DrawShape>,
        t: Option<DrawShape>,
        tr: Option<DrawShape>,
        r: Option<DrawShape>,
        br: Option<DrawShape>,
        b: Option<DrawShape>,
        c: Option<DrawShape>
    ) -> Self {
        Self {
            edges: [l, t, r, b],
            corners: [bl, tl, tr, br],
            center: c
        }
    }

    pub fn from_arr(parts: [Option<DrawShape>; 9]) -> Self {
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
            ii.next().unwrap()
        )
    }
}