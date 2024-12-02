use std::panic::PanicHookInfo;
use std::process::exit;

pub fn setup() {
    std::panic::set_hook(Box::new(panic));
}

fn panic(info: &PanicHookInfo) {
    let thread = std::thread::current()
        .name()
        .unwrap_or("unknown")
        .to_string();
    if let Some(message) = info.payload().downcast_ref::<&'static str>() {
        log::error!("Thread '{}' panicked with message '{}'", thread, message);
    } else if let Some(message) = info.payload().downcast_ref::<String>() {
        log::error!("Thread '{}' panicked with message '{}'", thread, message);
    } else if let Some(message) = info.payload().downcast_ref::<std::fmt::Arguments>() {
        log::error!("Thread '{}' panicked with message '{}'", thread, message);
    } else {
        log::error!("Thread '{}' panicked", thread);
    }
    exit(1)
}
