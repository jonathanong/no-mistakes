use no_mistakes::invocation::{command_output, install_test_deadline};
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::os::windows::io::{AsHandle, AsRawHandle, FromRawHandle};
use std::process::Command;
use std::time::Duration;
use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};
use windows_sys::Win32::System::Console::{
    AllocConsole, CreateConsoleScreenBuffer, FreeConsole, ReadConsoleOutputCharacterW,
    SetConsoleOutputCP, CONSOLE_TEXTMODE_BUFFER, COORD,
};

// Compile the exact production adapters in the small native Windows harness.
#[path = "../../src/invocation/deadline/output/stdio/windows.rs"]
mod console;
#[path = "../../src/invocation/deadline/output/stdio/wide.rs"]
mod wide;

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/invocation/deadline-json/unicode.json"
    )))
    .unwrap()
}

#[test]
fn redirected_deadline_output_remains_utf8() {
    let data = fixture();
    let text = data["text"].as_str().unwrap();
    let mut file = tempfile::tempfile().unwrap();
    console::write(&mut file, text).unwrap();
    file.rewind().unwrap();
    let mut actual = Vec::new();
    file.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, text.as_bytes());
}

#[test]
fn native_console_ignores_the_oem_output_code_page() {
    const CHILD: &str = "NO_MISTAKES_TEST_DEADLINE_CONSOLE_CHILD";
    const SUCCESS: &str = "deadline-console-unicode-ok";
    if std::env::var_os(CHILD).is_some() {
        // Keep diagnostics on the inherited pipe after allocating a private console.
        let mut diagnostic =
            File::from(std::io::stderr().as_handle().try_clone_to_owned().unwrap());
        let result = console_roundtrip();
        if let Err(error) = &result {
            let _ = writeln!(diagnostic, "deadline console regression: {error}");
            let _ = diagnostic.flush();
        } else {
            writeln!(diagnostic, "{SUCCESS}").unwrap();
            diagnostic.flush().unwrap();
        }
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }
    let name = concat!(
        module_path!(),
        "::native_console_ignores_the_oem_output_code_page"
    );
    let filter = name.split_once("::").unwrap().1;
    let _deadline = install_test_deadline(Duration::from_secs(10)).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", filter, "--nocapture"])
        .env(CHILD, "1");
    let output = command_output(&mut command).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // A misspelled --exact filter must not turn a zero-test child into success.
    assert!(String::from_utf8_lossy(&output.stderr).contains(SUCCESS));
}

fn console_roundtrip() -> io::Result<()> {
    let data = fixture();
    let expected: Vec<u16> = serde_json::from_value(data["consoleUtf16"].clone())?;
    // This runs only in a child: the test runner's own console stays untouched.
    unsafe { FreeConsole() };
    if unsafe { AllocConsole() } == 0 || unsafe { SetConsoleOutputCP(437) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let handle = unsafe {
        CreateConsoleScreenBuffer(
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            CONSOLE_TEXTMODE_BUFFER,
            std::ptr::null(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let mut file = unsafe { File::from_raw_handle(handle) };
    console::write(&mut file, data["consoleText"].as_str().unwrap())?;
    let mut actual = vec![0; expected.len()];
    let mut read = 0;
    let result = unsafe {
        ReadConsoleOutputCharacterW(
            file.as_raw_handle(),
            actual.as_mut_ptr(),
            actual.len() as u32,
            COORD { X: 0, Y: 0 },
            &mut read,
        )
    };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    actual.truncate(read as usize);
    if actual != expected {
        return Err(io::Error::other(format!(
            "console UTF-16 {actual:?}, expected {expected:?}"
        )));
    }
    Ok(())
}
