use crate::attributes::Attributes;
use crate::elements::UiElement;
use crate::styles::UiStyle;
use crate::uix::UiCompoundElement;

pub enum VNode {
    Element(VElement),
    Text(String),
    Component(VComponent),
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

pub struct VComponent {
    attributes: Attributes,
    style: UiStyle,
    component: Box<dyn UiCompoundElement>,
    key: String,
    subtree: Option<Box<VNode>>,
    tag: String,
}

impl VComponent {
    pub fn new(attributes: Attributes, style: UiStyle, component: Box<dyn UiCompoundElement>, key: String, subtree: Option<Box<VNode>>, tag: String) -> Self {
        Self { attributes, style, component, key, subtree, tag }
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