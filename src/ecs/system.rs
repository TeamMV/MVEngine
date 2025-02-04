use crate::ecs::EcsStorage;
use std::marker::PhantomData;
use std::mem;
use crate::ecs::entity::EntityType;

pub struct System<C> {
    phantom: PhantomData<C>,
    storage: EcsStorage,
}

impl<C> System<C> {
    pub fn new(storage: EcsStorage) -> Self {
        Self {
            phantom: PhantomData::default(),
            storage,
        }
    }
}

pub struct Components<'a, C> {
    phantom: PhantomData<&'a C>,
    data: Vec<(EntityType, &'a C)>,
    index: usize,
}

pub struct ComponentsMut<'a, C> {
    phantom: PhantomData<&'a mut C>,
    data: Vec<(EntityType, &'a mut C)>,
    index: usize,
}

impl<'a, C> Iterator for Components<'a, C> {
    type Item = (EntityType, &'a C);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let (entity, component) = &self.data[self.index];
            self.index += 1;
            Some((*entity, component))
        } else {
            None
        }
    }
}

impl<'a, C> Iterator for ComponentsMut<'a, C> {
    type Item = (EntityType, &'a mut C);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let (entity, component) = &mut self.data[self.index];
            self.index += 1;
            unsafe { Some((*entity, mem::transmute::<_, &'a mut C>(component))) }
        } else {
            None
        }
    }
}

impl<C: Sized + 'static> System<(C,)> {
    pub fn iter(&self) -> Components<'_, C> {
        let data = self.storage.get().get_components1::<C>().unwrap_or_default();
        Components {
            data,
            index: 0,
            phantom: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> ComponentsMut<'_, C> {
        let data = self.storage.get_mut().get_components1_mut::<C>().unwrap_or_default();
        ComponentsMut {
            data,
            index: 0,
            phantom: PhantomData,
        }
    }
}

macro_rules! impl_system_tuples {
    ($($name:ident)*, $method:ident, $method_mut:ident, $iterator:ident, $iterator_mut:ident) => {
        pub struct $iterator<'a, $($name: Sized + 'static),*> {
            data: Vec<(EntityType, $(&'a $name),*)>,
            index: usize,
            _marker: PhantomData<&'a ($($name),*)>,
        }

        pub struct $iterator_mut<'a, $($name: Sized + 'static),*> {
            data: Vec<(EntityType, $(&'a mut $name),*)>,
            index: usize,
            _marker: PhantomData<&'a mut ($($name),*)>,
        }

        impl<'a, $($name: Sized + 'static),*> Iterator for $iterator<'a, $($name),*> {
            type Item = (EntityType, $(&'a $name),*);

            fn next(&mut self) -> Option<Self::Item> {
                if self.index < self.data.len() {
                    let (entity, $($name),*) = &self.data[self.index];
                    self.index += 1;
                    Some((*entity, $($name),*))
                } else {
                    None
                }
            }
        }

        impl<'a, $($name: Sized + 'static),*> Iterator for $iterator_mut<'a, $($name),*> {
            type Item = (EntityType, $(&'a mut $name),*);

            fn next(&mut self) -> Option<Self::Item> {
                if self.index < self.data.len() {
                    let (entity, $($name),*) = &mut self.data[self.index];
                    self.index += 1;
                    unsafe { Some((*entity, $(mem::transmute::<_, &mut $name>($name)),*)) }
                } else {
                    None
                }
            }
        }

        impl<$($name: Sized + 'static),*> System<($($name),*)> {
            pub fn iter(&self) -> $iterator<'_, $($name),*> {
                let data = self
                    .storage
                    .get()
                    .$method::<$($name),*>()
                    .unwrap_or_default();
                $iterator {
                    data,
                    index: 0,
                    _marker: PhantomData,
                }
            }

            pub fn iter_mut(&mut self) -> $iterator_mut<'_, $($name),*> {
                let data = self
                    .storage
                    .get_mut()
                    .$method_mut::<$($name),*>()
                    .unwrap_or_default();
                $iterator_mut {
                    data,
                    index: 0,
                    _marker: PhantomData,
                }
            }
        }
    };
}

//impl_system_tuples!(C1, get_components1, get_components1_mut, Components1, Components1Mut);
impl_system_tuples!(C1 C2, get_components2, get_components2_mut, Components2, Components2Mut);
impl_system_tuples!(C1 C2 C3, get_components3, get_components3_mut, Components3, Components3Mut);
impl_system_tuples!(C1 C2 C3 C4, get_components4, get_components4_mut, Components4, Components4Mut);
impl_system_tuples!(C1 C2 C3 C4 C5, get_components5, get_components5_mut, Components5, Components5Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6, get_components6, get_components6_mut, Components6, Components6Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6 C7, get_components7, get_components7_mut, Components7, Components7Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6 C7 C8, get_components8, get_components8_mut, Components8, Components8Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6 C7 C8 C9, get_components9, get_components9_mut, Components9, Components9Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6 C7 C8 C9 C10, get_components10, get_components10_mut, Components10, Components10Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11, get_components11, get_components11_mut, Components11, Components11Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12, get_components12, get_components12_mut, Components12, Components12Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13, get_components13, get_components13_mut, Components13, Components13Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14, get_components14, get_components14_mut, Components14, Components14Mut);
impl_system_tuples!(C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14 C15, get_components15, get_components15_mut, Components15, Components15Mut);