use mvutils::unsafe_utils::Unsafe;
use mvengine_ecs::component::{Component, ComponentBase};
use mvengine_ecs::{Behavior, ECS};
use mvengine_ecs::entity::{Entity, EntityBase};
use mvengine_ecs::system::{JoinedSystem, System, SystemBase, SystemBehavior};

fn main() {
    let mut ecs = ECS::new();

    let comp = MyComponent::new();

    let mut my_ent = MyEntity::new();
    my_ent.base.add_component(&comp);

    ecs.insert_entity(my_ent);

    let mut my_sys = MySystem::new();
    ecs.insert_system(my_sys);

    ecs.init();

    ecs.update();
}

struct MySystem {
    base: SystemBase
}

impl MySystem {
    pub fn new() -> Self {
        Self {
            base: SystemBase::Joined(JoinedSystem::new(20)),
        }
    }
}

impl SystemBehavior for MySystem {
    fn init(&mut self) {

    }

    fn check_entity(&self, en: &Box<(dyn Entity + Send + Sync)>) {
        let comp = en.get_base().get_component::<MyComponent>().unwrap(); //since we explicitly say that we want MyCOmponent to be part of the Entity
        println!("String: {}", comp.some_str);
    }
}

impl System for MySystem {
    fn get_base(&self) -> &SystemBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut SystemBase {
        &mut self.base
    }

    fn entity_valid(&self, en: &EntityBase) -> bool {
        en.has_component::<MyComponent>()
    }
}



struct MyComponent {
    base: ComponentBase,
    some_str: String
}

impl MyComponent {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new("MyComponent"),
            some_str: "hello".to_string(),
        }
    }
}

impl Component for MyComponent {
    fn get_base(&self) -> &ComponentBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}

struct MyEntity {
    base: EntityBase,
}

impl MyEntity {
    pub fn new() -> Self {
       Self {
            base: EntityBase::new(),
        }
    }
}

impl Behavior for MyEntity {
    fn init(&mut self) {
        let my_comp = self.base.get_component_mut::<MyComponent>().unwrap();
        my_comp.some_str = "From Entity".to_string();
    }

    fn update(&mut self) {

    }
}

impl Entity for MyEntity {
    fn get_base(&self) -> &EntityBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut EntityBase {
        &mut self.base
    }
}