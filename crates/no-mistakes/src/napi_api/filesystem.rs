use std::io;
use std::path::Path;

#[cfg(not(coverage))]
use std::path::PathBuf;

#[cfg(not(coverage))]
use napi::{Env, Task};

pub(crate) fn rename_no_replace_impl(from: &Path, to: &Path) -> io::Result<bool> {
    platform_rename_no_replace(from, to)
}

#[cfg(target_os = "linux")]
fn platform_rename_no_replace(from: &Path, to: &Path) -> io::Result<bool> {
    use std::os::unix::ffi::OsStrExt;

    let from = std::ffi::CString::new(from.as_os_str().as_bytes())?;
    let to = std::ffi::CString::new(to.as_os_str().as_bytes())?;
    // SAFETY: both C strings remain live for the syscall and use AT_FDCWD paths.
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(true)
    } else if io::Error::last_os_error().kind() == io::ErrorKind::AlreadyExists {
        Ok(false)
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(target_os = "macos")]
fn platform_rename_no_replace(from: &Path, to: &Path) -> io::Result<bool> {
    use std::os::unix::ffi::OsStrExt;

    unsafe extern "C" {
        fn renamex_np(
            from: *const libc::c_char,
            to: *const libc::c_char,
            flags: u32,
        ) -> libc::c_int;
    }
    const RENAME_EXCL: u32 = 0x0000_0004;
    let from = std::ffi::CString::new(from.as_os_str().as_bytes())?;
    let to = std::ffi::CString::new(to.as_os_str().as_bytes())?;
    // SAFETY: both C strings remain live for the duration of renamex_np.
    let result = unsafe { renamex_np(from.as_ptr(), to.as_ptr(), RENAME_EXCL) };
    if result == 0 {
        Ok(true)
    } else if io::Error::last_os_error().kind() == io::ErrorKind::AlreadyExists {
        Ok(false)
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(windows)]
fn platform_rename_no_replace(from: &Path, to: &Path) -> io::Result<bool> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS};
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_WRITE_THROUGH};

    let from = from
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let to = to
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // SAFETY: both wide strings are NUL terminated and valid for the call.
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), MOVEFILE_WRITE_THROUGH) } != 0 {
        Ok(true)
    } else {
        // SAFETY: GetLastError reads thread-local Windows error state.
        match unsafe { GetLastError() } {
            ERROR_ALREADY_EXISTS | ERROR_FILE_EXISTS => Ok(false),
            code => Err(io::Error::from_raw_os_error(code as i32)),
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn platform_rename_no_replace(_from: &Path, _to: &Path) -> io::Result<bool> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "atomic no-replace rename is unavailable on this platform",
    ))
}

#[cfg(not(coverage))]
pub struct RenameNoReplaceTask {
    from: PathBuf,
    to: PathBuf,
}

#[cfg(not(coverage))]
impl RenameNoReplaceTask {
    pub(crate) fn new(from: PathBuf, to: PathBuf) -> Self {
        Self { from, to }
    }
}

#[cfg(not(coverage))]
impl Task for RenameNoReplaceTask {
    type Output = bool;
    type JsValue = bool;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        rename_no_replace_impl(&self.from, &self.to)
            .map_err(|error| napi::Error::from_reason(error.to_string()))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

#[cfg(test)]
mod tests;
