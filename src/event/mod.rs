use std::collections::VecDeque;

pub struct EventBus<Event> {
    receivers: Vec<Box<dyn EventReceiver<Event>>>,
}

impl<Event> EventBus<Event> {
    pub fn new() -> Self {
        Self {
            receivers: vec![],
        }
    }

    pub fn dispatch(&mut self, event: &mut Event) {
        let mut queue = EventQueue::new();
        for rec in &mut self.receivers {
            rec.on_dispatch(event, &mut queue);
        }
        for entry in &mut queue.events {
            self.dispatch(entry);
        }
    }

    pub fn subscribe(&mut self, receiver: impl EventReceiver<Event> + 'static) {
        self.receivers.push(Box::new(receiver));
    }
}

pub trait EventReceiver<Event> {
    fn on_dispatch(&mut self, event: &mut Event, queue: &mut EventQueue<Event>);
}

pub struct EventQueue<Event> {
    events: VecDeque<Event>
}

impl<Event> EventQueue<Event> {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }
    
    pub fn dispatch_new(&mut self, event: Event) {
        self.events.push_back(event);
    }
}