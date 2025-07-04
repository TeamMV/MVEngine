use crate::ui::elements::{Element, UiElementStub};
use crate::ui::styles::UiStyle;

pub mod parse;

pub struct StyleSheet {
    blocks: Vec<StyleBlock>,
}

impl StyleSheet {
    pub fn try_apply(&self, elem: Element) {
        let thingy = elem.get_mut();
        for block in &self.blocks {
            let cond = &block.cond;
            let matches = match cond {
                StyleCondition::Type(t) => thingy.attributes().elem_type == t.as_str(),
                StyleCondition::Class(c) => thingy.attributes().classes.contains(c),
                StyleCondition::Id(i) => thingy
                    .attributes()
                    .id
                    .as_ref()
                    .is_some_and(|s| s == i.as_str()),
            };
            if matches {
                let style = thingy.style_mut();
                style.merge_unset(&block.style);
            }
        }
    }
}

pub enum StyleCondition {
    Type(String),
    Class(String),
    Id(String),
}

pub struct StyleBlock {
    cond: StyleCondition,
    style: UiStyle,
}
