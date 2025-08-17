use crate::game::ecs::mem::storage::ComponentIdx;
use hashbrown::{Equivalent, HashMap};
use mvutils::hashers::U64IdentityHasher;
use std::alloc::Layout;
use std::hash::DefaultHasher;
use std::marker::PhantomData;
use std::{alloc, ptr};
use std::ptr::Pointee;
use bimap::BiHashMap;
use crate::ui::styles::Resolve::LayoutField;

pub const PHI: f64 = 1.618033988749894848204586834365638118_f64;

type InternalPointer = usize;

pub struct ContinuousBlob {
    data: *mut u8,
    len: usize,
    capacity: usize,
    layout: Layout,
    memmap: BiHashMap<ComponentIdx, InternalPointer>,
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
            memmap: BiHashMap::new(),
            next_idx: 0,
        };

        this
    }

    fn realloc(&mut self) {
        let old_cap = self.capacity;
        self.capacity = (self.capacity as f64 * PHI).ceil() as usize;
        if self.capacity - old_cap < self.layout.size() {
            self.capacity += self.layout.size();
        }
        unsafe {
            self.data = std::alloc::realloc(
                self.data,
                self.layout,
                self.capacity * self.layout.size(),
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
        let idx = *self.memmap.get_by_left(&idx)?;
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
        let idx = *self.memmap.get_by_left(&idx)?;
        if idx < self.len {
            unsafe {
                let added = self.data.add(idx * self.layout.size());
                let typed = added as *mut T;
                return typed.as_mut();
            }
        }
        None
    }

    pub fn get_mut_bruh<T: Sized + 'static>(&self, idx: ComponentIdx) -> Option<&mut T> {
        let idx = *self.memmap.get_by_left(&idx)?;
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
        if let Some((_, idx)) = self.memmap.remove_by_left(&idx) {
            if idx >= self.len {
                return;
            }
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

    pub fn get_all<T: Sized + 'static>(&self) -> impl Iterator<Item=(ComponentIdx, &T)> {
        Iter {
            phantom: Default::default(),
            index: 0,
            blob: self,
        }
    }

    pub fn get_all_mut<T: Sized + 'static>(&self) -> impl Iterator<Item=(ComponentIdx, &mut T)> {
        IterMut {
            phantom: Default::default(),
            index: 0,
            blob: self,
        }
    }
}

impl Drop for ContinuousBlob {
    fn drop(&mut self) {
        unsafe {
            let lay = Layout::from_size_align_unchecked(self.capacity * self.layout.size(), self.layout.align());
            alloc::dealloc(self.data, lay);
        }
    }
}

pub struct Iter<'a, T> {
    phantom: PhantomData<T>,
    index: usize,
    blob: &'a ContinuousBlob
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = (ComponentIdx, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.index >= self.blob.len {
                return None;
            }
            let p = self.index;
            let p = *self.blob.memmap.get_by_right(&p)?;
            let added = self.blob.data.add(self.index * self.blob.layout.size());
            let typed = added as *mut T;
            self.index += 1;
            typed.as_ref().map(|x| (p, x))
        }
    }
}

pub struct IterMut<'a, T> {
    phantom: PhantomData<T>,
    index: usize,
    blob: &'a ContinuousBlob
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = (ComponentIdx, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.index >= self.blob.len {
                return None;
            }
            let p = self.index;
            let p = *self.blob.memmap.get_by_right(&p)?;
            let added = self.blob.data.add(self.index * self.blob.layout.size());
            let typed = added as *mut T;
            self.index += 1;
            typed.as_mut().map(|x| (p, x))
        }
    }
}