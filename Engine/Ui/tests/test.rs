use mvengine_ui::elements::{UiElement, UiElementStub};
use mvengine_ui::styles::UiStyle;
use mvengine_ui::uix::{global_state, use_state, UiCompoundElement};
use mvutils::state::State;
use mvutils::{lazy, update, when};
use mvcore::color::utils::Colors;
use mvengine_ui::attributes::{AttributeValue, Attributes};
use mvengine_ui::elements::div::Div;
use mvengine_ui::uix::dom::{VComponent, VElement, VNode};
use uiproc::{ui, uix};

pub fn run() {
    let key = String::new();
    let elem = {
        let mut attributes = Attributes::new();
        let mut style = UiStyle::default();
        let mut elem = VElement::new(attributes, style, format!("{}_div1", key), |a, s| Div::new(a, s).wrap(), "Div".to_string());

        elem.add_child({
            let mut attributes = Attributes::new();
            let mut style = UiStyle::default();
            let mut elem = VElement::new(
                attributes,
                style,
                format!("{}_text1", key),
                |a, s| Div::new(a, s).wrap(), // pretend this is text because we dont have this component
                "Text".to_string(),
            );
            VNode::Element(elem)
        });
        elem.add_child({
            // TODO: make attributes and style either copy_from(&mut self, &Self), or clone
            let mut attributes = Attributes::new();
            let mut style = UiStyle::default();

            let mut attributes2 = Attributes::new();
            let mut style2 = UiStyle::default();

            let mut elem = VComponent::new(attributes, style, Box::new(GeneratedComponent::new(attributes2, style2, format!("{}_generatedcomponent1", key))), format!("{}_generatedcomponent1", key), "GeneratedComponent".to_string());

            VNode::Component(elem)
        });

        VNode::Element(elem)
    };
    //println!("{:?}", elem);
    dbg!(&elem);

    let elem = elem.expand();

    println!("\n\n\n\nEXPANDED!\n\n\n\n");
    //println!("{:?}", elem);
    dbg!(&elem);
}

fn some_function(input: i32) -> u32 {
    input as u32 * 2
}

lazy! {
    static STATE: State<i32> = State::new(10);
}

fn get_global_state() -> State<i32> {
    STATE.clone()
}

// #[uix]
// pub fn MyComponent() -> UiElement {
//     let clicks = use_state::<i32>(0);
//     let global = global_state::<i32>(get_global_state());
//
//     when!([] => {
//         println!("Only runs on first component generation");
//     });
//
//     when!([clicks] => {
//         *global.write() = *clicks.read() * 2;
//     });
//
//     when!([global] => {
//         if *global.read() == 10 {
//             self.request_regenerate();
//         }
//     });
//
//     ui! {
//         <Div>
//             <Text>{{ format!("Clicked 2x{} times", *gloabal.read()) }}</Text>
//             <Button onclick={{ *clicks.write() += 1 }}>Click</Button>
//         </Div>
//     }
// }

struct GeneratedComponent {
    key: String,
    attributes: Attributes,
    style: UiStyle,
    empty: State<()>,
    redraw: State<()>,
    clicks: State<i32>,
    global: State<i32>,
}

impl UiCompoundElement for GeneratedComponent {
    fn new(attributes: Attributes, style: UiStyle, key: String) -> Self
    where
        Self: Sized
    {
        let empty = State::new(());
        empty.force_outdated();

        let redraw = State::new(());

        let clicks = State::new(0);
        let global = get_global_state();

        if false {
            use_state::<i32>(0);
            global_state::<i32>(get_global_state());

            use_state::<bool>(false);

            use_state::<i32>(*clicks.read() * 2);
        }

        Self {
            key,
            attributes,
            style,
            empty,
            redraw,
            clicks,
            global,
        }
    }

    fn generate(&self) -> VNode {
        let clicks = self.clicks.clone();
        let global = self.global.clone();

        when!([self.empty] => {
            println!("Only runs on first component generation");
        });

        when!([clicks] => {
            *global.write() = *clicks.read() * 2;
        });

        when!([global] => {
            if *global.read() == 10 {
                self.request_regenerate();
            }
        });

        let mut attributes = Attributes::new();
        let mut style = UiStyle::default();
        let mut elem = VElement::new(attributes, style, format!("{}_div1", self.key), |a, s| Div::new(a, s).wrap(), "Div".to_string());

        elem.add_child({
            let mut attributes = Attributes::new();
            let mut style = UiStyle::default();
            let mut elem = VElement::new(
                attributes,
                style,
                format!("{}_text1", self.key),
                |a, s| Div::new(a, s).wrap(), // pretend this is text because we dont have this component
                "Text".to_string(),
            );
            elem.add_child({
                let text = { format!("Clicked 2x{} times", *global.read()) };
                VNode::Text(text)
            });
            VNode::Element(elem)
        });
        elem.add_child({
            let mut attributes = Attributes::new();
            let mut style = UiStyle::default();
            let onclick = {
                let clicks = clicks.clone();
                move |_: &mut UiElement| *clicks.write() = 1
            };
            attributes.with_attrib("onclick".to_string(), AttributeValue::Code(Box::new(onclick)));
            let mut elem = VElement::new(
                attributes,
                style,
                format!("{}_button1", self.key),
                |a, s| Div::new(a, s).wrap(), // pretend this is button because we dont have this component
                "Button".to_string(),
            );
            elem.add_child({
                let text = "Click".to_string();
                VNode::Text(text)
            });
            VNode::Element(elem)
        });
        VNode::Element(elem)
    }

    fn post_generate(&mut self) {
        update!([self.clicks, self.global, self.empty]);
    }

    fn regenerate(&mut self) -> bool {
        when!([self.clicks, self.global, self.redraw] => {
            update!([self.redraw]);
            true
        } else {
            false
        })
    }

    fn request_regenerate(&self) {
        self.redraw.force_outdated();
    }

    fn update_style(&mut self, style: UiStyle) {
        self.style = style;
        self.request_regenerate();
    }

    fn update_attributes(&mut self, attributes: Attributes) {
        self.attributes = attributes;
        self.request_regenerate();
    }
}