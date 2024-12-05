use std::iter::Zip;
use crate::mem::storage::ComponentStorage;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use std::marker::PhantomData;
use std::sync::Arc;

pub struct Components<T: Sized + 'static> {
    data: Option<Vec<&'static T>>,
    index: usize,
}

pub struct ComponentsMut<T: Sized + 'static> {
    data: Option<Vec<&'static mut T>>,
    index: usize,
}

impl<T: Sized + 'static> Iterator for Components<T> {
    type Item = &'static T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref data) = self.data {
            if self.index < data.len() {
                let item = data[self.index];
                self.index += 1;
                return Some(item);
            }
        }
        None
    }
}

impl<T: Sized + 'static> Iterator for ComponentsMut<T> {
    type Item = &'static mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut data) = self.data {
            if self.index < data.len() {
                let item = unsafe { &mut *(&mut *data[self.index] as *mut T) };
                self.index += 1;
                return Some(item);
            }
        }
        None
    }
}

pub struct System<C> {
    phantom: PhantomData<C>,
    storage: Arc<DangerousCell<ComponentStorage>>,
}

impl<C> System<C> {
    pub fn new(storage: Arc<DangerousCell<ComponentStorage>>) -> Self {
        Self {
            phantom: PhantomData::default(),
            storage,
        }
    }

    fn iter_single<T: Sized + 'static>(&self) -> Components<T> {
        let tmp: Arc<DangerousCell<ComponentStorage>> = self.storage.clone();
        let st = unsafe {
            Unsafe::cast_static(tmp.get())
        };
        let fetched: Option<Vec<&T>> = st.get_all_components::<T>();
        Components {
            data: fetched,
            index: 0,
        }
    }

    fn iter_mut_single<T: Sized + 'static>(&mut self) -> ComponentsMut<T> {
        let tmp: Arc<DangerousCell<ComponentStorage>> = self.storage.clone();
        let st = unsafe {
            Unsafe::cast_mut_static(tmp.get_mut())
        };
        let fetched: Option<Vec<&mut T>> = st.get_all_components_mut::<T>();
        ComponentsMut {
            data: fetched,
            index: 0,
        }
    }
}

macro_rules! impl_system_tuples {
    ($first:ident=$struct_name:ident=$struct_name_mut:ident) => {};


    ($first:ident=$struct_name:ident=$struct_name_mut:ident $($rest:ident=$rest_struct_name:ident=$rest_struct_name_mut:ident)*) => {
        impl_system_tuples!($($rest=$rest_struct_name=$rest_struct_name_mut)*);

        pub struct $struct_name<$first: Sized + 'static, $($rest: Sized + 'static),*> {
            $first: Components<$first>,
            $(
                $rest: Components<$rest>,
            )*
        }

        pub struct $struct_name_mut<$first: Sized + 'static, $($rest: Sized + 'static),*> {
            $first: ComponentsMut<$first>,
            $(
                $rest: ComponentsMut<$rest>,
            )*
        }

        impl<$first: Sized + 'static, $($rest: Sized + 'static),*> Iterator for $struct_name<$first, $($rest),*> {
            type Item = (&'static $first, $(&'static $rest),*);

            fn next(&mut self) -> Option<Self::Item> {
                let $first = self.$first.next();
                $(
                    let $rest = self.$rest.next();
                )*
                if $first.is_none() { return None; }

                Some(
                    ($first.unwrap(), $($rest.unwrap()),*)
                )
            }
        }

        impl<$first: Sized + 'static, $($rest: Sized + 'static),*> Iterator for $struct_name_mut<$first, $($rest),*> {
            type Item = (&'static mut $first, $(&'static mut $rest),*);

            fn next(&mut self) -> Option<Self::Item> {
                let $first = self.$first.next();
                $(
                    let $rest = self.$rest.next();
                )*
                if $first.is_none() { return None; }

                Some(
                    ($first.unwrap(), $($rest.unwrap()),*)
                )
            }
        }

        impl<$first: Sized + 'static, $($rest: Sized + 'static),*> System<($first, $($rest),*)> {
            pub fn iter(&self) -> $struct_name<$first, $($rest),*> {
                $struct_name {
                    $first: self.iter_single::<$first>(),
                    $(
                        $rest: self.iter_single::<$rest>(),
                    )*
                }
            }

            pub fn iter_mut(&mut self) -> $struct_name_mut<$first, $($rest),*> {
                $struct_name_mut {
                    $first: self.iter_mut_single::<$first>(),
                    $(
                        $rest: self.iter_mut_single::<$rest>(),
                    )*
                }
            }
        }
    };
}

impl_system_tuples!(
    C15=Components15=ComponentsMut15
    C14=Components14=ComponentsMut14
    C13=Components13=ComponentsMut13
    C12=Components12=ComponentsMut12
    C11=Components11=ComponentsMut11
    C10=Components10=ComponentsMut10
    C9=Components9=ComponentsMut9
    C8=Components8=ComponentsMut8
    C7=Components7=ComponentsMut7
    C6=Components6=ComponentsMut6
    C5=Components5=ComponentsMut5
    C4=Components4=ComponentsMut4
    C3=Components3=ComponentsMut3
    C2=Components2=ComponentsMut2
    C1=Components1=ComponentsMut1
);

pub struct Components1<C1: Sized + 'static> {
    C1: Components<C1>,
}

pub struct ComponentsMut1<C1: Sized + 'static> {
    C1: ComponentsMut<C1>,
}

impl<C1: Sized + 'static> Iterator for Components1<C1> {
    type Item = (&'static C1);

    fn next(&mut self) -> Option<Self::Item> {
        let C1 = self.C1.next();
        if C1.is_none() { return None; }

        Some(
            (C1.unwrap())
        )
    }
}
impl<C1: Sized + 'static> Iterator for ComponentsMut1
<C1> {
    type Item = (&'static mut C1);

    fn next(&mut self) -> Option<Self::Item> {
        let C1 = self.C1.next();
        if C1.is_none() { return None; }

        Some(
            (C1.unwrap())
        )
    }
}
impl<C1: Sized + 'static> System<(C1,)> {
    pub fn iter(&self) -> Components1<C1> {
        Components1 {
            C1: self.iter_single::<C1>(),
        }
    }

    pub fn iter_mut(&mut self) -> ComponentsMut1
    <C1> {
        ComponentsMut1
        {
            C1: self.iter_mut_single::<C1>(),
        }
    }
}