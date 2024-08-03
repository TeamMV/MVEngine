use std::sync::Arc;
use parking_lot::RwLock;
use crate::render::color::RgbColor;
use crate::ui::elements::{UiElement, UiElementState};
use crate::ui::styles::{Resolve, UiStyle};
#[macro_export]
macro_rules! blanked_partial_ord {
    ($t:ty) => {
        impl PartialEq<Self> for $t {
            fn eq(&self, other: &Self) -> bool {
                false //Like you would never ever use this, it is just required ._.
            }
        }

        impl PartialOrd for $t {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
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

pub fn resolve_color<F>(res: &Resolve<RgbColor>, def: RgbColor, state: &UiElementState, f: F) -> RgbColor where F: Fn(&UiStyle) -> &Resolve<RgbColor> {
    let resolved = res.resolve(state.ctx.dpi, state.parent.clone(), f);
    let mut ret = def;
    if resolved.is_none() {
        if res.is_none() { ret = RgbColor::transparent(); }
    } else {
        ret = resolved.unwrap();
    }
    ret
}