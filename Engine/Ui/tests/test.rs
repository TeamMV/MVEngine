use mvengine_ui::uix::{use_state, DynamicUi};
use uiproc::{ui, uix};

#[uix]
pub fn MyComponent() {
    let clicks = use_state::<i32>(0);

    return ui!([clicks] => {
        <Div>
            <Text>{{ format!("Clicked {} times", clicks) }}</Text>
            <Button onclick={{ *clicks.write() += 1 }}>Click</Button>
        </Div>
    });
}