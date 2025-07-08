use crate::graphics::tileset::{
    ClockingFramePump, FramePumpRange, LoopingRangeFramePump, Pump, TileSet,
};
use crate::math::vec::Vec4;
use crate::rendering::texture::Texture;
use crate::ui::context::UiResources;
use crate::ui::res::runtime::ResourceSavable;
use mvutils::save::{Loader, Savable, Saver};
use mvutils::unsafe_utils::Unsafe;
use std::ops::RangeBounds;

#[derive(Clone)]
pub struct GlobalAnimation<'a> {
    pump: ClockingFramePump<'a, LoopingRangeFramePump<'a>>,
    last: (&'a Texture, Vec4),
    tile_set: usize,
    fps: u16,
    range: FramePumpRange,
}

impl<'a> GlobalAnimation<'a> {
    pub fn new(
        tile_set: &'a TileSet,
        tile_set_res: usize,
        range: impl RangeBounds<usize>,
        fps: u16,
    ) -> Self {
        let fprange = FramePumpRange {
            start: range.start_bound().cloned(),
            end: range.end_bound().cloned(),
        };
        let mut pump = tile_set.frames_loop_range(range).clocked(fps);
        let last = pump.pump();
        Self {
            pump,
            last,
            tile_set: tile_set_res,
            fps,
            range: fprange,
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

impl ResourceSavable for GlobalAnimation<'_> {
    fn save_res(&self, saver: &mut impl Saver) {
        self.tile_set.save(saver);
        self.fps.save(saver);
        self.range.save(saver);
    }

    fn load_res(loader: &mut impl Loader, resources: &impl UiResources) -> Result<Self, String> {
        let tile_set = usize::load(loader)?;
        let fps = u16::load(loader)?;
        let range = FramePumpRange::load(loader)?;
        if let Some(tileset) = resources.resolve_tileset(tile_set) {
            //since this is only really used internally, this unsafe cast is probably fine
            let static_set = unsafe { Unsafe::cast_lifetime(tileset) };
            Ok(GlobalAnimation::<'static>::new(
                static_set, tile_set, range, fps,
            ))
        } else {
            Err("Couldnt resolve tileset for animation!".to_string())
        }
    }
}
