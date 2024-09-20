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
    _cached: mvutils::once::CreateOnce<mvengine_ui::uix::DynamicUi>,
    attributes: mvengine_ui::attributes::Attributes,
    style: mvengine_ui::styles::UiStyle,
    clicks: mvutils::state::State<i32>,
    shown: mvutils::state::State<bool>,
    number: mvutils::state::State<i32>,
    global: mvutils::state::State<i32>,
}
impl mvengine_ui::uix::UiCompoundElement for MyComponent2 {
    fn new(attributes: Option<mvengine_ui::attributes::Attributes>, style: Option<mvengine_ui::styles::UiStyle>) -> Self
    where
        Self: Sized,
    {
        let attributes = attributes.unwrap_or(mvengine_ui::attributes::Attributes::new());
        let style = style.unwrap_or(mvengine_ui::styles::UiStyle::default());
        let clicks = mvutils::state::State::new(0);
        let shown = mvutils::state::State::new(false);

        let number = mvutils::state::State::new(*clicks.read() * 2);

        let global = get_global_state();

        if false {
            use_state::<i32>(0);
            global_state::<i32>(get_global_state());

            use_state::<bool>(false);

            use_state::<i32>(*clicks.read() * 2);
        }
        Self { attributes, style, _cached: mvutils::once::CreateOnce::new(), clicks, shown, number, global }
    }
    fn generate(&self) -> UiElement {
        let clicks = self.clicks.clone();
        let global = self.global.clone();
        let shown = self.shown.clone();
        let number = self.number.clone();
        if clicks.is_outdated() {
            *number.write() = *clicks.read() * 2;
        }

        {
            let mut __attributes_4__ = mvengine_ui::attributes::Attributes::new();
            let __attribs_ref__ = &mut __attributes_4__;
            let mut __div_4__ = Div::new(__attributes_4__, mvengine_ui::styles::UiStyle::default());
            __div_4__.state_mut().children.push(mvengine_ui::elements::child::Child::Element(std::sync::Arc::new(parking_lot::RwLock::new({
                let mut __attributes_5__ = mvengine_ui::attributes::Attributes::new();
                let __attribs_ref__ = &mut __attributes_5__;
                let mut __text_5__ = Text::new(__attributes_5__, mvengine_ui::styles::UiStyle::default());
                __attribs_ref__.with_inner(mvengine_ui::attributes::AttributeValue::Code(Box::new({
                    ::alloc::__export::must_use({
                        let res = ::alloc::fmt::format(::alloc::__export::format_args!("Clicked 2x{} times", number));
                        res
                    })
                })));
                __text_5__.wrap()
            }))));
            __div_4__.state_mut().children.push(mvengine_ui::elements::child::Child::Element(std::sync::Arc::new(parking_lot::RwLock::new({
                let mut __attributes_6__ = mvengine_ui::attributes::Attributes::new();
                __attributes_6__.with_attrib("onclick".to_string(), mvengine_ui::attributes::AttributeValue::Code(Box::new({ *clicks.write() += 1 })));
                let __attribs_ref__ = &mut __attributes_6__;
                let mut __button_6__ = Button::new(__attributes_6__, mvengine_ui::styles::UiStyle::default());
                __attribs_ref__.with_inner(mvengine_ui::attributes::AttributeValue::Str("Click ".to_string()));
                __button_6__.state_mut().children.push(mvengine_ui::elements::child::Child::String("Click ".to_string()));
                __button_6__.wrap()
            }))));   ;
            __div_4__.wrap()
        }
    }
    fn regenerate(&mut self) {
        if (self.clicks).is_outdated() || (self.shown).is_outdated() || (self.number).is_outdated() || (self.global).is_outdated() { self._cached.regenerate(); } else { self._cached.check_children(); }
        (self.clicks).update();
        (self.shown).update();
        (self.number).update();
        (self.global).update();
    }
    fn get(&self) -> &mvengine_ui::elements::UiElement {
        if !self._cached.created() { let _ = self._cached.try_create(|| mvengine_ui::uix::DynamicUi::new(Box::new(|| self.generate()))); }
        self._cached.get_element()
    }
}