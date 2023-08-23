pub mod input_box;
pub mod button;
pub mod text;
pub mod layout;

use std::ops::Range;
use std::sync::{Arc, Mutex, RwLock};

use itertools::Itertools;
use mvutils::{fn_for, sealable};
use mvutils::utils::{Recover, TetrahedronOp};

use crate::gui::components::GuiElement::{Layout, Void};
use crate::gui::components::layout::GuiSection;
use crate::gui::components::text::GuiLabel;
use crate::gui::gui_formats::FormattedString;
use crate::gui::styles::BorderStyle::{Round, Triangle};
use crate::gui::styles::{BorderStyle, Direction, GuiStyle, GuiValueComputeSupply, HorizontalAlign, Overflow, Positioning, RotationCenter, Size, VerticalAlign, ViewState};
use crate::render::draw2d::Draw2D;
use crate::render::text::Font;
use crate::resolve;

pub struct GuiElementInfo {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub bounding_width: i32,
    pub bounding_height: i32,
    pub content_width: i32,
    pub content_height: i32,
    pub rotation: f32,
    pub rotation_center: (i32, i32),
    pub z_index: u32,

    pub parent: Option<Arc<GuiElement>>,

    pub handles: Vec<GuiEvent>,

    pub id: String,
    pub tags: Vec<String>,

    pub style: GuiStyle,

    pub compute_supply: GuiValueComputeSupply,
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
            id: "We're no strangers to love You know the rules and so do I (do I) A full commitment's what I'm thinking of You wouldn't get this from any other guy I just wanna tell you how I'm feeling Gotta make you understand Never gonna give you up Never gonna let you down Never gonna run around and desert you Never gonna make you cry Never gonna say goodbye Never gonna tell a lie and hurt you We've known each other for so long Your heart's been aching, but you're too shy to say it (say it) Inside, we both know what's been going on (going on) We know the game and we're gonna play it And if you ask me how I'm feeling Don't tell me you're too blind to see Never gonna give you up Never gonna let you down Never gonna run around and desert you Never gonna make you cry Never gonna say goodbye Never gonna tell a lie and hurt you Never gonna give you up Never gonna let you down Never gonna run around and desert you Never gonna make you cry Never gonna say goodbye Never gonna tell a lie and hurt you We've known each other for so long Your heart's been aching, but you're too shy to say it (to say it) Inside, we both know what's been going on (going on) We know the game and we're gonna play it I just wanna tell you how I'm feeling Gotta make you understand Never gonna give you up Never gonna let you down Never gonna run around and desert you Never gonna make you cry Never gonna say goodbye Never gonna tell a lie and hurt you Never gonna give you up Never gonna let you down Never gonna run around and desert you Never gonna make you cry Never gonna say goodbye Never gonna tell a lie and hurt you Never gonna give you up Never gonna let you down Never gonna run around and desert you Never gonna make you cry Never gonna say goodbye Never gonna tell a lie and hurt you".to_string(),
            tags: vec![],
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
    pub fn parent(&self) -> Option<Arc<GuiElement>> {
        self.parent.clone()
    }
    pub fn handles(&self) -> &Vec<GuiEvent> {
        &self.handles
    }
    pub fn style(&self) -> &GuiStyle {
        &self.style
    }

    pub(crate) fn recalculate_bounds(&mut self, ctx: &Draw2D) {
        self.compute_supply.dpi = ctx.dpi();
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

        let pos = resolve!(self, position);
        if pos == Positioning::Absolute {
            let x = resolve!(self, x);
            let y = resolve!(self, y);
            let origin = resolve!(self, origin);
            self.x = origin.is_right().yn(x - self.width, x);
            self.y = origin.is_top().yn(y - self.height, y);
        }

        self.rotation = resolve!(self, rotation);
        let rotation_center = resolve!(self, rotation_center);
        if let RotationCenter::Custom(center) = rotation_center {
            self.rotation_center = center;
        } else {
            self.rotation_center = (self.x + (self.width as f32 / 2.0) as i32, self.y + (self.height as f32 / 2.0) as i32);
        }
    }
}

pub type ClickFn = Box<dyn FnMut(i32, i32, u8)>;
pub type MouseFn = Box<dyn FnMut(i32, i32)>;

pub enum GuiEvent {
    OnClick(ClickFn),
    OnRelease(ClickFn),
    OnMouse(MouseFn),
    OnMouseLeave(MouseFn),
}

//All types here...
pub enum GuiElement {
    Void(GuiVoid),
    Layout(GuiLayout),
    Text(GuiTextElement),
    Button(GuiButtonElement),
    InputBox(GuiInputBoxElement)
}

pub enum GuiTextElement {
    Label(GuiLabel),
    Multiline(),
}

pub enum GuiButtonElement {
    Button(),
    Checkbox()
}

pub enum GuiInputBoxElement {
    Line(),
    Box(),
}

impl Default for GuiElement {
    fn default() -> Self {
        Void(GuiVoid {})
    }
}

