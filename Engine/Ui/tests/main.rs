use std::cmp::Ordering;
use std::sync::Arc;
use log::{error, LevelFilter};
use mvutils::state::State;
use mvcore::asset::asset::AssetType;
use mvcore::asset::manager::{AssetHandle, AssetManager};
use mvcore::color::parse::parse_color;
use mvcore::color::RgbColor;
use mvcore::math::vec::Vec2;
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::render::texture::{DrawTexture, Texture};
use mvcore::ToAD;
use mvengine_ui::elements::{ComputeUiElement, UiElementCallbacks, UiElementStub};
use mvengine_ui::render::ctx::{DrawContext2D, DrawShape};
use mvengine_ui::render::{ctx, UiRenderer};
use mvengine_ui::render::shapes::lexer::TokenStream;
use mvengine_ui::render::shapes::modifier::boolean;
use mvengine_ui::render::shapes::polygon::Polygon;
use mvengine_ui::render::shapes::shape_gen::ShapeGenerator;
use mvengine_ui::render::shapes::ShapeParser;
use mvengine_ui::styles::Interpolator;
use mvengine_ui::timing::{AnimationState, PeriodicTask, TIMING_MANAGER};

use mvengine_ui::uix::UiCompoundElement;
use uiproc::ui;

mod test;

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Trace);
    let mut info = WindowCreateInfo::default();
    info.title = "UI test".to_string();
    info.fps = 60;
    info.ups = 20;
    info.vsync = true;

    let subject = Polygon {
        vertices: vec![
            Vec2::new(2.0, 0.5),
            Vec2::new(3.0, 3.0),
            Vec2::new(4.0, 1.0),
            Vec2::new(6.0, 2.0),
            Vec2::new(5.0, 4.0),
            Vec2::new(7.0, 5.0),
            Vec2::new(4.0, 7.0),
            Vec2::new(2.0, 6.0),
            Vec2::new(1.0, 4.0),
            Vec2::new(2.0, 2.0),
        ],
    };

    let clipping = Polygon {
        vertices: vec![
            Vec2::new(1.0, 1.0),   // Point 1
            Vec2::new(4.0, 1.0),   // Point 2
            Vec2::new(5.0, 3.0),   // Point 3
            Vec2::new(3.0, 4.0),   // Point 4
            Vec2::new(6.0, 6.0),   // Point 5
            Vec2::new(4.5, 7.0),   // Point 6
            Vec2::new(2.0, 6.0),   // Point 7
            Vec2::new(0.5, 4.0),   // Point 8
            Vec2::new(0.5, 2.0),   // Point 9
        ],
    };

    let a = Polygon {
        vertices: vec![
            Vec2::new(1.0, 1.0),   // Point 1
            Vec2::new(3.0, 1.0),   // Point 2
            Vec2::new(3.0, 2.0),   // Point 3
        ],
    };

    let b = Polygon {
        vertices: vec![
            Vec2::new(1.5, 2.0),   // Point 1
            Vec2::new(2.5, 0.5),   // Point 2
            Vec2::new(2.5, 2.0),   // Point 3
        ],
    };

    let polygon1 = Polygon {
        vertices: vec![
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 4.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(2.0, 3.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(3.0, 2.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(4.0, 1.0),
        ],
    };

    let polygon2 = Polygon {
        vertices: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 5.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(1.0, 4.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(4.0, 1.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(3.0, 2.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(2.0, 3.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0),
        ],
    };

    //let rect = ctx::rectangle().xywh(100, 100, 100, 100).create();
    //let poly = &Polygon::detriangulate(&rect)[0];
    //println!("detri: {:?}", poly);

    let window = Window::new(info);
    window.run::<Application>();
}

struct Application {
    manager: Arc<AssetManager>,
    ctx: DrawContext2D,
    rot: f32,
    img: AssetHandle,
    texture: Option<DrawTexture>,
    shape: Option<DrawShape>
}

impl ApplicationLoopCallbacks for Application {
    fn new(window: &mut Window) -> Self {
        let renderer = UiRenderer::new(window, "TestApp".to_string());
        let manager = AssetManager::new(renderer.get_device(), 1, 1);
        let mut ctx = DrawContext2D::new(renderer);

        let img = manager.create_asset("C:/Users/v22ju/Desktop/coding/rust/MVEngine/Engine/Ui/tests/img.png", AssetType::Texture);
        img.load(|asset, idk| {});
        img.wait();

        println!("loaded, valid: {}", img.is_valid());
        //unsafe {
        //    TIMING_MANAGER.request(PeriodicTask::new(-1, 1000, |win, _| {
        //        println!("FPS: {}", win.get_value().try_get::<Window>().unwrap().fps());
        //    }, AnimationState::value(window)), None);
        //}

        Self { manager, ctx, rot: 0.0, img, texture: None, shape: None }
    }

    fn post_init(&mut self, window: &mut Window) {
        self.texture = Some(DrawTexture::Texture(Arc::new(self.img.get().as_texture().unwrap())));

        let ast = ShapeParser::parse(include_str!("test.msf")).unwrap();
        let mut shape = ShapeGenerator::generate(ast).unwrap();

        //let mut shape = boolean::compute_intersect(&circle1, &rect).unwrap();

        let health = State::new("1".to_string());

        let main_page = ui! {
            <Text>{{health.map(|s| format!("Current Health: {s}"))}}</Text>
        };

        shape.set_color(RgbColor::blue());

        self.shape = Some(shape);
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        self.ctx.shape(self.shape.clone().unwrap());

        if let Err(e) = self.ctx.draw() {
            error!("{:?}", e);
        }

        self.rot += 0.5;

        unsafe {
            TIMING_MANAGER.post_frame(delta_t as f32, 0);
        }
    }

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {}
}
