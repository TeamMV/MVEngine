use std::collections::vec_deque::IntoIter;
use std::collections::VecDeque;

pub struct EventBus<Event, Context: Copy> {
    receivers: Vec<Box<dyn EventReceiver<Event, Context>>>,
}

impl<Event, Context: Copy> EventBus<Event, Context> {
    pub fn new() -> Self {
        Self { receivers: vec![] }
    }

    pub fn dispatch(&mut self, event: &mut Event, context: Context) {
        let mut queue = EventQueue::new();
        for (rec) in &mut self.receivers {
            rec.on_dispatch(context, event, &mut queue);
        }
        for entry in &mut queue.events {
            self.dispatch(entry, context);
        }
    }

    pub fn subscribe(&mut self, receiver: impl EventReceiver<Event, Context> + 'static) {
        self.receivers.push(Box::new(receiver));
    }
}

pub trait EventReceiver<Event, Context> {
    fn on_dispatch(&mut self, context: Context, event: &mut Event, queue: &mut EventQueue<Event>);
}

pub struct EventQueue<Event> {
    events: VecDeque<Event>,
}

impl<Event> EventQueue<Event> {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    pub fn dispatch_new(&mut self, event: Event) {
        self.events.push_back(event);
    }
}

impl<Event> IntoIterator for EventQueue<Event> {
    type Item = Event;
    type IntoIter = IntoIter<Event>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

unsafe impl<Event: Send, Context: Copy> Send for EventBus<Event, Context> {}
