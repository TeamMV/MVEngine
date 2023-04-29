use mvutils::utils::RcMut;
use crate::gui::components::GuiElement::{Nothing, Paragraph};
use crate::gui::gui_formats::FormattedString;
use crate::render::draw::Draw2D;
use crate::gui::styles::{BorderStyle, GuiStyle, GuiValueComputeSupply};
use crate::gui::styles::BorderStyle::{Round, Triangle};
use crate::resolve;

pub struct GuiElementInfo {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub bounding_width: i32,
    pub bounding_height: i32,
    content_width: i32,
    content_height: i32,
    pub rotation: f32,
    pub rotation_center: (i32, i32),
    pub z_index: u32,

    pub parent: Option<RcMut<GuiElement>>,

    pub handles: Vec<GuiEvent>,

    pub style: GuiStyle,

    compute_supply: GuiValueComputeSupply
}

impl Default for GuiElementInfo {
    fn default() -> Self {
        GuiElementInfo {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            bounding_width: 0,
            bounding_height: 0,
            content_width: 0,
            content_height: 0,
            rotation: 0.0,
            rotation_center: (0, 0),
            z_index: 0,
            parent: None,
            handles: vec![],
            style: GuiStyle::default(),
            compute_supply: GuiValueComputeSupply::new(1.0, None),
        }
    }
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
    pub fn parent(&self) -> Option<RcMut<GuiElement>> {
        self.parent.clone()
    }
    pub fn handles(&self) -> &Vec<GuiEvent> {
        &self.handles
    }
    pub fn style(&self) -> &GuiStyle {
        &self.style
    }

