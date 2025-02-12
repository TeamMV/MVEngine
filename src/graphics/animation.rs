use std::ops::RangeBounds;
use crate::graphics::tileset::{ClockingFramePump, LoopingRangeFramePump, Pump, TileSet};
use crate::math::vec::Vec4;
use crate::rendering::texture::Texture;

pub struct GlobalAnimation<'a> {
    pump: ClockingFramePump<'a, LoopingRangeFramePump<'a>>,
    last: (&'a Texture, Vec4)
}

impl<'a> GlobalAnimation<'a> {
    pub fn new(tile_set: &'a TileSet, range: impl RangeBounds<usize>, fps: u16) -> Self {
        let mut pump = tile_set.frames_loop_range(range).clocked(fps);
        let last = pump.pump();
        Self {
            pump,
            last
        }
    }

    pub fn tick(&mut self) {
        self.last = self.pump.pump();
    }

    pub fn get_current(&self) -> (&'a Texture, Vec4) {
        self.last
    }

    pub fn start(&mut self) {
        self.pump.enable();
    }

    pub fn stop(&mut self) {
        self.pump.disable();
    }

    pub fn set_fps(&mut self, fps: u16) {
        self.pump.set_fps(fps);
    }
}