macro_rules! ge_fn {
    ($s:expr, $name:ident) => {
        return match $s {
            GuiElement::Void(e) => {e.$name()}
            GuiElement::Layout(e) => {e.$name()}
            GuiElement::Text(e) => {e.$name()}
            _ => {unreachable!()}
        }
    };

    ($s:expr, $name:ident, $($param:ident),*) => {
        return match $s {
            GuiElement::Void(e) => {e.$name($($param,)*)}
            GuiElement::Layout(e) => {e.$name($($param,)*)}
            GuiElement::Text(e) => {e.$name($($param,)*)}
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

    pub fn draw(&mut self, ctx: &mut Draw2D) {
        ge_fn!(self, draw, ctx)
    }

    pub fn handle(&mut self, event: GuiEvent) {
        ge_fn!(self, handle, event)
    }

    pub fn layout(&self) -> &GuiLayout {
        if let Layout(l) = self {
            return l;
        }
        panic!("Cannot retrieve Layout from non-Layout element!")
    }
}

macro_rules! gte_fn {
    ($s:expr, $name:ident) => {
        return match $s {
            GuiTextElement::Label(e) => {e.$name()}
            //GuiTextElement::Multiline(e) => {e.$name()}
            _ => {unreachable!()}
        }
    };

    ($s:expr, $name:ident, $($param:ident),*) => {
        return match $s {
            GuiTextElement::Label(e) => {e.$name($($param,)*)}
            //GuiTextElement::Multiline(e) => {e.$name($($param,)*)}
            _ => {unreachable!()}
        }
    };
}

impl GuiTextElement {
    pub fn info(&self) -> &GuiElementInfo {
        gte_fn!(self, info)
    }

    pub fn info_mut(&mut self) -> &mut GuiElementInfo {
        gte_fn!(self, info_mut)
    }

    pub fn draw(&mut self, ctx: &mut Draw2D) {
        gte_fn!(self, draw, ctx)
    }

    pub fn handle(&mut self, event: GuiEvent) {
        gte_fn!(self, handle, event)
    }
}

//GuiLayout

pub struct GuiElements {
    elements: Vec<Arc<RwLock<GuiElement>>>,
}

impl GuiElements {
    pub(crate) fn new() -> Self {
        Self { elements: vec![] }
    }

    pub fn add_element(&mut self, element: GuiElement) {
        self.elements.push(Arc::new(RwLock::new(element)))
    }

    pub fn add_elements(&mut self, elements: Vec<GuiElement>) {
        self.elements.extend(elements.into_iter().map(|elem| Arc::new(RwLock::new(elem))));
    }

    pub fn get_element_by_index(&mut self, idx: usize) -> Arc<RwLock<GuiElement>> {
        self.elements[idx].clone()
    }

    //pub fn get_elements_by_range(&self, range: Range<usize>) -> &[&'static mut GuiElement] {
    //    self.elements[range]
    //}

    pub fn get_element_by_id(&self, id: &String) -> Option<Arc<RwLock<GuiElement>>> {
        for element in self.elements.iter() {
            if &element.read().recover().info().id == id {
                return Some(element.clone());
            }
        }
        None
    }

    //pub fn get_elements_by_tag(
    //    &self,
    //    tag: &String,
    //) -> impl Iterator<Item = &'static mut GuiElement> {
    //    self.elements
    //        .iter()
    //        .filter(|e| e.info().tags.contains(&tag.to_string()))
    //}

    pub fn get_elements(&self) -> &[Arc<RwLock<GuiElement>>] {
        &self.elements
    }

    pub fn remove_element_by_id(&mut self, id: &String) {
        let mut indices: Vec<usize> = vec![];
        for (i, element) in self.elements.iter().enumerate() {
            if &element.read().recover().info().id == id {
                indices.push(i);
            }
        }

        for index in indices {
            self.elements.remove(index);
        }
    }

    pub fn remove_elements_by_ids(&mut self, ids: Vec<&String>) {
        for id in ids {
            self.remove_element_by_id(id);
        }
    }

    pub fn remove_elements_by_tag(&mut self, tag: &String) {
        let mut indices: Vec<usize> = vec![];
        for (i, element) in self.elements.iter().enumerate() {
            if element.read().recover().info().tags.contains(tag) {
                indices.push(i);
            }
        }

        for index in indices {
            self.elements.remove(index);
        }
    }

    pub fn remove_elements_by_tags(&mut self, tags: Vec<&String>) {
        for tag in tags {
            self.remove_elements_by_tag(tag);
        }
    }

    pub fn remove_element_by_index(&mut self, idx: usize) {
        self.elements.remove(idx);
    }

    pub fn remove_elements_by_range(&mut self, range: Range<usize>) {
        for i in range {
            self.elements.remove(i);
        }
    }

    pub fn remove_all_elements(&mut self) {
        self.elements.clear()
    }

    pub fn count_elements(&self) -> usize {
        self.elements.len()
    }
}

pub enum GuiLayout {
    Void(GuiVoid),
    Section(GuiSection),
}

macro_rules! gl_fn {
    ($s:expr, $name:ident) => {
        return match $s {
            GuiLayout::Void(e) => {e.$name()}
            GuiLayout::Section(e) => {e.$name()}
        }
    };

    ($s:expr, $name:ident, $($param:ident),*) => {
        return match $s {
            GuiLayout::Void(e) => {e.$name($($param,)*)}
            GuiLayout::Section(e) => {e.$name($($param,)*)}
        }
    };
}

impl GuiLayout {
    pub fn info(&self) -> &GuiElementInfo {
        gl_fn!(self, info)
    }

    pub fn info_mut(&mut self) -> &mut GuiElementInfo {
        gl_fn!(self, info_mut)
    }

    pub fn draw(&mut self, ctx: &mut Draw2D) {
        gl_fn!(self, draw, ctx)
    }

    pub fn handle(&mut self, event: GuiEvent) {
        gl_fn!(self, handle, event)
    }

    pub fn elements(&self) -> &GuiElements {
        gl_fn!(self, elements);
    }

    pub fn elements_mut(&mut self) -> &mut GuiElements {
        gl_fn!(self, elements_mut);
    }
}

pub trait GuiComponent {
    fn create() -> Self;
    fn info(&self) -> &GuiElementInfo;
    fn info_mut(&mut self) -> &mut GuiElementInfo;
    fn draw(&mut self, ctx: &mut Draw2D);
    fn handle(&mut self, event: GuiEvent) {
        self.info_mut().handles.push(event);
    }
}

pub trait GuiLayoutComponent {
    fn elements(&self) -> &GuiElements;
    fn elements_mut(&mut self) -> &mut GuiElements;
}

pub struct GuiVoid;

impl GuiComponent for GuiVoid {
    fn create() -> Self {
        Self
    }

    fn info(&self) -> &GuiElementInfo {
        todo!()
    }

    fn info_mut(&mut self) -> &mut GuiElementInfo {
        todo!()
    }

    fn draw(&mut self, ctx: &mut Draw2D) {
        todo!()
    }
}

impl GuiLayoutComponent for GuiVoid {
    fn elements(&self) -> &GuiElements {
        todo!()
    }

    fn elements_mut(&mut self) -> &mut GuiElements {
        todo!()
    }
}

pub(crate) fn draw_component_body(ctx: &mut Draw2D, info: &GuiElementInfo) {
    let br = resolve!(info, border_radius);
    let bs: BorderStyle = resolve!(info, border_style);

    if bs == Round {
        ctx.get_mut_gradient()
            .copy_of(&resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.rounded_rectangle_origin_rotated(
            info.x,
            info.y,
            info.bounding_width,
            info.bounding_height,
            br,
            br as f32,
            info.rotation,
            info.rotation_center.0,
            info.rotation_center.1,
        );
        if br > 0 {
            ctx.get_mut_gradient()
                .copy_of(&resolve!(info, border_color));
            ctx.void_rounded_rectangle_origin_rotated(
                    info.x - bw,
                    info.y - bw,
                    info.bounding_width + 2 * bw,
                    info.bounding_height + 2 * bw,
                    bw,
                    br + bw * 2,
                    (br + bw) as f32,
                    info.rotation,
                    info.rotation_center.0,
                    info.rotation_center.1,
                );
        }
    } else if bs == Triangle {
        ctx.get_mut_gradient()
            .copy_of(&resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.triangular_rectangle_origin_rotated(
            info.x,
            info.y,
            info.bounding_width,
            info.bounding_height,
            br,
            info.rotation,
            info.rotation_center.0,
            info.rotation_center.1,
        );
        if br > 0 {
            ctx.get_mut_gradient()
                .copy_of(&resolve!(info, border_color));
            ctx.void_triangular_rectangle_origin_rotated(
                    info.x - bw,
                    info.y - bw,
                    info.bounding_width + 2 * bw,
                    info.bounding_height + 2 * bw,
                    bw,
                    br + bw * 2,
                    info.rotation,
                    info.rotation_center.0,
                    info.rotation_center.1,
                );
        }
    } else {
        ctx.get_mut_gradient()
            .copy_of(&resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.rectangle_origin_rotated(
            info.x,
            info.y,
            info.bounding_width,
            info.bounding_height,
            info.rotation,
            info.rotation_center.0,
            info.rotation_center.1,
        );
        if br > 0 {
            ctx.get_mut_gradient()
                .copy_of(&resolve!(info, border_color));
            ctx.void_rectangle_origin_rotated(
                info.x - bw,
                info.y - bw,
                info.bounding_width + 2 * bw,
                info.bounding_height + 2 * bw,
                bw,
                info.rotation,
                info.rotation_center.0,
                info.rotation_center.1,
            );
        }
    }
}

macro_rules! center {
    ($total:ident, $value:ident) => {
        $total as f32 / 2.0 - $value as f32 / 2.0
    };
    ($total:ident, $value:ident, $res_type:ty) => {
        ($total as f32 / 2.0 - $value as f32 / 2.0) as $res_type
    };
}

//Specific abstraction

pub trait GuiTextComponent {
    fn get_text(&self) -> &String;
    fn set_text(&mut self, text: String);
    fn draw_line(&mut self, ctx: &mut Draw2D, text: String, font: Arc<Font>);
}

