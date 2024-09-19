use mvengine_ui::elements::UiElement;
use mvengine_ui::styles::UiStyle;
use mvengine_ui::uix::{use_state, DynamicUi};
use uiproc::{ui, uix};

fn some_function(input: i32) -> u32 {
    input as u32 * 2
}

#[uix]
pub fn MyComponent() -> UiElement {
    let clicks = use_state::<i32>(0);

    let number = clicks * 2;

    ui! {
        <Div>
            <Text>{{ format!("Clicked 2x{} times", number) }}</Text>
            <Button onclick={{ *clicks.write() += 1 }}>Click</Button>
        </Div>
    }
}