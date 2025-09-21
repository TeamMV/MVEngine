use crate::modify_style;
use mvengine_proc_macro::{style_expr, ui};
use mvutils::lazy;
use mvengine_proc_macro::multiline_str_into;
use crate::ui::elements::prelude::*;
use crate::ui::layouts::Adapter;
use crate::ui::styles::UiStyle;

use crate as mvengine;
use crate::ui::context::UiContext;

lazy! {
    static ROW_STYLE: UiStyle = multiline_str_into!(style_expr, {
        background: none;
        direction: horizontal;
        margin: none;
        padding: none;
    });
}

pub struct FlowLayout {
    all_elements: Vec<Element>,
    root: Element
}

impl FlowLayout {
    pub fn from_adapter<A: Adapter>(mut root: Element, elements_across: usize, mut adapter: A) -> Self {
        let mut i = 0;
        let ctx = root.get().context().clone();

        let mut row = Self::create_row(ctx.clone());
        let mut all_elements = vec![];

        while let Some(elem) = adapter.create_element(ctx.clone()) {
            all_elements.push(elem.clone());
            row.add_child(elem.to_child());
            i += 1;
            if i == elements_across {
                i = 0;
                //next row
                root.add_child(row.to_child());
                row = Self::create_row(ctx.clone());
            }
        }
        
        Self {
            all_elements,
            root,
        }
    }

    fn create_row(ctx: UiContext) -> Element {
        ui! {
            <Ui context={ctx}>
                <Div style={ROW_STYLE.clone()}/>
            </Ui>
        }
    }
}