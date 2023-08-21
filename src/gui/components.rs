use std::ops::Range;
use std::sync::Arc;

use itertools::Itertools;
use mvutils::utils::TetrahedronOp;

use crate::gui::components::GuiElement::{Layout, Void};
use crate::gui::gui_formats::FormattedString;
use crate::gui::styles::BorderStyle::{Round, Triangle};
use crate::gui::styles::{
    BorderStyle, Direction, GuiStyle, GuiValueComputeSupply, HorizontalAlign, Overflow,
    Positioning, Size, VerticalAlign, ViewState,
};
use crate::render::draw2d::Draw2D;
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

    pub fn draw(&mut self, ctx: &mut Draw2D) {
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
    elements: Vec<&'static mut GuiElement>,
}

impl GuiElements {
    pub(crate) fn new() -> Self {
        Self { elements: vec![] }
    }

    pub fn add_element(&mut self, element: &'static mut GuiElement) {
        self.elements.push(element)
    }

    pub fn add_elements(&mut self, elements: Vec<&'static mut GuiElement>) {
        self.elements.extend(elements);
    }

    pub fn get_element_by_index(&self, idx: usize) -> &'static mut GuiElement {
        self.elements[idx]
    }

    pub fn get_elements_by_range(&self, range: Range<usize>) -> &[&'static mut GuiElement] {
        self.elements[range]
    }

    pub fn get_element_by_id(&self, id: &String) -> Option<&'static mut GuiElement> {
        for element in self.elements {
            if &element.info().id == id {
                return Some(element);
            }
        }
        None
    }

    pub fn get_elements_by_tag(
        &self,
        tag: &String,
    ) -> impl Iterator<Item = &'static mut GuiElement> {
        self.elements
            .iter()
            .filter(|e| e.info().tags.contains(&tag.to_string()))
    }

    pub fn get_elements(&self) -> &[&'static mut GuiElement] {
        &self.elements
    }

    pub fn remove_element_by_id(&mut self, id: &String) {
        let mut indices: Vec<usize> = vec![];
        for (i, element) in self.elements.iter().enumerate() {
            if &element.info().id == id {
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
            if element.info().tags.contains(tag) {
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
        ctx.get_mut()
            .unwrap()
            .get_mut_gradient()
            .copy_of(&resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.get_mut().unwrap().rounded_rectangle_origin_rotated(
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
            ctx.get_mut()
                .unwrap()
                .get_mut_gradient()
                .copy_of(&resolve!(info, border_color));
            ctx.get_mut()
                .unwrap()
                .void_rounded_rectangle_origin_rotated(
                    info.x - bw,
                    info.y - bw,
                    info.bounding_width + 2 * bw,
                    info.bounding_height + 2 * bw,
                    bw,
                    br + bw,
                    (br + bw) as f32,
                    info.rotation,
                    info.rotation_center.0,
                    info.rotation_center.1,
                );
        }
    } else if bs == Triangle {
        ctx.get_mut()
            .unwrap()
            .get_mut_gradient()
            .copy_of(&resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.get_mut().unwrap().triangular_rectangle_origin_rotated(
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
            ctx.get_mut()
                .unwrap()
                .get_mut_gradient()
                .copy_of(&resolve!(info, border_color));
            ctx.get_mut()
                .unwrap()
                .void_triangular_rectangle_origin_rotated(
                    info.x - bw,
                    info.y - bw,
                    info.bounding_width + 2 * bw,
                    info.bounding_height + 2 * bw,
                    bw,
                    br + bw,
                    info.rotation,
                    info.rotation_center.0,
                    info.rotation_center.1,
                );
        }
    } else {
        ctx.get_mut()
            .unwrap()
            .get_mut_gradient()
            .copy_of(&resolve!(info, background_color));
        let bw: i32 = resolve!(info, border_width);
        ctx.get_mut().unwrap().rectangle_origin_rotated(
            info.x,
            info.y,
            info.bounding_width,
            info.bounding_height,
            info.rotation,
            info.rotation_center.0,
            info.rotation_center.1,
        );
        if br > 0 {
            ctx.get_mut()
                .unwrap()
                .get_mut_gradient()
                .copy_of(&resolve!(info, border_color));
            ctx.get_mut().unwrap().void_rectangle_origin_rotated(
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
            text: FormattedString {
                pieces: vec![],
                whole: "".to_string(),
            },
        }
    }

    fn info(&self) -> &GuiElementInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut GuiElementInfo {
        &mut self.info
    }

    fn draw(&mut self, ctx: &mut Draw2D) {
        let view_state = resolve!(self.info, view_state);
        if view_state != ViewState::Gone {
            self.info_mut().content_width = resolve!(self.info, font)
                .unwrap()
                .regular
                .get_metrics(self.text.whole.as_str())
                .width(self.info.content_height);
            self.info_mut().content_height = resolve!(self.info, text_size);
        } else {
            self.info_mut().content_width = 0;
            self.info_mut().content_height = 0;
        }
        self.info_mut().recalculate_bounds(ctx);

        if view_state == ViewState::Visible {
            draw_component_body(ctx, self.info());
            let left = resolve!(self.info, padding_left);
            let bottom = resolve!(self.info, padding_bottom);

            ctx.get_mut()
                .unwrap()
                .chroma_tilt(resolve!(self.info, text_chroma_tilt));
            ctx.get_mut()
                .unwrap()
                .chroma_compress(resolve!(self.info, text_chroma_compress));
            self.text.draw(
                ctx,
                self.info.x + left,
                self.info.y + bottom,
                self.info.content_height,
                resolve!(self.info, font),
                self.info.rotation,
                self.info.rotation_center.0,
                self.info.rotation_center.1,
                &resolve!(self.info, text_color),
                resolve!(self.info, text_chroma),
            );
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
    elements: GuiElements,
    max_size: (i32, i32),
}

impl GuiLayoutComponent for GuiSection {
    fn elements(&self) -> &GuiElements {
        &self.elements
    }

    fn elements_mut(&mut self) -> &mut GuiElements {
        &mut self.elements
    }
}

fn calc_size(elements: &GuiElements, spacing: i32) -> (i32, i32) {
    let mut width = 0;
    let mut height = 0;
    for i in 0..elements.count_elements() {
        let e = elements.get_element_by_index(i);
        width += e.info().width + spacing;
        height += e.info().height + spacing;
    }
    (width - spacing, height - spacing)
}

impl GuiComponent for GuiSection {
    fn create() -> Self {
        GuiSection {
            info: GuiElementInfo::default(),
            elements: GuiElements::new(),
            max_size: (0, 0),
        }
    }

    fn info(&self) -> &GuiElementInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut GuiElementInfo {
        &mut self.info
    }

    fn draw(&mut self, ctx: &mut Draw2D) {
        let view_state = resolve!(self.info, view_state);
        let spacing = resolve!(self.info, spacing);

        if view_state == ViewState::Gone {
            self.info_mut().content_width = 0;
            self.info_mut().content_height = 0;
        } else {
            let sizing = resolve!(self.info, size);
            let size = calc_size(self.elements(), spacing);
            self.max_size = size;
            if sizing == Size::Content {
                self.info_mut().content_width = size.0;
                self.info_mut().content_height = size.1;
            } else {
                self.info_mut().content_width = resolve!(self.info, width);
                self.info_mut().content_height = resolve!(self.info, height);
            }

            self.info.recalculate_bounds(ctx);
        }

        if view_state == ViewState::Visible {
            draw_component_body(ctx, self.info());

            let ha = resolve!(self.info, horizontal_align);
            let va = resolve!(self.info, vertical_align);
            let hia = resolve!(self.info, horizontal_item_align);
            let via = resolve!(self.info, vertical_item_align);
            let dir = resolve!(self.info, item_direction);

            let overflow = resolve!(self.info, overflow);

            let border_radius = resolve!(self.info, border_radius);
            let border_style = resolve!(self.info, border_style);

            if dir == Direction::LeftRight {
                let mut x = (ha == HorizontalAlign::Right).yn(
                    self.info.x + self.info.content_width - self.max_size.0,
                    (ha == HorizontalAlign::Center).yn(
                        self.info.x + self.info.content_width / 2 - self.max_size.0 / 2,
                        self.info.x,
                    ),
                );
                let mut y = 0;

                for i in 0..self.elements.count_elements() {
                    let mut element = self.elements.get_element_by_index(i).as_mut();
                    if va == VerticalAlign::Bottom {
                        if via == VerticalAlign::Bottom {
                            y = self.info.y;
                        } else if via == VerticalAlign::Center {
                            y = self.info.y + (self.max_size.1 - element.info().height) / 2;
                        } else {
                            y = self.info.y + (self.max_size.1 - element.info().height);
                        }
                    } else if va == VerticalAlign::Center {
                        if via == VerticalAlign::Bottom {
                            y = self.info.y + self.info.content_height / 2 - self.max_size.1 / 2;
                        } else if via == VerticalAlign::Center {
                            y = self.info.y
                                + (self.max_size.1 - element.info().height) / 2
                                + self.info.content_height / 2
                                - self.max_size.1 / 2;
                        } else {
                            y = self.info.y
                                + (self.max_size.1 - element.info().height)
                                + self.info.content_height / 2
                                - self.max_size.1 / 2;
                        }
                    } else {
                        if via == VerticalAlign::Bottom {
                            y = self.info.y + self.info.content_height - self.max_size.1;
                        } else if via == VerticalAlign::Center {
                            y = self.info.y
                                + (self.max_size.1 - element.info().height) / 2
                                + self.info.content_height
                                - self.max_size.1;
                        } else {
                            y = self.info.y
                                + (self.max_size.1 - element.info().height)
                                + self.info.content_height
                                - self.max_size.1;
                        }
                    }

                    if overflow == Overflow::Clamp {
                        x = x.clamp(
                            self.info.x,
                            self.info.x + self.info.content_width - element.info().width,
                        );
                        y = y.clamp(
                            self.info.y,
                            self.info.y + self.info.content_height - element.info().height,
                        );
                    } else if overflow == Overflow::Cut {
                        ctx.get_mut().unwrap().canvas(
                            self.info.x,
                            self.info.y,
                            self.info.content_width as u32,
                            self.info.content_height as u32,
                        );
                        ctx.get_mut()
                            .unwrap()
                            .style_canvas(border_style.as_cnvs_style(), border_radius as f32);
                    }

                    element.info_mut().x = x;
                    element.info_mut().y = y;

                    element.draw(ctx);

                    x += element.info().width + spacing;
                }
            } else {
                let mut x = 0;
                let mut y = (va == VerticalAlign::Top).yn(
                    self.info.y + self.info.content_height - self.max_size.1,
                    (va == VerticalAlign::Center).yn(
                        self.info.y + self.info.content_height / 2 - self.max_size.1 / 2,
                        self.info.y,
                    ),
                );

                for i in 0..self.elements.count_elements() {
                    let mut element = self.elements.get_element_by_index(i);
                    if ha == HorizontalAlign::Left {
                        if hia == HorizontalAlign::Left {
                            x = self.info.x;
                        } else if hia == HorizontalAlign::Center {
                            x = self.info.x + (self.max_size.0 - element.info().width) / 2;
                        } else {
                            x = self.info.x + (self.max_size.0 - element.info().width);
                        }
                    } else if ha == HorizontalAlign::Center {
                        if hia == HorizontalAlign::Left {
                            x = self.info.x + self.info.content_width / 2 - self.max_size.0 / 2;
                        } else if hia == HorizontalAlign::Center {
                            x = self.info.x
                                + (self.max_size.0 - element.info().width) / 2
                                + self.info.content_width / 2
                                - self.max_size.0 / 2;
                        } else {
                            x = self.info.x
                                + (self.max_size.0 - element.info().width)
                                + self.info.content_width / 2
                                - self.max_size.0 / 2;
                        }
                    } else {
                        if hia == HorizontalAlign::Left {
                            x = self.info.x + self.info.content_width - self.max_size.0;
                        } else if hia == HorizontalAlign::Center {
                            x = self.info.x
                                + (self.max_size.0 - element.info().width) / 2
                                + self.info.content_width
                                - self.max_size.0;
                        } else {
                            x = self.info.x
                                + (self.max_size.0 - element.info().width)
                                + self.info.content_width
                                - self.max_size.0;
                        }
                    }

                    if overflow == Overflow::Clamp {
                        x = x.clamp(
                            self.info.x,
                            self.info.x + self.info.content_width - element.info().width,
                        );
                        y = y.clamp(
                            self.info.y,
                            self.info.y + self.info.content_height - element.info().height,
                        );
                    } else if overflow == Overflow::Cut {
                        ctx.get_mut().unwrap().canvas(
                            self.info.x,
                            self.info.y,
                            self.info.content_width as u32,
                            self.info.content_height as u32,
                        );
                        ctx.get_mut()
                            .unwrap()
                            .style_canvas(border_style.as_cnvs_style(), border_radius as f32);
                    }

                    element.info_mut().x = x;
                    element.info_mut().y = y;

                    element.draw(ctx);

                    y += element.info().height + spacing;
                }
            }
        }
    }
}
