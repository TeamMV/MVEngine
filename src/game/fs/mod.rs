use std::io;
use std::path::Path;
use std::process::Command;

pub mod cfgdir;
pub mod smartdir;

pub fn os_file_view(file: impl AsRef<Path>) -> io::Result<()> {
    let path = file.as_ref();

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", path.to_str().unwrap()])
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        if Command::new("xdg-open").arg(path).spawn().is_err() {
            let _ = Command::new("open").arg(path).spawn();
        }
    }

    Ok(())
}
