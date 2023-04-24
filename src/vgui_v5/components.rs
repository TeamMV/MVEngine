use mvutils::screen::Measurements;
use mvutils::utils::XTraFMath;
use crate::render::draw::Draw2D;
use crate::vgui_v5::styles::{BorderStyle, GuiStyle, GuiValue};
use crate::vgui_v5::styles::BorderStyle::{Round, Triangle};

pub struct GuiElementInfo {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub rotation: f32,
    pub rotation_center: (i32, i32),
    pub z_index: u32,

    pub parent: Box<dyn GuiElement>,

    pub handles: &'static mut Vec<GuiEvent>,

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
    pub fn z_index(&self) -> u32 {
        self.z_index
    }
    pub fn parent(&self) -> &impl GuiElement {
        &self.parent.as_ref()
    }
    pub fn handles(&self) -> &'static mut Vec<GuiEvent> {
        self.handles
    }
    pub fn style(&self) -> &GuiStyle {
        &self.style
    }
}

pub type ClickFn = dyn FnMut(i32, i32, u8);
pub type MouseFn = dyn FnMut(i32, i32);

pub enum GuiEvent {
    OnClick(ClickFn),
    OnRelease(ClickFn),
    OnMouse(MouseFn),
    OnMouseLeave(MouseFn),
}

pub trait GuiElement {
    fn info(&self) -> &GuiElementInfo;
    fn info_mut(&mut self) -> &mut GuiElementInfo;
    fn draw(&self, ctx: &mut Draw2D);
    fn handle(&mut self, event: GuiEvent) {
        self.info().handles.push(event);
    }
}

macro_rules! center {
    () => {};
}

pub(crate) fn draw_component_body(ctx: &mut Draw2D, info: &GuiElementInfo) {
    let br = info.style.border_radius.unwrap(ctx, info, Box::new(|s| {&s.border_radius}));
    let bs: BorderStyle = info.style.border_style.unwrap(ctx, info, Box::new(|s| {&s.border_style}));
    let mut margins: [f32; 4] = [0.0; 4];
    margins[0] = info.style.margin[0].unwrap(ctx, info, Box::new(|s| {&s.margin[0]}));
    margins[1] = info.style.margin[1].unwrap(ctx, info, Box::new(|s| {&s.margin[1]}));
    margins[2] = info.style.margin[2].unwrap(ctx, info, Box::new(|s| {&s.margin[2]}));
    margins[3] = info.style.margin[3].unwrap(ctx, info, Box::new(|s| {&s.margin[3]}));
    if bs == Round {
        ctx.get_mut_gradient().copy_of(unwrap_value!(ctx, info.style, background_color));
        let bw: i32 = unwrap_value!(ctx, info.style, border_width);
        ctx.rounded_rectangle_origin_rotated(info.x + bw - margins[0], info.y + bw - margins[2], info.width - bw * 2 + margins[1], info.height - bw * 2 + margins[3], br, br as f32, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.get_mut_gradient().copy_of(unwrap_value!(ctx, info.style, border_color));
            ctx.void_rounded_rectangle_origin_rotated(info.x - margins[0], info.y - margins[2], info.width + margins[1], info.height + margins[3], bw, br, (br + bw) as f32, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    } else if bs == Triangle {
        ctx.get_mut_gradient().copy_of(unwrap_value!(ctx, info.style, background_color));
        let bw: i32 = unwrap_value!(ctx, info.style, border_width);
        ctx.triangular_rectangle_origin_rotated(info.x + bw - margins[0], info.y + bw - margins[2], info.width - bw * 2 + margins[1], info.height - bw * 2 + margins[3], br, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.get_mut_gradient().copy_of(unwrap_value!(ctx, info.style, border_color));
            ctx.void_triangular_rectangle_origin_rotated(info.x - margins[0], info.y - margins[2], info.width + margins[1], info.height + margins[3], bw, br + bw, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    } else {
        ctx.get_mut_gradient().copy_of(unwrap_value!(ctx, info.style, background_color));;
        let bw: i32 = unwrap_value!(ctx, info.style, border_width);
        ctx.rectangle_origin_rotated(info.x + bw - margins[0], info.y + bw - margins[2], info.width - bw * 2 + margins[1], info.height - bw * 2 + margins[3], info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.get_mut_gradient().copy_of(unwrap_value!(ctx, info.style, border_color));
            ctx.void_rectangle_origin_rotated(info.x - margins[0], info.y - margins[2], info.width + margins[1], info.height + margins[3], bw, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    }
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

    fn draw(&self, ctx: &mut Draw2D) {
        draw_component_body(ctx, self.info());
        
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