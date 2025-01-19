use std::any::TypeId;
use std::cmp::Ordering;
use std::ops::Deref;
use std::sync::Arc;
use log::{error, LevelFilter};
use mvutils::once::CreateOnce;
use mvutils::state::State;
use parking_lot::RwLock;
use mvcore::asset::asset::AssetType;
use mvcore::asset::manager::{AssetHandle, AssetManager};
use mvcore::color::parse::parse_color;
use mvcore::color::RgbColor;
use mvcore::math::vec::Vec2;
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::render::texture::{DrawTexture, Texture};
use mvcore::ToAD;
use mvengine_ui::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use mvengine_ui::render::ctx::{DrawContext2D, DrawShape};
use mvengine_ui::render::{ctx, UiRenderer};
use mvengine_ui::render::shapes::lexer::TokenStream;
use mvengine_ui::render::shapes::modifier::boolean;
use mvengine_ui::render::shapes::polygon::Polygon;
use mvengine_ui::render::shapes::shape_gen::ShapeGenerator;
use mvengine_ui::render::shapes::ShapeParser;
use mvengine_ui::styles::{BackgroundRes, ChildAlign, Direction, Interpolator, Origin, Position, SideStyle, UiStyle, UiValue, EMPTY_STYLE};
use mvengine_ui::timing::{AnimationState, PeriodicTask, TIMING_MANAGER};
use mvengine_ui::elements::implementations::div::Div;
use mvengine_ui::{anim, get_shape, modify_style, ui, ui_mut, Ui};
use mvengine_ui::anim::{easing, AnimationMode, FillMode};
use mvengine_ui::attributes::Attributes;
use mvengine_ui::context::UiResources;
use mvengine_ui::ease::{Easing, EasingGen, EasingMode};
use mvengine_ui::uix::UiCompoundElement;
use uiproc::ui;
use mvengine_ui::elements::child::{Child, ToChild};
use mvengine_ui::elements::events::UiHoverAction;
use crate::r::R;

mod test;
mod r;

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Trace);
    let mut info = WindowCreateInfo::default();
    info.title = "UI test".to_string();
    info.fps = 60;
    info.ups = 20;
    info.vsync = true;

    let window = Window::new(info);
    window.run::<Application>();
}

struct Application {
    manager: Arc<AssetManager>,
    ctx: DrawContext2D,
}

impl ApplicationLoopCallbacks for Application {
    fn new(window: &mut Window) -> Self {
        let renderer = UiRenderer::new(window, "TestApp".to_string());
        let manager = AssetManager::new(renderer.get_device(), 1, 1);
        let mut ctx = DrawContext2D::new(renderer);
        //unsafe {
        //    TIMING_MANAGER.request(PeriodicTask::new(-1, 1000, |win, _| {
        //        println!("FPS: {}", win.get_value().try_get::<Window>().unwrap().fps());
        //    }, AnimationState::value(window)), None);
        //}

        R::initialize(ctx.renderer().get_device());
        let ui = ui_mut();
        ui.init(R.deref().deref());
        ui.init_input(window.get_input());
        window.set_input_processor(Ui::input_processor);

        let mut style = UiStyle::default();
        modify_style!(style.x = UiValue::Just(100));
        modify_style!(style.y = UiValue::Just(100));
        modify_style!(style.width = UiValue::Auto);
        modify_style!(style.height = UiValue::Auto);
        modify_style!(style.position = UiValue::Just(Position::Absolute));
        modify_style!(style.child_align = UiValue::Just(ChildAlign::Middle));

        let mut style2 = UiStyle::default();
        modify_style!(style2.width = UiValue::Just(50));
        modify_style!(style2.height = UiValue::Just(50));
        modify_style!(style2.background.color = UiValue::Just(RgbColor::red()));

        let mut style3 = UiStyle::default();
        modify_style!(style3.direction = UiValue::Just(Direction::Vertical));
        modify_style!(style3.child_align = UiValue::Just(ChildAlign::Middle));
        modify_style!(style3.background.color = UiValue::Just(RgbColor::blue()));

        let mut style4 = UiStyle::default();
        modify_style!(style4.height = UiValue::Just(25));
        modify_style!(style4.width = UiValue::Just(25));
        modify_style!(style4.background.resource = UiValue::Just(BackgroundRes::Texture.into()));
        modify_style!(style4.background.texture = UiValue::Just(R.mv.texture.test.into()));

        let mut anim_style = style.clone();
        modify_style!(anim_style.background.color = UiValue::Just(RgbColor::magenta()));

        let mut div = ui! {
            <Div style={style.clone()}>
                <Div style={style2}/>
                <Div style={style3}>
                    <Div style={style4.clone()}/>
                    <Div style={style4.clone()}/>
                    <Div style={style4}/>
                </Div>
            </Div>
        };

        div.state_mut().events.on_hover(move |e| {
            if let UiHoverAction::Enter = e.base.action {
                anim::animate_self(e.base.elem, &anim_style, 500, easing(EasingGen::linear(), EasingMode::In), FillMode::Keep, AnimationMode::KeepProgress);
            } else {
                anim::animate_self(e.base.elem, &style, 500, easing(EasingGen::linear(), EasingMode::In), FillMode::Keep, AnimationMode::KeepProgress);
            }
        });

        ui.add_root(Arc::new(RwLock::new(div.wrap())));

        Self { manager, ctx }
    }

    fn post_init(&mut self, window: &mut Window) {
        let health = State::new("1".to_string());
        let main_page = ui! {
            <Div>{{health.map(|s| format!("Current Health: {s}"))}}</Div>
        };
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        ui_mut().compute_styles_and_draw(&mut self.ctx);

        if let Err(e) = self.ctx.draw() {
            error!("{:?}", e);
        }

        unsafe {
            TIMING_MANAGER.post_frame(delta_t as f32, 0);
        }
    }

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {}
}
