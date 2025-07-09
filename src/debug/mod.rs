pub mod print;

pub use print::*;

use mvutils::lazy;
use mvutils::utils::Time;
use parking_lot::RwLock;

lazy! {
    pub static PROFILER: MVEngineProfiler = MVEngineProfiler::new();
}



pub struct MVEngineProfiler {
    inner: RwLock<MVEnPrInner>
}

impl MVEngineProfiler {
    fn new() -> Self {
        Self {
            inner: RwLock::new(MVEnPrInner::new()),
        }
    }

    pub(crate) fn ui_compute<F: Fn(&mut Timer)>(&self, f: F) {
        let mut l = self.inner.write();
        f(&mut l.ui_compute);
    }

    pub(crate) fn ui_draw<F: Fn(&mut Timer)>(&self, f: F) {
        let mut l = self.inner.write();
        f(&mut l.ui_draw);
    }

    pub(crate) fn render_batch<F: Fn(&mut Timer)>(&self, f: F) {
        let mut l = self.inner.write();
        f(&mut l.render_batch);
    }

    pub(crate) fn render_draw<F: Fn(&mut Timer)>(&self, f: F) {
        let mut l = self.inner.write();
        f(&mut l.render_draw);
    }

    pub(crate) fn render_swap<F: Fn(&mut Timer)>(&self, f: F) {
        let mut l = self.inner.write();
        f(&mut l.render_swap);
    }

    pub(crate) fn app_draw<F: Fn(&mut Timer)>(&self, f: F) {
        let mut l = self.inner.write();
        f(&mut l.app_draw);
    }

    pub(crate) fn app_update<F: Fn(&mut Timer)>(&self, f: F) {
        let mut l = self.inner.write();
        f(&mut l.app_update);
    }

    pub(crate) fn ecs_find<F: Fn(&mut Timer)>(&self, f: F) {
        let mut l = self.inner.write();
        f(&mut l.ecs_find);
    }

    pub(crate) fn input<F: Fn(&mut Timer)>(&self, f: F) {
        let mut l = self.inner.write();
        f(&mut l.input);
    }
}

pub(crate) struct MVEnPrInner {
    pub(crate) app_draw: Timer,
    pub(crate) app_update: Timer,
    pub(crate) ui_compute: Timer,
    pub(crate) ui_draw: Timer,
    pub(crate) render_batch: Timer,
    pub(crate) render_draw: Timer,
    pub(crate) render_swap: Timer,
    pub(crate) ecs_find: Timer,
    pub(crate) input: Timer,
}

impl MVEnPrInner {
    fn new() -> Self {
        Self {
            ui_compute: Timer::new(),
            ui_draw: Timer::new(),
            render_batch: Timer::new(),
            render_draw: Timer::new(),
            render_swap: Timer::new(),
            app_draw: Timer::new(),
            app_update: Timer::new(),
            ecs_find: Timer::new(),
            input: Timer::new(),
        }
    }
}

pub(crate) struct Timer {
    start_time: u64,
    paused_at: Option<u64>,
    accumulated: u64,
    time: u64,
    running: bool,
}

impl Timer {
    fn new() -> Self {
        Self {
            start_time: 0,
            paused_at: None,
            accumulated: 0,
            time: 0,
            running: false,
        }
    }

    pub(crate) fn start(&mut self) {
        self.start_time = u64::time_nanos();
        self.accumulated = 0;
        self.paused_at = None;
        self.time = 0;
        self.running = true;
    }

    pub(crate) fn pause(&mut self) {
        if self.running && self.paused_at.is_none() {
            self.paused_at = Some(u64::time_nanos());
        }
    }

    pub(crate) fn resume(&mut self) {
        if let Some(paused_time) = self.paused_at.take() {
            let now = u64::time_nanos();
            self.accumulated += now - paused_time;
        }
    }

    pub(crate) fn stop(&mut self) {
        if self.running {
            let now = u64::time_nanos();
            let total_elapsed = match self.paused_at {
                Some(paused_time) => paused_time - self.start_time - self.accumulated,
                None => now - self.start_time - self.accumulated,
            };
            self.time = total_elapsed;
            self.running = false;
        }
    }

    pub(crate) fn time_nanos(&self) -> u64 {
        self.time
    }
}