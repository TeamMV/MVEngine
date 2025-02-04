use crate::ui::anim::ElementAnimationInfo;
use crate::ui::utils::AnyType;
use hashbrown::HashMap;
use mvutils::hashers::U64IdentityHasher;
use mvutils::lazy;
use mvutils::utils::{TetrahedronOp, Time};

pub struct TimingManager {
    tasks: HashMap<u64, (Box<dyn TimingTask>, Option<Box<dyn FnMut()>>), U64IdentityHasher>,
}

lazy! {
    pub static mut TIMING_MANAGER: TimingManager = TimingManager::new();
}

impl TimingManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::with_hasher(U64IdentityHasher::default()),
        }
    }

    pub fn request<T>(&mut self, task: T, on_finish: Option<Box<dyn FnMut()>>) -> u64
    where
        T: TimingTask + 'static,
    {
        let id = mvutils::utils::next_id("MVCore::ui::timingTask");
        self.tasks.insert(id, (Box::new(task), on_finish));
        id
    }

    pub fn cancel(&mut self, id: u64) {
        self.tasks.remove(&id);
    }

    pub fn is_present(&self, id: u64) -> bool {
        self.tasks.contains_key(&id)
    }

    pub fn post_frame(&mut self, dt: f32, frame: u64) {
        let mut to_remove = vec![];
        for task in self.tasks.iter_mut() {
            task.1 .0.iteration(dt, frame);
            if task.1 .0.is_done() {
                to_remove.push(task.0.clone());
                if task.1 .1.is_some() {
                    let on_finish = task.1 .1.as_mut().unwrap();
                    on_finish();
                }
            }
        }
        for id in to_remove {
            self.tasks.remove(&id);
        }
    }
}

pub trait TimingTask {
    fn is_done(&self) -> bool;
    fn iteration(&mut self, dt: f32, frame: u64);
}

///A TimingTask, that is executed every frame until <u>limit</u> iterations are reached. Pass -1 for infinite cycling
pub struct IterationTask {
    function: Box<dyn FnMut(&mut AnimationState, u32)>,
    state: AnimationState,
    current_iter: u32,
    limit: i32,
}

impl IterationTask {
    pub fn new<F>(limit: i32, function: F, state: AnimationState) -> Self
    where
        F: FnMut(&mut AnimationState, u32) + 'static,
    {
        Self {
            current_iter: 0,
            limit,
            state,
            function: Box::new(function),
        }
    }
}

impl TimingTask for IterationTask {
    fn is_done(&self) -> bool {
        self.limit > 0 && self.current_iter as i32 >= self.limit
    }

    fn iteration(&mut self, dt: f32, frame: u64) {
        (self.function)(&mut self.state, self.current_iter);
        self.current_iter += 1;
    }
}

///A TimingTask, that is executed every frame for <u>duration</u> milliseconds
pub struct DurationTask {
    function: Box<dyn FnMut(&mut AnimationState, u32)>,
    state: AnimationState,
    init_time: u128,
    duration: u32,
}

impl DurationTask {
    pub fn new<F>(duration_ms: u32, function: F, state: AnimationState) -> Self
    where
        F: FnMut(&mut AnimationState, u32) + 'static,
    {
        Self {
            function: Box::new(function),
            state,
            init_time: u128::time_millis(),
            duration: duration_ms,
        }
    }
}

impl TimingTask for DurationTask {
    fn is_done(&self) -> bool {
        u128::time_millis() > self.init_time + self.duration as u128
    }

    fn iteration(&mut self, dt: f32, frame: u64) {
        (self.function)(
            &mut self.state,
            (u128::time_millis() - self.init_time) as u32,
        );
    }
}

///A TimingTask, that is executed after <u>delay</u> milliseconds
pub struct DelayTask {
    function: Box<dyn FnMut(&mut AnimationState, u32)>,
    state: AnimationState,
    init_time: u128,
    delay: u32,
    done: bool,
}

impl DelayTask {
    pub fn new<F>(delay_ms: u32, function: F, state: AnimationState) -> Self
    where
        F: FnMut(&mut AnimationState, u32) + 'static,
    {
        Self {
            function: Box::new(function),
            state,
            init_time: u128::time_millis(),
            delay: delay_ms,
            done: false,
        }
    }
}

impl TimingTask for DelayTask {
    fn is_done(&self) -> bool {
        self.done && u128::time_millis() > self.init_time + self.delay as u128
    }

    fn iteration(&mut self, dt: f32, frame: u64) {
        let ms = u128::time_millis();
        if ms > self.init_time + self.delay as u128 {
            (self.function)(&mut self.state, (ms - self.init_time) as u32);
            self.done = true;
        }
    }
}

///A TimingTask, that is executed every <u>delay</u> milliseconds once, until <u>limit</u> is reached. pass -1 for infinite cycling
pub struct PeriodicTask {
    function: Box<dyn FnMut(&mut AnimationState, u32)>,
    state: AnimationState,
    init_time: u128,
    delay: u32,
    limit: i32,
    current_iter: u32,
}

impl PeriodicTask {
    pub fn new<F>(limit: i32, delay_ms: u32, function: F, state: AnimationState) -> Self
    where
        F: FnMut(&mut AnimationState, u32) + 'static,
    {
        Self {
            function: Box::new(function),
            state,
            init_time: u128::time_millis(),
            delay: delay_ms,
            limit,
            current_iter: 0,
        }
    }
}

impl TimingTask for PeriodicTask {
    fn is_done(&self) -> bool {
        self.limit > 0 && self.current_iter as i32 > self.limit
    }

    fn iteration(&mut self, dt: f32, frame: u64) {
        let ms = u128::time_millis();
        if ms > self.init_time + self.delay as u128 {
            self.init_time = ms;
            (self.function)(&mut self.state, self.current_iter);
            self.current_iter += 1;
        }
    }
}

pub struct AnimationState {
    pub(crate) element: Option<ElementAnimationInfo>,
    pub(crate) value: Option<AnyType>,
    pub(crate) values: Option<Vec<AnyType>>,
}

impl AnimationState {
    pub fn empty() -> Self {
        Self {
            element: None,
            value: None,
            values: None,
        }
    }

    pub fn element(em: ElementAnimationInfo) -> Self {
        Self {
            element: Some(em),
            value: None,
            values: None,
        }
    }

    pub fn value<T>(value: &T) -> Self {
        Self {
            element: None,
            value: Some(AnyType::new(value)),
            values: None,
        }
    }

    pub fn get_value(&self) -> AnyType {
        return self.value.clone().unwrap();
    }

    pub fn values(values: &[AnyType]) -> Self {
        Self {
            element: None,
            value: None,
            values: Some(values.to_vec()),
        }
    }

    pub fn get_value_at(&self, idx: usize) -> AnyType {
        let vec = self.values.as_ref().unwrap();
        vec[idx].clone()
    }
}
