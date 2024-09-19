use mvutils::state::State;
use crate::attributes::Attributes;
use crate::elements::{UiElement, UiElementStub};
use crate::elements::child::Child;
use crate::styles::UiStyle;

pub struct DynamicUi {
    cached: UiElement,
    generator: Box<dyn FnMut() -> UiElement>,
}

impl DynamicUi {
    pub fn new(generator: Box<dyn FnMut() -> UiElement>) -> Self {
        Self {
            cached: generator(),
            generator,
        }
    }

    /// Returns the cached element
    pub fn get_element(&mut self) -> &mut UiElement {
        &mut self.cached
    }

    /// This will return the underlying UiElement. Keep in mind that this removes and dynamic functionality with states.
    pub fn to_static(mut self) -> UiElement {
        self.cached
    }

    pub fn regenerate(&mut self) {
        self.cached = self.generator();
    }

    pub fn check_children(&mut self) {
        for child in self.cached.children_mut() {
            if let Child::Element(e) = child {
                let guard = e.write();
                if let UiElement::Compound(compound) = guard {
                    compound.get_mut().regenerate();
                } else {
                    guard.
                }
            }
        }
    }
}

/// Creates a new State\<T\> object which can be used inside the ui! macro. Please preserve the form
/// ```use_state::<T>(init)```
/// <br>
/// example:<br>
/// ```
/// #[uix]
/// pub fn TestComponent() {
///     let test_state = use_state::<i32>(5);
/// }
/// ```
pub fn use_state<T>(init: T) -> State<T> { panic!("You can only use use_state() in an uix function!") }

/// Imports an existing State\<T\> object which can be used inside the ui! macro. Please preserve the form
/// ```global_state::<T>(init)```
/// <br>
/// example:<br>
/// ```
/// static STATE: State<i32> = State::new(5);
///
/// #[uix]
/// pub fn TestComponent() {
///     let test_state = global_state::<i32>(STATE);
/// }
/// ```
pub fn global_state<T>(init: State<T>) -> State<T> { panic!("You can only use use_state() in an uix function!") }

pub trait UiCompoundElement {
    fn new(attributes: Option<Attributes>, style: Option<UiStyle>) -> Self where Self: Sized;

    fn generate(&self) -> UiElement;

    fn regenerate(&mut self);
}