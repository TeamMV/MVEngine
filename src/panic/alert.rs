use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;
use std::thread;
use bitflags::bitflags;
use winapi::um::winuser;
use winapi::um::winuser::{MessageBoxW, MB_CANCELTRYCONTINUE, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MB_OKCANCEL, MB_RETRYCANCEL, MB_YESNO, MB_YESNOCANCEL};

pub enum AlertFlavor {
    Info,
    Error
}

bitflags! {
    #[derive(PartialEq)]
    pub struct AlertButtons: u8 {
        const YES = 1;
        const NO = 1 << 1;
        const TRY = 1 << 2;
        const CANCEL = 1 << 3;
        const OK = 1 << 4;
    }
}

pub fn os_alert(title: &str, msg: &str, flavor: AlertFlavor, buttons: AlertButtons) -> Option<AlertButtons> {
    #[cfg(target_os = "windows")]
    { return windows_alert(title, msg, flavor, buttons); }

    #[cfg(target_os = "linux")]
    { return linux_alert(title, msg, flavor, buttons); }

    #[cfg(target_os = "macos")]
    { return macos_alert(title, msg, flavor, buttons); }

    eprintln!("Bro what os u on?");
}

#[cfg(target_os = "windows")]
fn windows_alert(title: &str, msg: &str, flavor: AlertFlavor, buttons: AlertButtons) -> Option<AlertButtons> {
    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    let msg_wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(Some(0)).collect();

    let icon_flag = match flavor {
        AlertFlavor::Info => MB_ICONINFORMATION,
        AlertFlavor::Error => MB_ICONERROR,
    };

    let button_flag = match buttons {
        b if b.contains(AlertButtons::YES) && b.contains(AlertButtons::NO) && b.contains(AlertButtons::CANCEL) => MB_YESNOCANCEL,
        b if b.contains(AlertButtons::YES) && b.contains(AlertButtons::NO) => MB_YESNO,
        b if b.contains(AlertButtons::OK) && b.contains(AlertButtons::CANCEL) => MB_OKCANCEL,
        b if b.contains(AlertButtons::TRY) && b.contains(AlertButtons::CANCEL) => MB_RETRYCANCEL,
        b if b.contains(AlertButtons::OK) => MB_OK,
        _ => MB_OK,
    };

    let (tx, rx) = crossbeam_channel::bounded(0);
    let handle = thread::spawn(move || {
        unsafe {
            let pressed = MessageBoxW(
                std::ptr::null_mut(),
                msg_wide.as_ptr(),
                title_wide.as_ptr(),
                button_flag | icon_flag,
            );
            let _ = tx.send(pressed);
        }
    });

    let pressed = rx.recv().unwrap_or(0);
    let _ = handle.join();

    let result = match pressed {
        winuser::IDOK => Some(AlertButtons::OK),
        winuser::IDYES => Some(AlertButtons::YES),
        winuser::IDNO => Some(AlertButtons::NO),
        winuser::IDRETRY => Some(AlertButtons::TRY),
        winuser::IDCANCEL => Some(AlertButtons::CANCEL),
        _ => None,
    };

    result
}

#[cfg(target_os = "linux")]
fn linux_alert(title: &str, msg: &str, flavor: AlertFlavor, buttons: AlertButtons) -> Option<AlertButtons> {
    use std::process::Command;

    let mut command = if Command::new("zenity").arg("--version").output().is_ok() {
        let mut cmd = Command::new("zenity");
        cmd.arg("--question")
            .arg("--title").arg(title)
            .arg("--text").arg(msg);
        match flavor {
            AlertFlavor::Error => { cmd.arg("--icon-name=dialog-error"); }
            AlertFlavor::Info => { cmd.arg("--icon-name=dialog-information"); }
        }

        // Map buttons roughly
        if buttons.contains(AlertButtons::YES) && buttons.contains(AlertButtons::NO) {
            cmd.arg("--ok-label=Yes").arg("--cancel-label=No");
        } else if buttons.contains(AlertButtons::OK) {
            cmd.arg("--ok-label=OK");
        } else if buttons.contains(AlertButtons::OK) && buttons.contains(AlertButtons::CANCEL) {
            cmd.arg("--ok-label=OK").arg("--cancel-label=Cancel");
        }

        cmd
    } else if Command::new("kdialog").arg("--version").output().is_ok() {
        let mut cmd = Command::new("kdialog");
        cmd.arg("--yesno").arg(msg).arg("--title").arg(title);
        cmd
    } else {
        eprintln!("No zenity or kdialog found. Using stderr fallback.");
        eprintln!("{}: {}", title, msg);
        return None;
    };

    let status = command.status().ok()?;

    // Zenity: 0 = OK/Yes, 1 = No/Cancel
    if status.success() {
        if buttons.contains(AlertButtons::YES) {
            Some(AlertButtons::YES)
        } else {
            Some(AlertButtons::OK)
        }
    } else {
        if buttons.contains(AlertButtons::NO) {
            Some(AlertButtons::NO)
        } else {
            Some(AlertButtons::CANCEL)
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_alert(title: &str, msg: &str, flavor: AlertFlavor, buttons: AlertButtons) -> Option<AlertButtons> {
    use std::process::Command;

    // Compose AppleScript for dialog
    let buttons_str = if buttons.contains(AlertButtons::YES) && buttons.contains(AlertButtons::NO) {
        "buttons {\"Yes\", \"No\"} default button 1"
    } else if buttons.contains(AlertButtons::OK) && buttons.contains(AlertButtons::CANCEL) {
        "buttons {\"OK\", \"Cancel\"} default button 1"
    } else if buttons.contains(AlertButtons::OK) {
        "buttons {\"OK\"} default button 1"
    } else {
        "buttons {\"OK\"} default button 1"
    };

    let icon = match flavor {
        AlertFlavor::Error => "stop",
        AlertFlavor::Info => "note",
    };

    let script = format!(
        "display dialog \"{}\" with title \"{}\" with icon {} {}",
        msg.replace('"', "\\\""),
        title.replace('"', "\\\""),
        icon,
        buttons_str
    );

    let output = Command::new("osascript")
        .arg("-e").arg(&script)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("button returned:Yes") {
        Some(AlertButtons::YES)
    } else if stdout.contains("button returned:No") {
        Some(AlertButtons::NO)
    } else if stdout.contains("button returned:Cancel") {
        Some(AlertButtons::CANCEL)
    } else if stdout.contains("button returned:OK") {
        Some(AlertButtons::OK)
    } else {
        None
    }
}