use mvengine_ui::elements::UiElement;
use mvengine_ui::styles::UiStyle;
use mvengine_ui::uix::{global_state, use_state, DynamicUi, UiCompoundElement};
use mvutils::state::State;
use mvutils::when;
use uiproc::{ui, uix};

fn some_function(input: i32) -> u32 {
    input as u32 * 2
}

fn get_global_state() -> State<i32> {
    todo!()
}

#[uix]
pub fn MyComponent() -> UiElement {
    let clicks = use_state::<i32>(0);
    let global = global_state::<i32>(get_global_state());

    let shown = use_state::<bool>(false);

    let number = use_state::<i32>(*clicks.read() * 2);

    when!([clicks] => {
        *number.write() = *clicks.read() * 2;
    });

    ui! {
        <Div>
            <Text>{{ format!("Clicked 2x{} times", number) }}</Text>
            <Button onclick={{ *clicks.write() += 1 }}>Click</Button>
        </Div>
    }
}
