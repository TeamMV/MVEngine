use std::alloc::Layout;
use std::ops::Index;
use hashbrown::Equivalent;

pub const PHI: f64 = 1.618033988749894848204586834365638118_f64;

pub struct ContinuousBlob {
    data: *mut u8,
    len: usize,
    capacity: usize,
    layout: Layout
}

impl ContinuousBlob {
    pub fn new(layout: Layout) -> Self {
        let ptr = unsafe { std::alloc::alloc(layout) };

        let mut this = Self {
            data: ptr,
            len: 0,
            capacity: layout.size(),
            layout,
        };

        this
    }

    fn realloc(&mut self) {
        self.capacity = (self.capacity as f64 * PHI).ceil() as usize;
        unsafe {
            self.data = std::alloc::realloc(self.data, Layout::from_size_align_unchecked(
            self.capacity * self.layout.size(), self.layout.align()
            ), self.capacity);
        }
    }

    pub fn push_next<T: Sized + 'static>(&mut self, t: T) -> Option<usize> {
        unsafe {
            if self.layout.equivalent(&Layout::for_value(&t)) {
                if self.capacity < self.len + self.layout.size() {
                    self.realloc();
                }
                let added = self.data.add(self.len * self.layout.size());
                let typed = added as *mut T;
                typed.write(t);
                self.len += 1;
                return Some(self.len - 1);
            }
        }
        None
    }

    pub fn get<T: Sized + 'static>(&self, idx: usize) -> Option<&T> {
        if idx >= 0 && idx < self.len {
            unsafe {
                let added = self.data.add(idx * self.layout.size());
                let typed = added as *mut T;
                return typed.as_ref()
            }
        }
        None
    }

    pub fn get_mut<T: Sized + 'static>(&mut self, idx: usize) -> Option<&mut T> {
        if idx >= 0 && idx < self.len {
            unsafe {
                let added = self.data.add(idx * self.layout.size());
                let typed = added as *mut T;
                return typed.as_mut()
            }
        }
        None
    }
}