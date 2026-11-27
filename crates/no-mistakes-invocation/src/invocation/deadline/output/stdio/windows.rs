use std::fs::File;
use std::io::{self, Write};
use std::os::windows::io::AsRawHandle;
use windows_sys::Win32::System::Console::{GetConsoleMode, WriteConsoleW};

pub(super) fn write(file: &mut File, text: &str) -> io::Result<()> {
    let handle = file.as_raw_handle();
    let mut mode = 0;
    // A console needs Unicode console I/O; redirected files/pipes need UTF-8.
    // SAFETY: file keeps handle live and mode is writable output storage.
    if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
        return file.write_all(text.as_bytes());
    }
    super::wide::write_all(text, |units| {
        let mut written = 0;
        // SAFETY: handle is a console; units and written remain valid for this synchronous write.
        let result = unsafe {
            WriteConsoleW(
                handle,
                units.as_ptr().cast(),
                units.len() as u32,
                &mut written,
                std::ptr::null(),
            )
        };
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(written as usize)
    })
}
