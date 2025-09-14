use crate::ui::elements::child::ToChild;
use crate::ui::elements::{Button, LocalElement, UiElementBuilder, _Self};
use crate::modify_style;
use mvengine_proc_macro::style_expr;
use crate::input::{Input, MouseAction, RawInputEvent};
use crate::rendering::pipeline::RenderingPipeline;
use crate::rendering::{OpenGLRenderer, RenderContext};
use crate::ui::attributes::{Attributes, IntoAttrib, UiState};
use crate::ui::context::UiContext;
use crate::ui::elements::components::boring::BoringText;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{create_style_obs, Element, UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::geometry::SimpleRect;
use crate::ui::styles::{InheritSupplier, UiStyle, UiStyleWriteObserver};
use mvutils::state::State;
use mvutils::unsafe_utils::DangerousCell;
use ropey::Rope;
use std::rc::{Rc, Weak};
use mvutils::once::CreateOnce;
use mvutils::thread::ThreadSafe;
use mvengine_proc_macro::{multiline_str_into, ui};
use crate::input::consts::MouseButton;
use crate::ui::elements::child::{Child, ToChildFromIterator};
use crate::ui::elements::div::Div;
use crate::ui::rendering::WideRenderContext;

#[derive(Clone)]
pub struct Selection {
    value: Rope,
    index: usize
}

impl Selection {
    pub fn new() -> Self {
        Self {
            value: Rope::new(),
            index: 0,
        }
    }
}

#[derive(Clone)]
pub struct ComboBox {
    weak: LocalElement,
    attributes: Attributes,
    state: UiElementState,
    style: UiStyle,
    body: ElementBody,
    text: BoringText,
    context: UiContext,
    selection: State<Selection>,
    values: State<Vec<Rope>>,
    open: bool,
    list_elem: ThreadSafe<Element>
}

impl UiElementBuilder for ComboBox {
    fn _builder(&self, context: UiContext, attributes: Attributes, style: UiStyle) -> _Self {
        ()
    }

    fn set_weak(&mut self, weak: LocalElement) {
        self.weak = weak;
    }

    fn wrap(self) -> UiElement {
        UiElement::ComboBox(self)
    }
}

impl ComboBox {
    pub fn builder(context: UiContext, attributes: Attributes, style: UiStyle) -> Self {
        use crate as mvengine;
        let div_style = UiStyle::inheriting();
        let div = ui! {
            <Ui context={context.clone()}>
                <Div style={div_style}>

                </Div>
            </Ui>>
        };

        Self {
            weak: LocalElement::new(),
            attributes,
            state: UiElementState::new(context.clone()),
            style,
            body: ElementBody::new(),
            text: BoringText,
            context,
            selection: State::new(Selection::new()),
            values: State::new(vec![]),
            open: false,
            list_elem: ThreadSafe::new(div),
        }
    }

    pub fn selection<T: IntoAttrib<State<Selection>>>(mut self, attrib: T) -> Self {
        self.selection = attrib.into_attrib();
        self
    }

    pub fn values<T: IntoAttrib<State<Vec<Rope>>>>(mut self, attrib: T) -> Self {
        self.values = attrib.into_attrib();
        let child = Self::get_child(&self.values, self.context.clone());
        let child = child.to_child();
        let div = self.list_elem.get_mut();
        div.remove_all_children();
        div.add_child(child);
        self
    }
}

impl UiElementStub for ComboBox {
    fn wrapped(&self) -> Element {
        self.weak.to_wrapped()
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }

    fn state(&self) -> &UiElementState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut UiElementState {
        &mut self.state
    }

    fn style(&self) -> &UiStyle {
        &self.style
    }

    fn style_mut(&mut self) -> UiStyleWriteObserver {
        create_style_obs(&mut self.style, &mut self.state)
    }

    fn context(&self) -> &UiContext {
        &self.context
    }

    fn body(&self) -> &ElementBody {
        &self.body
    }

    fn body_mut(&mut self) -> &mut ElementBody {
        &mut self.body
    }
}

impl UiElementCallbacks for ComboBox {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect, debug: bool) {
        self.body.draw(&self.style, &self.state, ctx, &self.context, crop_area);

        if self.values.is_outdated() {
            self.values.update();
            let div = self.list_elem.get_mut();
            div.remove_all_children();
            let c = Self::get_child(&self.values, self.context.clone());
            div.add_child(c.to_child());
        }

        if self.open {
            let div = self.list_elem.get_mut();
            let tx = self.state.rect.x();
            let ty = self.state.rect.y() - self.state.rect.height();

            if div.state().is_valid() {
                div.compute_styles(ctx);
            }

            let div_state = div.state_mut();
            div_state.move_to(tx, ty);

            let div_rect = &div_state.rect.bounding;
            let area = ctx.area();
            let div_rect = div_rect.create_clamp(&area);
            div_state.move_to(div_rect.x, div_rect.y);

            ctx.push();
            ctx.next_layer();
            div.draw(ctx, crop_area, debug);
            ctx.pop();
        }
    }

    fn raw_input_callback(&mut self, action: RawInputEvent, input: &Input) -> bool {
        let mx = input.mouse_x;
        let my = input.mouse_y;
        let is_inside = self.inside(mx, my);

        let div = self.list_elem.get_mut();
        if let RawInputEvent::Mouse(MouseAction::Press(MouseButton::Left)) = &action {
            if !self.open {
                if is_inside {
                    self.open = true;
                    div.state_mut().invalidate();
                }
            }
        }
        self.super_input(action, input)
    }
}

impl ComboBox {
    pub fn get_child(values: &State<Vec<Rope>>, context: UiContext) -> impl Iterator<Item=Child> {
        let c = context;
        let vals = values.read().clone();
        vals.into_iter()
            .map(move |x| {
                use crate as mvengine;

                let style = UiStyle::inheriting();
                let elem = ui! {
                    <Ui context={c.clone()}>
                        <Button style={style}>{x}</Button>
                    </Ui>
                };
                elem.to_child()
            })
    }
}