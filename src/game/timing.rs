use hashbrown::HashMap;
use mvutils::hashers::U64IdentityHasher;
use mvutils::utils::Time;

#[repr(transparent)]
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct TaskId(u64);

pub struct Scheduler {
    last_id: u64,
    tasks: HashMap<u64, Box<dyn ScheduleTask>, U64IdentityHasher>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            tasks: HashMap::with_hasher(U64IdentityHasher::default()),
        }
    }

    pub fn queue<S: ScheduleTask>(&mut self, task: S) -> TaskId {
        self.tasks.insert(self.last_id, Box::new(task));
        self.last_id = self.last_id.wrapping_add(1);
        TaskId(self.last_id - 1)
    }

    pub fn cancel(&mut self, task: TaskId) {
        self.tasks.remove(&task.0);
    }

    pub fn tick(&mut self, task: TaskId) -> bool {
        if let Some(t) = self.tasks.get_mut(&task.0) {
            let state = t.poll();
            if !state.keep {
                self.tasks.remove(&task.0);
            }
            state.run
        } else { false }
    }
}

pub struct TaskState {
    run: bool,
    keep: bool,
}

impl TaskState {
    pub fn once() -> Self {
        Self {
            run: true,
            keep: false,
        }
    }
    
    pub fn remove() -> Self {
        Self {
            run: false,
            keep: false,
        }
    }

    pub fn run() -> Self {
        Self {
            run: true,
            keep: true,
        }
    }

    pub fn silent() -> Self {
        Self {
            run: false,
            keep: true,
        }
    }
}

pub trait ScheduleTask {
    fn poll(&mut self) -> TaskState;
}

pub struct DelayedTask {
    start: u64,
    delay: u64
}

impl DelayedTask {
    pub fn new(delay_ms: u64) -> Self {
        DelayedTask {
            start: u64::time_millis(),
            delay: delay_ms,
        }
    }
}

impl ScheduleTask for DelayedTask {
    fn poll(&mut self) -> TaskState {
        let current = u64::time_millis();
        if current - self.start >= self.delay {
            TaskState::once()
        } else {
            TaskState::silent()
        }
    }
}

pub struct RepeatTask {
    times: i32,
    n: i32
}

impl RepeatTask {
    pub fn new(times: i32) -> Self {
        RepeatTask {
            times,
            n: 0,
        }
    }
    
    pub fn forever() -> Self {
        Self::new(-1)
    }
}

impl ScheduleTask for RepeatTask {
    fn poll(&mut self) -> TaskState {
        if self.times < 0 || self.n < self.times {
            self.n += 1;
            TaskState::run()
        } else {
            TaskState::remove()
        }
    }
}

pub struct PeriodicTask {
    period: u64,
    start: u64,
    times: i32,
    n: i32,
}

impl PeriodicTask {
    pub fn new(period_ms: u64, times: i32) -> Self {
        PeriodicTask {
            period: period_ms,
            start: u64::time_millis(),
            times,
            n: 0,
        }
    }
    
    pub fn forever(period_ms: u64) -> Self {
        Self::new(period_ms, -1)
    }
}

impl ScheduleTask for PeriodicTask {
    fn poll(&mut self) -> TaskState {
        let current = u64::time_millis();
        if current - self.start >= self.period {
            if self.times < 0 || self.n < self.times {
                self.n += 1;
                self.start = current;
                TaskState::run()
            } else {
                TaskState::remove()
            }
        } else {
            TaskState::silent()
        }
    }
}
