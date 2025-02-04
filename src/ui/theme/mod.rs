use std::hash::BuildHasher;
use hashbrown::HashMap;
use mvutils::hashers::U64IdentityHasher;
use crate::ui::styles::{UiStyle, DEFAULT_STYLE};

pub const CATEGORY_BUTTON_CLICK: u64 = 1;
pub const CATEGORY_TEXT_BASIC: u64 = 1 << 1;
pub const CATEGORY_TEXT_CLICK: u64 = 1 << 2;
pub const CATEGORY_CONTAINER_BACKGROUND: u64 = 1 << 3;

pub struct UiTheme {
    styles: HashMap<u64, UiStyle, U64IdentityHasher>,
    fallback: UiStyle
}

impl UiTheme {
    pub fn new(style: UiStyle) -> Self {
        Self {
            styles: HashMap::with_hasher(U64IdentityHasher::default()),
            fallback: style,
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            styles: HashMap::with_hasher(U64IdentityHasher::default()),
            fallback: DEFAULT_STYLE.clone(),
        }
    }
}