use std::sync::{Arc, Mutex};
use std::thread;
use mvsync::block::AwaitSync;
use mvutils::utils::Recover;
use crate::MVCore;
use crate::render::ApplicationLoopCallbacks;
use crate::render::window::{Window, WindowSpecs};

pub struct LoadingScreenSpecs {
    pub width: u32,
    pub height: u32,
    pub initial_text: &'static str
}

pub struct LoadingScreen {
    text: Arc<Mutex<String>>,
    closed: Arc<Mutex<bool>>
}

impl LoadingScreen {

    pub(crate) fn new(core: Arc<MVCore>, specs: LoadingScreenSpecs) -> Arc<LoadingScreen> {
        let text = Arc::new(Mutex::new(specs.initial_text.to_string()));
        let closed = Arc::new(Mutex::new(false));

        let loader = Loader {
            text: text.clone(),
            closed: closed.clone()
        };

        thread::spawn(move || {
            core.get_render().run_window(WindowSpecs {
                width: specs.width,
                height: specs.height,
                title: String::new(),
                fullscreen: false,
                decorated: false,
                resizable: false,
                theme: None,
                green_eco_mode: true,
                vsync: true,
                fps: 60,
                ups: 0,
            }, loader);
        });

        Arc::new(LoadingScreen {
            text,
            closed
        })
    }

    pub fn close(&self) {
        *self.closed.lock().unwrap() = true;
    }

}

struct Loader {
    text: Arc<Mutex<String>>,
    closed: Arc<Mutex<bool>>
}

impl ApplicationLoopCallbacks for Loader {
    fn start(&self, window: Arc<Window<Self>>) {

    }

    fn update(&self, _: Arc<Window<Self>>) {}

    fn draw(&self, window: Arc<Window<Self>>) {
        if *self.closed.lock().recover() {
            window.close();
            return;
        }
    }

    fn exit(&self, _: Arc<Window<Self>>) {}
}