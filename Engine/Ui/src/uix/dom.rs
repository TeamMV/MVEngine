use std::fmt::Debug;
use crate::attributes::Attributes;
use crate::elements::UiElement;
use crate::styles::UiStyle;
use crate::uix::UiCompoundElement;

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

    pub fn regenerate_subtree(&self) -> VNode {
        todo!()
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