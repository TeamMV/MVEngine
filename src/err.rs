use std::panic::PanicInfo;
use std::process::exit;
use log::error;

pub fn setup() {
    std::panic::set_hook(Box::new(panic));
}

pub fn panic(info: &PanicInfo) {
    let thread = std::thread::current()
        .name()
        .unwrap_or("unknown")
        .to_string();
    if let Some(message) = info.payload().downcast_ref::<&'static str>() {
        error!("Thread '{}' panicked with message '{}'", thread, message);
    } else if let Some(message) = info.payload().downcast_ref::<String>() {
        error!("Thread '{}' panicked with message '{}'", thread, message);
    } else if let Some(message) = info.payload().downcast_ref::<std::fmt::Arguments>() {
        error!("Thread '{}' panicked with message '{}'", thread, message);
    } else {
        error!("Thread '{}' panicked", thread);
    }
    exit(1);
}
