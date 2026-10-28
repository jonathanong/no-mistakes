use std::io::{self, Write};

#[cfg(any(windows, test))]
#[path = "stdio/wide.rs"]
mod wide;
#[cfg(windows)]
#[path = "stdio/windows.rs"]
mod windows;

pub(super) fn write(text: &str) -> io::Result<()> {
    let mut writer = duplicate_stdout()?;
    #[cfg(windows)]
    windows::write(&mut writer, text)?;
    #[cfg(not(windows))]
    writer.write_all(text.as_bytes())?;
    writer.flush()
}

#[cfg(unix)]
fn duplicate_stdout() -> io::Result<std::fs::File> {
    use std::os::fd::AsFd;
    std::io::stdout()
        .as_fd()
        .try_clone_to_owned()
        .map(std::fs::File::from)
}

#[cfg(windows)]
fn duplicate_stdout() -> io::Result<std::fs::File> {
    use std::os::windows::io::AsHandle;
    std::io::stdout()
        .as_handle()
        .try_clone_to_owned()
        .map(std::fs::File::from)
}

#[cfg(not(any(unix, windows)))]
fn duplicate_stdout() -> io::Result<impl Write> {
    Ok(std::io::stdout())
}
