use std::ops::Deref;
use mvutils::screen::Measurements;
use mvutils::utils::{R, XTraFMath};
use crate::render::draw::Draw2D;
use crate::resolve;
use crate::vgui_v5::styles::{BorderStyle, GuiStyle};
use crate::vgui_v5::styles::BorderStyle::{Round, Triangle};

pub struct GuiElementInfo {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub bounding_width: i32,
    pub bounding_height: i32,
    e_width: i32,
    e_height: i32,
    pub rotation: f32,
    pub rotation_center: (i32, i32),
    pub z_index: u32,

    pub parent: Box<dyn GuiElement>,

    pub handles: Vec<GuiEvent>,

    pub style: GuiStyle
}

impl GuiElementInfo {
    pub fn x(&self) -> i32 {
        self.x
    }
    pub fn y(&self) -> i32 {
        self.y
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn bounding_width(&self) -> i32 {
        self.bounding_width
    }
    pub fn bounding_height(&self) -> i32 {
        self.bounding_height
    }
    pub fn z_index(&self) -> u32 {
        self.z_index
    }
    pub fn parent(&self) -> &Box<dyn GuiElement> {
        &self.parent
    }
    pub fn handles(&self) -> &Vec<GuiEvent> {
        &self.handles
    }
    pub fn style(&self) -> &GuiStyle {
        &self.style
    }

    pub(crate) fn recalculate_bounds(&mut self) {
        let mut paddings: [i32; 4] = [0; 4];
        paddings[0] = resolve!(self, true, padding[0]);
        paddings[1] = resolve!(self, true, padding[1]);
        paddings[2] = resolve!(self, true, padding[2]);
        paddings[3] = resolve!(self, true, padding[3]);

        let mut margins: [i32; 4] = [0; 4];
        margins[0] = resolve!(self, true, margin[0]);
        margins[1] = resolve!(self, true, margin[1]);
        margins[2] = resolve!(self, true, margin[2]);
        margins[3] = resolve!(self, true, margin[3]);

        self.e_width = resolve!(self, true, width);
        self.e_height = resolve!(self, true, height);
        self.bounding_width = self.e_width + paddings[0] + paddings[1];
        self.bounding_height = self.e_width + paddings[2] + paddings[3];
        self.width = self.bounding_width + margins[0] + margins[1];
        self.height = self.bounding_height + margins[2] + margins[3];
    }
}

pub type ClickFn = *const dyn FnMut(i32, i32, u8);
pub type MouseFn = *const dyn FnMut(i32, i32);

pub enum GuiEvent {
    OnClick(ClickFn),
    OnRelease(ClickFn),
    OnMouse(MouseFn),
    OnMouseLeave(MouseFn),
}

pub trait GuiElement {
    fn info(&self) -> &GuiElementInfo;
    fn info_mut(&mut self) -> &mut GuiElementInfo;
    fn draw(&self, ctx: R<Draw2D>);
    fn handle(&mut self, event: GuiEvent) {
        self.info_mut().handles.push(event);
    }
}

pub(crate) fn draw_component_body(ctx: R<Draw2D>, info: &GuiElementInfo) {
    let br = info.style.border_radius.unwrapt(ctx.clone(), info, |s| {&s.border_radius});
    let bs: BorderStyle = info.style.border_style.unwrapt(ctx.clone(), info, |s| {&s.border_style});
    let mut paddings: [i32; 4] = [0; 4];
    paddings[0] = resolve!(info, true, padding[0]);
    paddings[1] = resolve!(info, true, padding[1]);
    paddings[2] = resolve!(info, true, padding[2]);
    paddings[3] = resolve!(info, true, padding[3]);
    //left right bottom top

    if bs == Round {
        ctx.borrow_mut().get_mut_gradient().copy_of(info.style.background_color.unwrapt(ctx.clone(), info, |s| {&s.background_color}));
        let bw: i32 = info.style.border_width.unwrapt(ctx.clone(), info, |s| {&s.border_width});
        ctx.borrow_mut().rounded_rectangle_origin_rotated(info.x + bw, info.y + bw, info.width - bw * 2 + paddings[1] + paddings[0], info.height - bw * 2 + paddings[3] + paddings[2], br, br as f32, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.borrow_mut().get_mut_gradient().copy_of(info.style.border_color.unwrapt(ctx.clone(), info, |s| {&s.border_color}));
            ctx.borrow_mut().void_rounded_rectangle_origin_rotated(info.x, info.y, info.width + paddings[1] + paddings[0], info.height + paddings[3] + paddings[2], bw, br, (br + bw) as f32, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    } else if bs == Triangle {
        ctx.borrow_mut().get_mut_gradient().copy_of(info.style.background_color.unwrapt(ctx.clone(), info, |s| {&s.background_color}));
        let bw: i32 = info.style.border_width.unwrapt(ctx.clone(), info, |s| {&s.border_width});
        ctx.borrow_mut().triangular_rectangle_origin_rotated(info.x + bw, info.y + bw, info.width - bw * 2 + paddings[1] + paddings[0], info.height - bw * 2 + paddings[3] + paddings[2], br, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.borrow_mut().get_mut_gradient().copy_of(info.style.border_color.unwrapt(ctx.clone(), info, |s| {&s.border_color}));
            ctx.borrow_mut().void_triangular_rectangle_origin_rotated(info.x, info.y, info.width + paddings[1] + paddings[0], info.height + paddings[3] + paddings[2], bw, br + bw, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    } else {
        ctx.borrow_mut().get_mut_gradient().copy_of(info.style.background_color.unwrapt(ctx.clone(), info, |s| {&s.background_color}));;
        let bw: i32 = info.style.border_width.unwrapt(ctx.clone(), info, |s| {&s.border_width});
        ctx.borrow_mut().rectangle_origin_rotated(info.x + bw, info.y + bw, info.width - bw * 2 + paddings[1] + paddings[0], info.height - bw * 2 + paddings[3] + paddings[2], info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.borrow_mut().get_mut_gradient().copy_of(info.style.border_color.unwrapt(ctx.clone(), info, |s| {&s.border_color}));
            ctx.borrow_mut().void_rectangle_origin_rotated(info.x, info.y, info.width + paddings[1] + paddings[0], info.height + paddings[3] + paddings[2], bw, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    }
}

macro_rules! center {
    ($total:ident, $value:ident) => {$total as f32 / 2.0 - $value as f32 / 2.0};
    ($total:ident, $value:ident, $res_type:ty) => {($total as f32 / 2.0 - $value as f32 / 2.0) as $res_type};
}

//Specific abstraction

pub trait GuiTextComponent: GuiElement {
    fn get_text(&self) -> &String;
    fn set_text(&mut self, text: String);
}

//Implementation (real component shit)

//------------------------------------
//GuiParagraph
//------------------------------------
pub struct GuiParagraph {
    info: GuiElementInfo,
    text: String,
}

impl GuiElement for GuiParagraph {
    fn info(&self) -> &GuiElementInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut GuiElementInfo {
        &mut self.info
    }

    fn draw(&self, ctx: R<Draw2D>) {
        draw_component_body(ctx, self.info());
        let left = resolve!(padding[0]);
        let bottom = resolve!(padding[2]);

        ctx.borrow_mut().get_mut_gradient().copy_of(self.info.style.text_color.unwrapt(ctx.clone(), self.info(), |s| {&s.text_color}));
        ctx.borrow_mut().text_origin_rotated(resolve!(text_chroma), center!(), self.info.y, resolve!(text_size))
    }
}

impl GuiTextComponent for GuiParagraph {
    fn get_text(&self) -> &String {
        &self.text
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}