use crate::math::vec::Vec4;
use crate::rendering::texture::Texture;
use mvutils::clock::Clock;
use std::collections::Bound;
use std::ops::RangeBounds;

#[derive(Clone)]
struct FramePumpRange {
    start: Bound<usize>,
    end: Bound<usize>,
}

impl FramePumpRange {
    fn new<R: RangeBounds<usize>>(range: R) -> Self {
        Self {
            start: range.start_bound().cloned(),
            end: range.end_bound().cloned(),
        }
    }

    fn start(&self) -> usize {
        match self.start {
            Bound::Included(frame) => frame,
            Bound::Excluded(frame) => frame + 1,
            Bound::Unbounded => 0,
        }
    }

    fn end_check(&self, value: usize, total: usize) -> bool {
        value < total
            && match self.end {
                Bound::Included(frame) => value <= frame,
                Bound::Excluded(frame) => value < frame,
                Bound::Unbounded => true,
            }
    }
}

pub struct TileSet {
    texture: Texture,
    tile_width: i32,
    tile_height: i32,
    count: usize,
    cache: Vec<Vec4>,
}

impl TileSet {
    pub fn new(texture: Texture, width: i32, height: i32, count: usize) -> Self {
        let mut cache = Vec::with_capacity(count);

        let (tex_width, tex_height) = texture.dimensions;
        let z = width as f32 / tex_width as f32;
        let w = height as f32 / tex_height as f32;

        for i in 0..count {
            let xoff = (width * i as i32) % tex_width as i32;
            let yoff = (width * i as i32) / tex_width as i32 * height;

            let coords = Vec4::new(
                xoff as f32 / tex_width as f32,
                1.0 - w - yoff as f32 / tex_height as f32,
                z,
                w,
            );
            cache.push(coords);
        }
        Self {
            texture,
            tile_width: width,
            tile_height: height,
            count,
            cache,
        }
    }

    pub fn get_tile(&self, index: usize) -> Option<(&Texture, Vec4)> {
        if index >= self.count {
            return None;
        }
        Some((&self.texture, self.cache[index]))
    }

    pub fn frames(&self) -> FramePump<'_> {
        FramePump {
            tileset: self,
            current: 0,
        }
    }

    pub fn frames_loop(&self) -> LoopingFramePump<'_> {
        LoopingFramePump {
            tileset: self,
            current: 0,
        }
    }

    pub fn frames_range<R: RangeBounds<usize>>(&self, range: R) -> RangeFramePump<'_> {
        let range = FramePumpRange::new(range);
        RangeFramePump {
            tileset: self,
            current: range.start(),
            range,
        }
    }

    pub fn frames_loop_range<R: RangeBounds<usize>>(&self, range: R) -> LoopingRangeFramePump<'_> {
        let range = FramePumpRange::new(range);
        LoopingRangeFramePump {
            tileset: self,
            current: range.start(),
            range,
        }
    }

    pub fn get_texture(&self) -> &Texture {
        &self.texture
    }

    pub fn tile_width(&self) -> i32 {
        self.tile_width
    }

    pub fn tile_height(&self) -> i32 {
        self.tile_height
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

pub trait Pump {
    type Item;

    fn pump(&mut self) -> Self::Item;
}

#[derive(Clone)]
pub struct FramePump<'a> {
    tileset: &'a TileSet,
    current: usize,
}

impl<'a> Pump for FramePump<'a> {
    type Item = Option<(&'a Texture, Vec4)>;

    fn pump(&mut self) -> Self::Item {
        if self.current >= self.tileset.count {
            return None;
        }
        let res = self.tileset.get_tile(self.current);
        self.current += 1;
        res
    }
}

#[derive(Clone)]
pub struct RangeFramePump<'a> {
    tileset: &'a TileSet,
    current: usize,
    range: FramePumpRange,
}

impl<'a> Pump for RangeFramePump<'a> {
    type Item = Option<(&'a Texture, Vec4)>;

    fn pump(&mut self) -> Self::Item {
        if !self.range.end_check(self.current, self.tileset.count) {
            return None;
        }
        let res = self.tileset.get_tile(self.current);
        self.current += 1;
        res
    }
}

#[derive(Clone)]
pub struct LoopingFramePump<'a> {
    tileset: &'a TileSet,
    current: usize,
}

impl<'a> LoopingFramePump<'a> {
    pub fn clocked(self, fps: u16) -> ClockingFramePump<'a, Self> {
        ClockingFramePump::new(self, fps)
    }

    pub fn clocked_disabled(self, fps: u16) -> ClockingFramePump<'a, Self> {
        ClockingFramePump::new_disabled(self, fps)
    }
}

impl<'a> Pump for LoopingFramePump<'a> {
    type Item = (&'a Texture, Vec4);

    fn pump(&mut self) -> Self::Item {
        if self.current >= self.tileset.count {
            self.current = 0;
        }
        let res = self
            .tileset
            .get_tile(self.current)
            .expect("Frame will always exist");
        self.current += 1;
        res
    }
}

#[derive(Clone)]
pub struct LoopingRangeFramePump<'a> {
    tileset: &'a TileSet,
    current: usize,
    range: FramePumpRange,
}

impl<'a> LoopingRangeFramePump<'a> {
    pub fn clocked(self, fps: u16) -> ClockingFramePump<'a, Self> {
        ClockingFramePump::new(self, fps)
    }

    pub fn clocked_disabled(self, fps: u16) -> ClockingFramePump<'a, Self> {
        ClockingFramePump::new_disabled(self, fps)
    }
}

impl<'a> Pump for LoopingRangeFramePump<'a> {
    type Item = (&'a Texture, Vec4);

    fn pump(&mut self) -> Self::Item {
        if !self.range.end_check(self.current, self.tileset.count) {
            self.current = self.range.start();
        }
        let res = self
            .tileset
            .get_tile(self.current)
            .expect("Frame will always exist");
        self.current += 1;
        res
    }
}

#[derive(Clone)]
pub struct ClockingFramePump<'a, P: Pump<Item = (&'a Texture, Vec4)>> {
    pump: P,
    current: (&'a Texture, Vec4),
    clock: Clock,
}

impl<'a, P: Pump<Item = (&'a Texture, Vec4)>> ClockingFramePump<'a, P> {
    pub fn new(mut pump: P, fps: u16) -> Self {
        let current = pump.pump();
        ClockingFramePump {
            pump,
            current,
            clock: Clock::new(fps),
        }
    }

    pub fn new_disabled(mut pump: P, fps: u16) -> Self {
        let current = pump.pump();
        ClockingFramePump {
            pump,
            current,
            clock: Clock::new_disabled(fps),
        }
    }

    pub fn set_fps(&mut self, fps: u16) {
        self.clock.set_tps(fps);
    }

    pub fn disable(&mut self) {
        self.clock.disable();
    }

    pub fn enable(&mut self) {
        self.clock.enable();
    }
}

impl<'a, P: Pump<Item = (&'a Texture, Vec4)>> Pump for ClockingFramePump<'a, P> {
    type Item = P::Item;

    fn pump(&mut self) -> Self::Item {
        if self.clock.ready() {
            self.clock.tick();
            self.current = self.pump.pump();
        }
        self.current
    }
}
