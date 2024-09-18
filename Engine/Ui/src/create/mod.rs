use std::any::Any;
use mvutils::state::State;
use crate::elements::UiElement;

pub struct DynamicUi {
    cached: UiElement,
    _when_call: Box<dyn FnMut(&mut DynamicUi)>
}

impl DynamicUi {
    pub fn new(cached: UiElement, _when_call: Box<dyn FnMut(&mut DynamicUi)>) -> Self {
        Self {
            cached,
            _when_call,
        }
    }

    ///Returns the underlying element and recreate its tree, if one if its dependent states have changed.
    pub fn get_element(&mut self) -> &mut UiElement {
        (self._when_call)(self);
        &mut self.cached
    }

    ///This will return the underlying UiElement. Keep in mind that this removes and dynamic functionality with states.
    pub fn to_static(mut self) -> UiElement {
        (&self._when_call)(&mut self);
        self.cached
    }
}