use hashbrown::{Equivalent, HashMap};
use std::alloc::Layout;
use std::ptr;
use std::ptr::Pointee;
use mvutils::hashers::U64IdentityHasher;
use crate::game::ecs::mem::storage::ComponentIdx;

pub const PHI: f64 = 1.618033988749894848204586834365638118_f64;

type InternalPointer = usize;

pub struct ContinuousBlob {
    data: *mut u8,
    len: usize,
    capacity: usize,
    layout: Layout,
    memmap: HashMap<ComponentIdx, InternalPointer, U64IdentityHasher>,
    next_idx: ComponentIdx,
}

impl ContinuousBlob {
    pub fn new(layout: Layout) -> Self {
        let ptr = unsafe { std::alloc::alloc(layout) };

        let this = Self {
            data: ptr,
            len: 0,
            capacity: layout.size(),
            layout,
            memmap: HashMap::with_hasher(U64IdentityHasher::default()),
            next_idx: 0,
        };

        this
    }

    fn realloc(&mut self) {
        self.capacity = (self.capacity as f64 * PHI).ceil() as usize;
        unsafe {
            self.data = std::alloc::realloc(
                self.data,
                Layout::from_size_align_unchecked(
                    self.capacity * self.layout.size(),
                    self.layout.align(),
                ),
                self.capacity,
            );
        }
    }

    fn maybe_shrink(&mut self) {
        if (self.len as f64 * PHI).ceil() < self.capacity as f64 { 
            self.capacity = self.len + 2;
            unsafe {
                self.data = std::alloc::realloc(
                    self.data,
                    Layout::from_size_align_unchecked(
                        self.capacity * self.layout.size(),
                        self.layout.align(),
                    ),
                    self.capacity,
                );
            }
        }
    }

    pub fn push_next<T: Sized + 'static>(&mut self, t: T) -> Option<ComponentIdx> {
        unsafe {
            if self.layout.equivalent(&Layout::for_value(&t)) {
                if self.capacity < self.len + self.layout.size() {
                    self.realloc();
                }
                let added = self.data.add(self.len * self.layout.size());
                let typed = added as *mut T;
                typed.write(t);
                self.memmap.insert(self.next_idx, self.len);
                self.len += 1;
                self.next_idx += 1;
                return Some(self.next_idx - 1);
            }
        }
        None
    }

    pub fn get<T: Sized + 'static>(&self, idx: ComponentIdx) -> Option<&T> {
        let idx = *self.memmap.get(&idx)?;
        if idx < self.len {
            unsafe {
                let added = self.data.add(idx * self.layout.size());
                let typed = added as *mut T;
                return typed.as_ref();
            }
        }
        None
    }

    pub fn get_mut<T: Sized + 'static>(&mut self, idx: ComponentIdx) -> Option<&mut T> {
        let idx = *self.memmap.get(&idx)?;
        if idx < self.len {
            unsafe {
                let added = self.data.add(idx * self.layout.size());
                let typed = added as *mut T;
                return typed.as_mut();
            }
        }
        None
    }

    pub fn remove(&mut self, idx: ComponentIdx) {
        if let Some(idx) = self.memmap.remove(&idx) {
            if idx >= self.len { return; }
            self.len -= 1;
            if idx < self.len {
                unsafe {
                    let src = self.data.add(idx * self.layout.size());
                    let dst = self.data.add(self.len * self.layout.size());
                    ptr::copy_nonoverlapping(src, dst, self.layout.size());
                }
            }
            self.maybe_shrink();
        }
    }

    pub fn get_all<T: Sized + 'static>(&self) -> Vec<&T> {
        let mut out = Vec::with_capacity(self.len);
        unsafe {
            for i in 0..self.len {
                let added = self.data.add(i * self.layout.size());
                let typed = added as *mut T;
                if typed.as_ref().is_some() {
                    out.push(typed.as_ref().unwrap());
                }
            }
        }
        out
    }

    pub fn get_all_mut<T: Sized + 'static>(&mut self) -> Vec<&mut T> {
        let mut out = Vec::with_capacity(self.len);
        unsafe {
            for i in 0..self.len {
                let added = self.data.add(i * self.layout.size());
                let typed = added as *mut T;
                if typed.as_mut().is_some() {
                    out.push(typed.as_mut().unwrap());
                }
            }
        }
        out
    }

    pub fn get_all_traits_mut<T: ?Sized>(
        &mut self,
        meta: &<T as Pointee>::Metadata,
    ) -> Vec<&mut T> {
        let mut out = Vec::with_capacity(self.len);
        unsafe {
            for i in 0..self.len {
                let added = self.data.add(i * self.layout.size()).as_mut().unwrap();
                let typed = std::ptr::from_raw_parts_mut(added, *meta) as *mut T;
                //let typed: &mut T = mem::transmute::<&mut u8, &mut T>(added);
                out.push(typed.as_mut().unwrap());
            }
        }
        out
    }
}
