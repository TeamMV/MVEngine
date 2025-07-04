use crate::color::{Color, ColorFormat, RgbColor};
use crate::ui::elements::{UiElementState, UiElementStub};
use crate::ui::styles::{Resolve, ResolveResult, UiStyle};
#[macro_export]
macro_rules! blanked_partial_ord {
    ($t:ty) => {
        impl PartialEq<Self> for $t {
            fn eq(&self, _: &Self) -> bool {
                false //Like you would never ever use this, it is just required ._.
            }
        }

        impl PartialOrd for $t {
            fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
                None
            }
        }
    };
}

#[macro_export]
macro_rules! fast_partial_ord {
    ($t:ty, {$($impl1:tt)*}, {$($impl2:tt)*}) => {
        impl PartialEq<Self> for $t {
            fn eq(&self, other: &Self) -> bool {
                $($impl1)*
            }
        }

        impl PartialOrd for $t {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                $($impl2)*
            }
        }
    };
}

pub fn resolve_color<F, Format: ColorFormat>(
    res: &Resolve<Color<Format>>,
    def: Color<Format>,
    state: &UiElementState,
    f: F,
) -> Color<Format>
where
    F: Fn(&UiStyle) -> &Resolve<Color<Format>>,
    Format::ComponentType: PartialOrd<Format::ComponentType>,
{
    let resolved = res.resolve(state.ctx.dpi, state.parent.clone(), f);
    let mut ret = def;
    if resolved.is_none() {
        if res.is_none() {
            ret = Format::from_rgb(RgbColor::transparent());
        }
    } else {
        ret = resolved.unwrap();
    }
    ret
}

pub struct AnyType {
    pub(crate) pointer: *const (),
}

impl AnyType {
    pub fn new<T: Sized>(t: &T) -> Self {
        Self {
            pointer: (t as *const T as *const ()),
        }
    }

    pub fn try_get<T: Sized>(&self) -> Option<&'static T> {
        unsafe {
            let ref_opt = (self.pointer as *const T).as_ref();
            if ref_opt.is_some() {
                return Some(ref_opt.unwrap());
            }
            return None;
        }
    }
}

impl Clone for AnyType {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer,
        }
    }
}

pub trait OptionGetMapOr<T> {
    fn get_map_or<R>(&self, mapper: fn(&T) -> R, def: R) -> R;
}

impl<T> OptionGetMapOr<T> for Option<T> {
    fn get_map_or<R>(&self, mapper: fn(&T) -> R, def: R) -> R {
        if self.is_some() {
            return mapper(self.as_ref().unwrap());
        }
        def
    }
}

#[macro_export]
macro_rules! expect_inner_element_by_id {
    ($parent:expr, $elem_type:ident, $id:literal, $var:ident => $body:block) => {{
        let __e = $parent
            .get()
            .find_element_by_id($id)
            .expect(&format!("Element '{}' is not on parent!", $id));
        let __inner = __e.get();
        let $var = enum_val_ref!(UiElement, __inner, $elem_type);
        $body
    }};
}

#[macro_export]
macro_rules! expect_inner_element_by_id_mut {
    ($parent:expr, $elem_type:ident, $id:literal, $var:ident => $body:block) => {{
        let __e = $parent
            .get()
            .find_element_by_id($id)
            .expect(&format!("Element '{}' is not on parent!", $id));
        let __inner = __e.get_mut();
        let $var = enum_val_ref_mut!(UiElement, __inner, $elem_type);
        $body
    }};
}

#[macro_export]
macro_rules! expect_element_by_id {
    ($parent:expr, $id:literal) => {{
        let e = $parent
            .get()
            .find_element_by_id($id)
            .expect(&format!("Element '{}' is not on parent!", $id));
        e
    }};
}

#[macro_export]
macro_rules! find_inner_element_by_id {
    ($parent:expr, $elem_type:ident, $id:literal, $var:ident => $body:block) => {{
        if let Some(__e) = $parent.get().find_element_by_id($id) {
            let __inner = __e.get();
            if let $crate::UiElement::$elem_type($var) = __inner {
                Some($body)
            } else {
                None
            }
        } else {
            None
        }
    }};
}

#[macro_export]
macro_rules! find_inner_element_by_id_mut {
    ($parent:expr, $elem_type:ident, $id:literal, $var:ident => $body:block) => {{
        if let Some(__e) = $parent.get().find_element_by_id($id) {
            let __inner = __e.get_mut();
            if let $crate::UiElement::$elem_type($var) = __inner {
                Some($body)
            } else {
                None
            }
        } else {
            None
        }
    }};
}

#[macro_export]
macro_rules! find_element_by_id {
    ($parent:expr, $id:literal) => {{ $parent.get().find_element_by_id($id) }};
}

pub fn resolve_resolve<T, E, F>(res: &Resolve<T>, elem: &E, map: F) -> ResolveResult<T>
where
    T: PartialOrd + Clone + 'static,
    E: UiElementStub + 'static,
    F: Fn(&UiStyle) -> &Resolve<T>,
{
    res.resolve(elem.state().ctx.dpi, elem.state().parent.clone(), |f| {
        map(f)
    })
}
