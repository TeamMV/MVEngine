use std::io::Write;
use std::ops::Deref;
use std::sync::Arc;
use log::{LevelFilter};
use mvutils::{create_once};
use parking_lot::Mutex;

create_once! {
    pub(in crate::panic) static LOGGER: CachedLogger;
}

pub fn init(output: impl Write + 'static, level: LevelFilter, cache_size: usize) {
    if let Ok(_) = LOGGER.try_create(|| CachedLogger::new(output, cache_size)) {
        mvlogger::init(LOGGER.deref().clone(), level);
    }
}

#[derive(Clone)]
pub struct CachedLogger {
    output: Arc<Mutex<dyn Write + 'static>>,
    pub(in super) cache: Arc<Mutex<Vec<String>>>,
}

unsafe impl Send for CachedLogger {}
unsafe impl Sync for CachedLogger {}

impl CachedLogger {
    pub fn new(output: impl Write + 'static, cache_size: usize) -> Self {
        Self {
            output: Arc::new(Mutex::new(output)),
            cache: Arc::new(Mutex::new(Vec::with_capacity(cache_size))),
        }
    }
}

impl Write for CachedLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let str = String::from_utf8(buf.to_vec()).unwrap_or("Illegal string received".to_string());
        let mut lock = self.cache.lock();
        if lock.len() == lock.capacity() {
            lock.remove(0);
        }
        lock.push(str);
        drop(lock);

        self.output.lock().write_all(buf)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.output.lock().flush()
    }
}