use std::ops::Range;
use std::process::id;
use std::sync::Arc;
use itertools::Itertools;
use mvutils::utils::{IncDec, TetrahedronOp};
use crate::gui::components::GuiElement::{Void, Paragraph, Layout};
use crate::gui::gui_formats::FormattedString;
use crate::gui::styles::{BorderStyle, GuiStyle, GuiValueComputeSupply, Positioning, Size, ViewState};
use crate::gui::styles::BorderStyle::{Round, Triangle};
use crate::render::draw::Draw2D;
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

    pub compute_supply: GuiValueComputeSupply
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

    pub(crate) fn recalculate_bounds(&mut self, ctx: Arc<Draw2D>) {
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
    Void(GuiVoid),
    Layout(GuiLayout),
    Paragraph(GuiMarkdown),
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
            GuiElement::Paragraph(e) => {e.$name()}
            _ => {unreachable!()}
        }
    };

    ($s:expr, $name:ident, $($param:ident),*) => {
        return match $s {
            GuiElement::Void(e) => {e.$name($($param,)*)}
            GuiElement::Layout(e) => {e.$name($($param,)*)}
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

    pub fn draw(&mut self, ctx: Arc<Draw2D>) {
        ge_fn!(self, draw, ctx)
    }

    pub fn handle(&mut self, event: GuiEvent) {
        ge_fn!(self, handle, event)
    }
}

impl GuiElement {
    pub fn layout(&self) -> &GuiLayout {
        if let Layout(l) = self {
            return l;
        }
        panic!("Cannot retrieve Layout from non-Layout element!")
    }
}

//GuiLayout

pub struct GuiElements {
    elements: Vec<&'static GuiElement>,
}

impl GuiElements {
    pub(crate) fn new() -> Self {
        Self {
            elements: vec![],
        }
    }

    pub fn add_element(&mut self, element: &GuiElement) {
        self.elements.push(element)
    }

    pub fn add_elements(&mut self, elements: Vec<&GuiElement>) {
        self.elements.extend(elements);
    }

    pub fn get_element_by_index(&self, idx: usize) -> &GuiElement {
        self.elements[idx]
    }

    pub fn get_elements_by_range(&self, range: Range<usize>) -> Vec<&GuiElement> {
        self.elements[range].to_vec()
    }

    pub fn get_element_by_id(&self, id: &String) -> Option<&GuiElement> {
        for element in self.elements {
            if &element.info().id == id {
                return Some(element);
            }
        }
        None
    }

    pub fn get_elements_by_tag(&self, tag: &String) -> Vec<&GuiElement> {
        let mut vec: Vec<&GuiElement> = vec![];
        for element in self.elements {
            if element.info().tags.contains(tag) {
                vec.push(element);
            }
        }
        vec
    }

    pub fn get_elements(&self) -> &Vec<&GuiElement> {
        &self.elements
    }

    pub fn remove_element_by_id(&mut self, id: &String) {
        for (i, element) in self.elements.iter().enumerate() {
            if &element.info().id == id {
                self.elements.remove(i);
            }
        }
    }

    pub fn remove_elements_by_ids(&mut self, ids: Vec<&String>) {
        for id in ids {
            self.remove_element_by_id(id);
        }
    }

    pub fn remove_elements_by_tag(&mut self, tag: &String) {
        for (i, element) in self.elements.iter().enumerate() {
            if element.info().tags.contains(tag) {
                self.elements.remove(i);
            }
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

pub struct Iter {
    layout: &'static GuiLayout,
    idx: usize
}

impl Iter {
    pub fn new(layout: &GuiLayout) -> Self {
        Self {
            layout,
            idx: 0,
        }
    }
}

impl Iterator for Iter {
    type Item = &'static GuiElement;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.layout.elements().count_elements() {
            self.idx.inc();
            return Some(self.layout.elements().get_element_by_index(self.idx - 1));
        }
        None
    }
}

#[derive(Copy, Clone)]
pub enum GuiLayout {
    Void(GuiVoid),
    Section(GuiSection),
}

macro_rules! gl_fn {
    ($s:expr, $name:ident) => {
        return match $s {
            GuiLayoutt::Void(e) => {e.$name()}
            GuiLayoutt::Section(e) => {e.$name()}
            _ => {unreachable!()}
        }
    };

    ($s:expr, $name:ident, $($param:ident),*) => {
        return match $s {
            GuiLayout::Void(e) => {e.$name($($param,)*)}
            GuiLayout::Section(e) => {e.$name($($param,)*)}
            _ => {unreachable!()}
        }
    };
}

impl GuiLayout {
    pub fn elements(&self) -> &GuiElements {
        gl_fn!(self, elements);
    }

    pub fn elements_mut(&mut self) -> &mut GuiElements {
        gl_fn!(self, elements_mut);
    }

    pub fn iter(&self) -> Iter {
        return gl_fn!(self);
    }
}

pub trait GuiComponent {
    fn create() -> Self;
    fn info(&self) -> &GuiElementInfo;
    fn info_mut(&mut self) -> &mut GuiElementInfo;
    fn draw(&mut self, ctx: Arc<Draw2D>);
    fn handle(&mut self, event: GuiEvent) {
        self.info_mut().handles.push(event);
    }
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

    fn draw(&mut self, ctx: Arc<Draw2D>) {
        todo!()
    }
}

pub trait GuiLayoutComponent {
    fn elements(&self) -> &GuiElements;
    fn elements_mut(&mut self) -> &mut GuiElements;

    fn iter(&self) -> Iter {
        Iter::new(Self)
    }
}

pub(crate) fn draw_component_body(mut ctx: Arc<Draw2D>, info: &GuiElementInfo) {

    let br = resolve!(info, border_radius);
    let bs: BorderStyle = resolve!(info, border_style);

    if bs == Round {
        ctx.get_mut_gradient().copy_of(&resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.rounded_rectangle_origin_rotated(info.x, info.y, info.bounding_width, info.bounding_height, br, br as f32, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.get_mut_gradient().copy_of(&resolve!(info, border_color));
            ctx.void_rounded_rectangle_origin_rotated(info.x - bw, info.y - bw, info.bounding_width + 2 * bw, info.bounding_height + 2 * bw, bw, br + bw, (br + bw) as f32, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    } else if bs == Triangle {
        ctx.get_mut_gradient().copy_of(&resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.triangular_rectangle_origin_rotated(info.x, info.y, info.bounding_width, info.bounding_height, br, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.get_mut_gradient().copy_of(&resolve!(info, border_color));
            ctx.void_triangular_rectangle_origin_rotated(info.x - bw, info.y - bw, info.bounding_width + 2 * bw, info.bounding_height + 2 * bw, bw, br + bw, info.rotation, info.rotation_center.0, info.rotation_center.1);
        }
    } else {
        ctx.get_mut_gradient().copy_of(&resolve!(info, background_color));;
        let bw: i32 = resolve!(info, border_width);
        ctx.rectangle_origin_rotated(info.x, info.y, info.bounding_width, info.bounding_height, info.rotation, info.rotation_center.0, info.rotation_center.1);
        if br > 0 {
            ctx.get_mut_gradient().copy_of(&resolve!(info, border_color));
            ctx.void_rectangle_origin_rotated(info.x - bw, info.y - bw, info.bounding_width + 2 * bw, info.bounding_height + 2 * bw, bw, info.rotation, info.rotation_center.0, info.rotation_center.1);
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

impl GuiComponent for GuiMarkdown {
    fn create() -> Self {
        GuiMarkdown {
            info: GuiElementInfo::default(),
            text: FormattedString { pieces: vec![], whole: "".to_string() },
        }
    }

    fn info(&self) -> &GuiElementInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut GuiElementInfo {
        &mut self.info
    }

    fn draw(&mut self, mut ctx: Arc<Draw2D>) {
        if self.info.style.view_state != ViewState::Gone {
            self.info_mut().content_width = resolve!(self.info, font).unwrap().regular.get_metrics(self.text.whole.as_str()).width(self.info.content_height);
            self.info_mut().content_height = resolve!(self.info, text_size);
        } else {
            self.info_mut().content_width = 0;
            self.info_mut().content_height = 0;
        }
        self.info_mut().recalculate_bounds(ctx.clone());

        if self.info.style.view_state == ViewState::Visible {
            draw_component_body(ctx.clone(), self.info());
            let left = resolve!(self.info, padding_left);
            let bottom = resolve!(self.info, padding_bottom);

            ctx.chroma_tilt(resolve!(self.info, text_chroma_tilt));
            ctx.chroma_compress(resolve!(self.info, text_chroma_compress));
            self.text.draw(ctx.clone(), self.info.x + left, self.info.y + bottom, self.info.content_height, resolve!(self.info, font), self.info.rotation, self.info.rotation_center.0, self.info.rotation_center.1, &resolve!(self.info, text_color), resolve!(self.info, text_chroma));
        }
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

//Layouts

pub struct GuiSection {
    info: GuiElementInfo,
    elements: GuiElements
}

impl GuiLayoutComponent for GuiSection {
    fn elements(&self) -> &GuiElements {
        &self.elements
    }

    fn elements_mut(&mut self) -> &mut GuiElements {
        &mut self.elements
    }
}

fn calc_size(elements: &GuiElements) -> (i32, i32) {
    let mut width = 0;
    let mut height = 0;
    for i in 0..elements.count_elements() {
        let e = elements.get_element_by_index(i);
        width += e.info().width;
        height += e.info().height;
    }
    (width, height)
}

impl GuiComponent for GuiSection {
    fn create() -> Self {
        GuiSection {
            info: GuiElementInfo::default(),
            elements: GuiElements::new(),
        }
    }

    fn info(&self) -> &GuiElementInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut GuiElementInfo {
        &mut self.info
    }

    fn draw(&mut self, ctx: Arc<Draw2D>) {
        if self.info.style.view_state == ViewState::Gone {
            self.info_mut().content_width = 0;
            self.info_mut().content_height = 0;
        } else {
            if self.info.style.size == Size::Content {
                let size = calc_size(self.elements());
                self.info_mut().content_width = size.0;
                self.info_mut().content_height = size.1;
            } else {
                self.info_mut().content_width = resolve!(self.info, width);
                self.info_mut().content_height = resolve!(self.info, height);
            }

            self.info.recalculate_bounds(ctx.clone());
        }

        if self.info.style.view_state == ViewState::Visible {
            draw_component_body(ctx.clone(), self.info());


        }
    }
}