use std::any::Any;
use mvutils::state::State;
use mvutils::unsafe_utils::Unsafe;
use crate::elements::UiElement;

pub struct DynamicUi {
    cached: UiElement,
    _when_call: Box<dyn FnMut(&mut DynamicUi)>,
}

impl DynamicUi {
    pub fn new(cached: UiElement, _when_call: Box<dyn FnMut(&mut DynamicUi)>) -> Self {
        Self {
            cached,
            _when_call,
        }
    }

    ///Returns the underlying element and recreate its tree, if one of its dependent states have changed.
    pub fn get_element(&mut self) -> &mut UiElement {
        unsafe {
            let mut static_self = (self as *mut Self).as_mut().unwrap();
            (self._when_call)(static_self);
            &mut self.cached
        }
    }

    ///Returns the underlying element which can be outdated
    pub fn get_cache(&mut self) -> &mut UiElement {
        &mut self.cached
    }

    ///This will return the underlying UiElement. Keep in mind that this removes and dynamic functionality with states.
    pub fn to_static(mut self) -> UiElement {
        unsafe {
            let mut static_self = (&mut self as *mut Self).as_mut().unwrap();
            (self._when_call)(static_self);
            self.cached
        }
    }
}

///creates a new State\<T\> object which can be used inside the ui! macro. Please preserve the form
/// ```use_state::<T>(init)```
/// <br>
/// example:<br>
/// ```
/// #[uix]
/// pub fn TestComponent() {
///     let test_state = use_state::<i32>(5);
/// }
/// ```
pub fn use_state<T>(init: T) {}

pub trait UiCompoundElement {
    fn new() -> Self where Self: Sized;

    fn get_dyn_ui(&mut self) -> &mut DynamicUi;
}