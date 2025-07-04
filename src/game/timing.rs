use crate::utils::F0To1;
use hashbrown::HashMap;
use mvutils::hashers::U64IdentityHasher;
use mvutils::utils::Time;
use parking_lot::RwLock;
use std::sync::Arc;

#[repr(transparent)]
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct TaskId(u64);

type Task = Arc<RwLock<dyn ScheduleTask + 'static>>;

///A handle so that the task doesn't have to be acquired every time again
pub struct TaskHandle<'a> {
    id: TaskId,
    task: Task,
    map: &'a mut HashMap<u64, Task, U64IdentityHasher>,
}

impl TaskHandle<'_> {
    pub fn tick(&mut self) -> bool {
        let mut task = self.task.write();
        let state = task.poll();
        drop(task);
        if !state.keep {
            self.map.remove(&self.id.0);
        }
        state.run
    }

    pub fn progress(&self) -> F0To1 {
        self.task.read().progress()
    }

    pub fn cancel(self) {
        self.map.remove(&self.id.0);
    }
}

pub struct Scheduler {
    last_id: u64,
    tasks: HashMap<u64, Task, U64IdentityHasher>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            tasks: HashMap::with_hasher(U64IdentityHasher::default()),
        }
    }

    pub fn queue<S: ScheduleTask + 'static>(&mut self, task: S) -> TaskId {
        self.tasks.insert(self.last_id, Arc::new(RwLock::new(task)));
        self.last_id = self.last_id.wrapping_add(1);
        TaskId(self.last_id - 1)
    }

    pub fn cancel(&mut self, task: TaskId) {
        self.tasks.remove(&task.0);
    }

    pub fn handle(&mut self, task: TaskId) -> Option<TaskHandle> {
        let t = self.tasks.get(&task.0)?;
        Some(TaskHandle {
            id: task,
            task: t.clone(),
            map: &mut self.tasks,
        })
    }

    pub fn tick(&mut self, task: TaskId) -> bool {
        if let Some(t) = self.tasks.get(&task.0) {
            let t = t.clone();
            let mut t = t.write();
            let state = t.poll();
            if !state.keep {
                self.tasks.remove(&task.0);
            }
            state.run
        } else {
            false
        }
    }

    pub fn progress(&self, task: TaskId) -> F0To1 {
        if let Some(task) = self.tasks.get(&task.0) {
            task.read().progress()
        } else {
            0.0
        }
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
    fn progress(&self) -> F0To1;
}

pub struct DelayedTask {
    start: u64,
    delay: u64,
    pg: F0To1,
}

impl DelayedTask {
    pub fn new(delay_ms: u64) -> Self {
        DelayedTask {
            start: u64::time_millis(),
            delay: delay_ms,
            pg: 0.0,
        }
    }
}

impl ScheduleTask for DelayedTask {
    fn poll(&mut self) -> TaskState {
        let current = u64::time_millis();
        let elapsed = current - self.start;

        self.pg = (elapsed as f64 / self.delay as f64).min(1.0).max(0.0);

        if elapsed >= self.delay {
            self.pg = 1.0;
            TaskState::once()
        } else {
            TaskState::silent()
        }
    }

    fn progress(&self) -> F0To1 {
        self.pg
    }
}

pub struct RepeatTask {
    times: i32,
    n: i32,
    pg: F0To1,
}

impl RepeatTask {
    pub fn new(times: i32) -> Self {
        RepeatTask {
            times,
            n: 0,
            pg: 0.0,
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

            if self.times > 0 {
                self.pg = (self.n as f64 / self.times as f64).min(1.0);
            } else {
                self.pg = 0.0;
            }

            TaskState::run()
        } else {
            self.pg = 1.0;
            TaskState::remove()
        }
    }

    fn progress(&self) -> F0To1 {
        self.pg
    }
}

pub struct PeriodicTask {
    period: u64,
    start: u64,
    times: i32,
    n: i32,
    pg: F0To1,
}

impl PeriodicTask {
    pub fn new(period_ms: u64, times: i32) -> Self {
        PeriodicTask {
            period: period_ms,
            start: u64::time_millis(),
            times,
            n: 0,
            pg: 0.0,
        }
    }

    pub fn forever(period_ms: u64) -> Self {
        Self::new(period_ms, -1)
    }
}

impl ScheduleTask for PeriodicTask {
    fn poll(&mut self) -> TaskState {
        let current = u64::time_millis();
        let elapsed = current - self.start;

        self.pg = (elapsed as f64 / self.period as f64).min(1.0);

        if elapsed >= self.period {
            if self.times < 0 || self.n < self.times {
                self.n += 1;
                self.start = current;
                self.pg = 0.0;
                TaskState::run()
            } else {
                self.pg = 1.0;
                TaskState::remove()
            }
        } else {
            TaskState::silent()
        }
    }

    fn progress(&self) -> F0To1 {
        self.pg
    }
}

pub struct DurationTask {
    start: u64,
    duration: u64,
    pg: F0To1,
}

impl DurationTask {
    pub fn new(duration_ms: u64) -> Self {
        DurationTask {
            start: u64::time_millis(),
            duration: duration_ms,
            pg: 0.0,
        }
    }
}

impl ScheduleTask for DurationTask {
    fn poll(&mut self) -> TaskState {
        let current = u64::time_millis();
        let elapsed = current - self.start;

        self.pg = (elapsed as f64 / self.duration as f64).min(1.0);

        if elapsed >= self.duration {
            TaskState::remove()
        } else {
            TaskState::run()
        }
    }

    fn progress(&self) -> F0To1 {
        self.pg
    }
}
