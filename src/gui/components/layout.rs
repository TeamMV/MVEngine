use mvutils::utils::{Recover, TetrahedronOp};
use crate::gui::components;
use crate::gui::components::{GuiComponent, GuiElementInfo, GuiElements, GuiLayoutComponent};
use crate::gui::styles::{Direction, HorizontalAlign, Overflow, Size, VerticalAlign, ViewState};
use crate::render::draw2d::Draw2D;
use crate::resolve;

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

fn calc_size(elements: &mut GuiElements, spacing: i32) -> (i32, i32) {
    let mut width = 0;
    let mut height = 0;
    for i in 0..elements.count_elements() {
        let e = elements.get_element_by_index(i);
        width += e.read().recover().info().width + spacing;
        height += e.read().recover().info().height + spacing;
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
            let size = calc_size(self.elements_mut(), spacing);
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
            components::draw_component_body(ctx, self.info());

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
                let mut y;

                for i in 0..self.elements.count_elements() {
                    let element = self.elements.get_element_by_index(i);
                    let mut element = element.write().recover();
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
                        ctx.canvas(
                            self.info.x,
                            self.info.y,
                            self.info.content_width as u32,
                            self.info.content_height as u32,
                        );
                        ctx.style_canvas(border_style.as_cnvs_style(), border_radius as f32);
                    }

                    element.info_mut().x = x;
                    element.info_mut().y = y;

                    element.draw(ctx);

                    x += element.info().width + spacing;
                }
            } else {
                let mut x;
                let mut y = (va == VerticalAlign::Top).yn(
                    self.info.y + self.info.content_height - self.max_size.1,
                    (va == VerticalAlign::Center).yn(
                        self.info.y + self.info.content_height / 2 - self.max_size.1 / 2,
                        self.info.y,
                    ),
                );

                for i in 0..self.elements.count_elements() {
                    let element = self.elements.get_element_by_index(i);
                    let mut element = element.write().recover();
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
                        ctx.canvas(
                            self.info.x,
                            self.info.y,
                            self.info.content_width as u32,
                            self.info.content_height as u32,
                        );
                        ctx.style_canvas(border_style.as_cnvs_style(), border_radius as f32);
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