use log::debug;
use crate::ui::elements::{Element, UiElementStub};
use crate::ui::styles::{UiStyle, EMPTY_STYLE};

pub struct UniqueSelectLayout {
    elements: Vec<Element>,
    selected: Option<usize>,
    prev_style: Option<UiStyle>,
    selected_style: UiStyle
}

impl UniqueSelectLayout {
    pub fn empty() -> Self {
        Self {
            elements: vec![],
            selected: None,
            prev_style: None,
            selected_style: EMPTY_STYLE.clone(),
        }
    }

    pub fn new(elements: Vec<Element>) -> Self {
        Self {
            elements,
            selected: None,
            prev_style: None,
            selected_style: EMPTY_STYLE.clone(),
        }
    }

    pub fn from_ids(root: Element, ids: &[&str]) -> Self {
        let mut elems = vec![];
        for id in ids {
            if let Some(e) = root.find_element_by_id(id) {
                elems.push(e);
            }
        }
        Self::new(elems)
    }

    pub fn from_collection(root: Element) -> Self {
        Self::new(root.collect_elem_children())
    }

    pub fn set_select_style(&mut self, style: UiStyle) {
        self.selected_style = style;
    }

    pub fn check_events(&mut self) -> Option<usize> {
        for (i, e) in self.elements.iter().enumerate() {
            if e.was_left_clicked() {
                if let Some(prev) = self.selected {
                    //revert style
                    if let Some(p_style) = self.prev_style.take() {
                        let prev_elem = &self.elements[prev];
                        let mut prev_style = prev_elem.get_mut().style_mut();
                        *prev_style = p_style;
                    }
                    if prev == i {
                        //unselect
                        self.selected = None;
                        return Some(i);
                    }
                }
                self.selected = Some(i);
                //set style
                let mut style = e.get_mut().style_mut();
                self.prev_style = Some(style.clone());
                style.merge_at_set_of(&self.selected_style);
                drop(style);
                e.refresh_style();
                return Some(i);
            }
        }
        None
    }

    pub fn selected_idx(&self) -> Option<usize> {
        self.selected
    }

    pub fn selected(&self) -> Option<Element> {
        if let Some(idx) = self.selected {
            Some(self.elements[idx].clone())
        } else {
            None
        }
    }
}