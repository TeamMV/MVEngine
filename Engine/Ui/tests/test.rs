use mvengine_ui::elements::UiElement;
use mvengine_ui::styles::UiStyle;
use mvengine_ui::uix::{global_state, use_state, DynamicUi, UiCompoundElement};
use mvutils::state::State;
use mvutils::{update, when};
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

pub struct MyComponent2 {
    _cached: mvutils::once::CreateOnce<DynamicUi>,
    attributes: mvengine_ui::attributes::Attributes,
    style: UiStyle,
    clicks: State<i32>,
    shown: State<bool>,
    number: State<i32>,
    global: State<i32>,
}
impl UiCompoundElement for MyComponent2 {
    fn new(attributes: Option<mvengine_ui::attributes::Attributes>, style: Option<UiStyle>) -> Self
    where
        Self: Sized,
    {
        let attributes = attributes.unwrap_or(mvengine_ui::attributes::Attributes::new());
        let style = style.unwrap_or(UiStyle::default());
        let clicks = State::new(0);
        let shown = State::new(false);

        let number = State::new(*clicks.read() * 2);

        let global = get_global_state();

        Self { attributes, style, _cached: mvutils::once::CreateOnce::new(), clicks, shown, number, global }
    }

    fn generate(&self) -> UiElement {
        let clicks = self.clicks.clone();
        let global = self.global.clone();
        let shown = self.shown.clone();
        let number = self.number.clone();
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
    fn regenerate(&mut self) {
        when!([self . clicks , self . shown , self . number , self . global , ] => {
            self . _cached . regenerate ();
        } else {
            self . _cached . check_children ();
        });
        update!([self . clicks , self . shown , self . number , self . global , ]);
    }
    fn get(&self) -> &UiElement {
        if !self._cached.created() { let _ = self._cached.try_create(|| DynamicUi::new(Box::new(|| self.generate()))); }
        self._cached.get_element()
    }
}