    pub(crate) fn recalculate_bounds(&mut self, ctx: RcMut<Draw2D>) {
        self.compute_supply.dpi = ctx.borrow_mut().dpi();
        self.compute_supply.parent = self.parent.clone();

        let mut paddings: [i32; 4] = [0; 4];
        paddings[0] = resolve!(self, padding_left);
        paddings[1] = resolve!(self, padding_right);
        paddings[2] = resolve!(self, padding_bottom);
        paddings[3] = resolve!(self, padding_top);

        let mut margins: [i32; 4] = [0; 4];
        margins[0] = resolve!(self, margin_left);
        margins[1] = resolve!(self, margin_right);
        margins[2] = resolve!(self, margin_bottom);
        margins[3] = resolve!(self, margin_top);

        self.bounding_width = self.content_width + paddings[0] + paddings[1];
        self.bounding_height = self.content_height + paddings[2] + paddings[3];
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

//All types here...
pub enum GuiElement {
    Nothing(GuiEmpty),
    Paragraph(GuiMarkdown)
}

impl Default for GuiElement {
    fn default() -> Self {
        Nothing(GuiEmpty {})
    }
}

macro_rules! ge_fn {
    ($s:expr, $name:ident) => {
        return match $s {
            GuiElement::Nothing(e) => {e.$name()}
            GuiElement::Paragraph(e) => {e.$name()}
            _ => {unreachable!()}
        }
    };

    ($s:expr, $name:ident, $($param:ident),*) => {
        return match $s {
            GuiElement::Nothing(e) => {e.$name($($param,)*)}
            GuiElement::Paragraph(e) => {e.$name($($param,)*)}
            _ => {unreachable!()}
        }
    };
}

impl GuiElement {
    pub fn info(&self) -> &GuiElementInfo {
        ge_fn!(self, info)
    }

    pub fn info_mut(&mut self) -> &mut GuiElementInfo {
        ge_fn!(self, info_mut)
    }

    pub fn draw(&mut self, ctx: RcMut<Draw2D>) {
        ge_fn!(self, draw, ctx)
    }

    pub fn handle(&mut self, event: GuiEvent) {
        ge_fn!(self, handle, event)
    }
}

pub trait GuiElementAbs {
    fn info(&self) -> &GuiElementInfo;
    fn info_mut(&mut self) -> &mut GuiElementInfo;
    fn draw(&mut self, ctx: RcMut<Draw2D>);
    fn handle(&mut self, event: GuiEvent) {
        self.info_mut().handles.push(event);
    }
}

pub struct GuiEmpty;
impl GuiElementAbs for GuiEmpty {
    fn info(&self) -> &GuiElementInfo {
        todo!()
    }

    fn info_mut(&mut self) -> &mut GuiElementInfo {
        todo!()
    }

    fn draw(&mut self, ctx: RcMut<Draw2D>) {
        todo!()
    }
}

pub(crate) fn draw_component_body(ctx: RcMut<Draw2D>, info: &GuiElementInfo) {

    let br = resolve!(info, border_radius);
    let bs: BorderStyle = resolve!(info, border_style);

    if bs == Round {
        ctx.borrow_mut().get_mut_gradient().copy_of(resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.borrow_mut().rounded_rectangle_origin_rotated(info.x, info.y, info.bounding_width, info.bounding_height, br, br as f32, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.borrow_mut().get_mut_gradient().copy_of(resolve!(info, border_color));
            ctx.borrow_mut().void_rounded_rectangle_origin_rotated(info.x - bw, info.y - bw, info.bounding_width + 2 * bw, info.bounding_height + 2 * bw, bw, br + bw, (br + bw) as f32, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    } else if bs == Triangle {
        ctx.borrow_mut().get_mut_gradient().copy_of(resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.borrow_mut().triangular_rectangle_origin_rotated(info.x, info.y, info.bounding_width, info.bounding_height, br, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.borrow_mut().get_mut_gradient().copy_of(resolve!(info, border_color));
            ctx.borrow_mut().void_triangular_rectangle_origin_rotated(info.x - bw, info.y - bw, info.bounding_width + 2 * bw, info.bounding_height + 2 * bw, bw, br + bw, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    } else {
        ctx.borrow_mut().get_mut_gradient().copy_of(resolve!(info, background_color));;
        let bw: i32 = resolve!(info, border_width);
        ctx.borrow_mut().rectangle_origin_rotated(info.x, info.y, info.bounding_width, info.bounding_height, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.borrow_mut().get_mut_gradient().copy_of(resolve!(info, border_color));
            ctx.borrow_mut().void_rectangle_origin_rotated(info.x - bw, info.y - bw, info.bounding_width + 2 * bw, info.bounding_height + 2 * bw, bw, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    }
}

macro_rules! center {
    ($total:ident, $value:ident) => {$total as f32 / 2.0 - $value as f32 / 2.0};
    ($total:ident, $value:ident, $res_type:ty) => {($total as f32 / 2.0 - $value as f32 / 2.0) as $res_type};
}

//Specific abstraction

pub trait GuiTextComponent {
    fn get_text(&self) -> &FormattedString;
    fn set_text(&mut self, text: FormattedString);
}

//Implementation (real component shit)

//------------------------------------
//GuiParagraph
//------------------------------------
pub struct GuiMarkdown {
    info: GuiElementInfo,
    text: FormattedString,
}

impl GuiElementAbs for GuiMarkdown {
    fn info(&self) -> &GuiElementInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut GuiElementInfo {
        &mut self.info
    }

    fn draw(&mut self, ctx: RcMut<Draw2D>) {
        self.info_mut().content_height = resolve!(self.info, text_size);
        self.info_mut().content_width = resolve!(self.info, font).unwrap().regular.get_metrics(self.text.whole.as_str()).width(self.info.content_height);
        self.info_mut().recalculate_bounds(ctx.clone());

        draw_component_body(ctx.clone(), self.info());
        let left = resolve!(self.info, padding_left);
        let bottom = resolve!(self.info, padding_bottom);

        ctx.borrow_mut().chroma_tilt(resolve!(self.info, text_chroma_tilt));
        ctx.borrow_mut().chroma_compress(resolve!(self.info, text_chroma_compress));
        self.text.draw(ctx.clone(), self.info.x + left, self.info.y + bottom, self.info.content_height, resolve!(self.info, font), self.info.rotation, self.info.rotation_center.0, self.info.rotation_center.1, resolve!(self.info, text_color), resolve!(self.info, text_chroma));
    }
}

impl GuiTextComponent for GuiMarkdown {
    fn get_text(&self) -> &FormattedString {
        &self.text
    }

    fn set_text(&mut self, text: FormattedString) {
        self.text = text;
    }
}

impl GuiMarkdown {
    pub fn create() -> Self {
        GuiMarkdown {
            info: GuiElementInfo::default(),
            text: FormattedString { pieces: vec![], whole: "".to_string() },
        }
    }
}