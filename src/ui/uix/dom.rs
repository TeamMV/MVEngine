use std::fmt::Debug;
use crate::ui::attributes::Attributes;
use crate::ui::elements::UiElement;
use crate::ui::styles::UiStyle;
use crate::ui::uix::UiCompoundElement;

#[derive(Debug)]
pub enum VNode {
    Element(VElement),
    Text(String),
    Component(VComponent),
}

impl VNode {
    pub fn expand(self) -> VNode {
        match self {
            VNode::Element(mut e) => {
                e.children = e.children.into_iter().map(VNode::expand).collect();
                VNode::Element(e)
            }
            VNode::Text(t) => VNode::Text(t),
            VNode::Component(c) => c.into_subtree(),
        }
    }
}

pub struct VElement {
    attributes: Attributes,
    style: UiStyle,
    children: Vec<VNode>,
    key: String,
    constructor: fn(Attributes, UiStyle) -> UiElement,
    tag: String,
}

impl VElement {
    pub fn new(attributes: Attributes, style: UiStyle, key: String, constructor: fn(Attributes, UiStyle) -> UiElement, tag: String) -> Self {
        Self { attributes, style, children: vec![], key, constructor, tag }
    }

    pub fn add_child(&mut self, child: VNode) {
        self.children.push(child);
    }
}

impl Debug for VElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VElement")
            .field("key", &self.key)
            .field("tag", &self.tag)
            .field("children", &self.children)
            .finish()
    }
}

pub struct VComponent {
    attributes: Attributes,
    style: UiStyle,
    component: Box<dyn UiCompoundElement>,
    key: String,
    subtree: Option<Box<VNode>>,
    tag: String,
}

impl VComponent {
    pub fn new(attributes: Attributes, style: UiStyle, component: Box<dyn UiCompoundElement>, key: String, tag: String) -> Self {
        Self { attributes, style, component, key, subtree: None, tag }
    }

    pub fn generate_subtree(&mut self) {
        if self.subtree.is_some() {
            return;
        }

        // TODO: make attributes and style either copy_from(&mut self, &Self), or clone
        // TODO: update styles and attributes, just in case
        self.subtree = Some(Box::new(self.component.generate()))
    }

    pub fn subtree_generated(&self) -> bool {
        self.subtree.is_some()
    }

    pub fn should_regenerate(&mut self) -> bool {
        self.component.regenerate()
    }

    pub fn regenerate_subtree(&mut self) -> VNode {
        let node = self.component.generate();
        self.component.post_generate();
        node
    }

    pub fn take_subtree(&mut self) -> Box<VNode> {
        self.subtree.take().unwrap()
    }

    pub fn set_subtree(&mut self, subtree: Box<VNode>) {
        self.subtree = Some(subtree);
    }

    pub fn update_attributes(&mut self, attributes: Attributes) {
        // TODO: if attributes haven't changed, don't do anything
        // if self.attributes == attributes { return; }
        self.attributes = attributes;
        // TODO: clone attributes to the custom component
        // self.component.update_attributes(attributes);
    }

    pub fn update_style(&mut self, style: UiStyle) {
        // TODO: if style hasn't changed, don't do anything
        // if self.style == style { return; }
        self.style = style;
        // TODO: clone style to the custom component
        // self.component.update_style(style);
    }

    pub fn into_subtree(mut self) -> VNode {
        self.generate_subtree();
        self.subtree.unwrap().expand()
    }
}

impl Debug for VComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VComponent")
            .field("key", &self.key)
            .field("tag", &self.tag)
            .field("subtree", &self.subtree)
            .finish()
    }
}

pub struct VTransaction<'dom> {
    path: Vec<usize>,
    tx: VTransactionType<'dom>,
}

pub enum VTransactionType<'dom> {
    CreateNode { node: &'dom VNode },
    DeleteNode,
    ReplaceNode { new_node: &'dom VNode },
    UpdateAttributes { attributes: Attributes },
    UpdateStyle { style: UiStyle },
    UpdateTextContent { text: String },
    MoveNode { to: Vec<usize> },
}