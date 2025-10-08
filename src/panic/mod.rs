use std::io::Write;
use std::path::Path;
use std::process::Command;
use ::log::LevelFilter;
use itertools::Itertools;
use native_dialog::MessageDialogBuilder;
use crate::game::fs::cfgdir;
use crate::panic::alert::{AlertButtons, AlertFlavor};

mod log;
pub mod alert;

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

pub fn setup_panic(use_internal_logger: bool, log_dir: impl AsRef<Path> + Sync + Send + Clone + 'static) {
    if use_internal_logger {
        if !log::LOGGER.created() {
            panic!("To use internal logger for panic, please call `setup_logger` first");
        }
    }
    std::panic::set_hook(Box::new(move |info| {
        if let Some(message) = info.payload().downcast_ref::<&'static str>() {
            process_panic(message, log_dir.clone());
        } else if let Some(message) = info.payload().downcast_ref::<String>() {
            process_panic(message, log_dir.clone());
        } else if let Some(message) = info.payload().downcast_ref::<std::fmt::Arguments>() {
            process_panic(&format!("{}", message), log_dir.clone());
        } else {
            process_panic("No message available", log_dir.clone());
        }
    }));
}

fn process_panic(msg: &str, log_dir: impl AsRef<Path>) {
    if log::LOGGER.created() {
        //amazing logs available!

        let new_msg = format!("{msg}\nDo you want to open the log file?");
        let pressed = alert::os_alert("Application crashed!", &new_msg, AlertFlavor::Error, AlertButtons::YES | AlertButtons::NO);
        if let Some(pressed) = pressed {
            if pressed == AlertButtons::YES {
                //save logs to file
                let cached = log::LOGGER.cache.lock();
                let cfg_dir = cfgdir::acquire_config_smart_dir();
                let log_file = cfg_dir.join(log_dir);

                let data = cached
                    .iter()
                    .join("");

                drop(cached);

                let filename = "latest.log";

                if let Some(_) = log_file.save_object(&data, filename) {
                    //open notepad
                    let full_path = log_file.path().join(filename);
                    let command = Command::new("notepad.exe")
                        .arg(full_path.as_os_str())
                        .spawn();
                    if let Err(e) = command {
                        eprintln!("Cannot open notepad: {e}");
                    }
                }
            }
        }

        std::panic::resume_unwind(Box::new(msg.to_string()));
    } else {
        alert::os_alert("Application crashed!", msg, AlertFlavor::Error, AlertButtons::OK);

        std::panic::resume_unwind(Box::new(msg.to_string()));
    }
}

pub fn setup_logger(output: impl Write + 'static, level: LevelFilter, cached_lines: usize) {
    log::init(output, level, cached_lines);
}