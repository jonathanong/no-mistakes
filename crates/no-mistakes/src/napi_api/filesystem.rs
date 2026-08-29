use std::io;
use std::path::Path;

use std::fs::File;

pub(crate) fn rename_no_replace_impl(from: &Path, to: &Path) -> io::Result<bool> {
    platform_rename_no_replace(from, to)
}

#[cfg(unix)]
pub(crate) fn acquire_planning_artifact_lock_impl(path: &Path) -> io::Result<File> {
    use std::fs::{symlink_metadata, OpenOptions, Permissions};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "planning artifact lock must be a regular file with one link",
        ));
    }
    file.set_permissions(Permissions::from_mode(0o600))?;
    // SAFETY: flock only borrows this live descriptor; the File retains it until release.
    flock_impl(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB).map_err(map_advisory_lock_error)?;
    let path_metadata = symlink_metadata(path)?;
    validate_planning_artifact_lock_identity(&metadata, &path_metadata)?;
    Ok(file)
}

#[cfg(unix)]
fn map_advisory_lock_error(error: io::Error) -> io::Error {
    if error.kind() == io::ErrorKind::WouldBlock {
        io::Error::new(io::ErrorKind::WouldBlock, "planning artifact lock is busy")
    } else {
        error
    }
}

#[cfg(unix)]
fn validate_planning_artifact_lock_identity(
    opened: &std::fs::Metadata,
    path: &std::fs::Metadata,
) -> io::Result<()> {
    use std::os::unix::fs::MetadataExt;

    if !path.is_file()
        || path.nlink() != 1
        || path.dev() != opened.dev()
        || path.ino() != opened.ino()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "planning artifact lock changed during acquisition",
        ));
    }
    Ok(())
}

#[cfg(windows)]
pub(crate) fn acquire_planning_artifact_lock_impl(path: &Path) -> io::Result<File> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let metadata = file.metadata()?;
    let opened = windows_file_state(&file)?;
    if !metadata.is_file() || !opened.is_regular_file_with_one_link() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "planning artifact lock must be a regular file with one link",
        ));
    }
    lock_planning_artifact_lock_impl(&file)?;
    let path_file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let path_metadata = path_file.metadata()?;
    let path_state = windows_file_state(&path_file)?;
    if !path_metadata.is_file()
        || !path_state.is_regular_file_with_one_link()
        || path_state.identity != opened.identity
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "planning artifact lock changed during acquisition",
        ));
    }
    Ok(file)
}

#[cfg(windows)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct WindowsFileIdentity {
    volume_serial_number: u32,
    file_index: u64,
}

#[cfg(windows)]
struct WindowsFileState {
    identity: WindowsFileIdentity,
    number_of_links: u32,
    attributes: u32,
}

#[cfg(windows)]
impl WindowsFileState {
    fn is_regular_file_with_one_link(&self) -> bool {
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
        };

        self.number_of_links == 1
            && self.attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) == 0
    }
}

#[cfg(windows)]
fn windows_file_state(file: &File) -> io::Result<WindowsFileState> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };

    // SAFETY: zero initializes all output fields before the Win32 call writes them.
    let mut information = unsafe { std::mem::zeroed::<BY_HANDLE_FILE_INFORMATION>() };
    // SAFETY: the File owns a live Windows handle and the output value remains live
    // and writable for the duration of the call.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(WindowsFileState {
        identity: WindowsFileIdentity {
            volume_serial_number: information.dwVolumeSerialNumber,
            file_index: (u64::from(information.nFileIndexHigh) << 32)
                | u64::from(information.nFileIndexLow),
        },
        number_of_links: information.nNumberOfLinks,
        attributes: information.dwFileAttributes,
    })
}

#[cfg(windows)]
fn lock_planning_artifact_lock_impl(file: &File) -> io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_LOCK_VIOLATION};
    use windows_sys::Win32::Storage::FileSystem::{
        LockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
    };
    use windows_sys::Win32::System::IO::OVERLAPPED;

    // SAFETY: zero is the documented initial state for an OVERLAPPED used with a
    // synchronous file handle, and the value remains live for the call.
    let mut overlapped = unsafe { std::mem::zeroed::<OVERLAPPED>() };
    // SAFETY: the File owns a live Windows handle for the duration of this call.
    if unsafe {
        LockFileEx(
            file.as_raw_handle(),
            LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0,
            1,
            0,
            &mut overlapped,
        )
    } != 0
    {
        Ok(())
    } else {
        // SAFETY: GetLastError reads thread-local Windows error state immediately
        // after the failed LockFileEx call.
        let code = unsafe { GetLastError() };
        if code == ERROR_LOCK_VIOLATION {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "planning artifact lock is busy",
            ))
        } else {
            Err(io::Error::from_raw_os_error(code as i32))
        }
    }
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn acquire_planning_artifact_lock_impl(_path: &Path) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "planning artifact locks are unavailable on this platform",
    ))
}

#[cfg(windows)]
fn unlock_planning_artifact_lock_impl(file: &File) -> io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::UnlockFileEx;
    use windows_sys::Win32::System::IO::OVERLAPPED;

    // SAFETY: zero is the documented initial state for an OVERLAPPED used with a
    // synchronous file handle, and the value remains live for the call.
    let mut overlapped = unsafe { std::mem::zeroed::<OVERLAPPED>() };
    // SAFETY: the File owns a live Windows handle for the duration of this call.
    if unsafe { UnlockFileEx(file.as_raw_handle(), 0, 1, 0, &mut overlapped) } != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(unix)]
fn unlock_planning_artifact_lock_impl(file: &File) -> io::Result<()> {
    use std::os::fd::AsRawFd;

    // SAFETY: flock only borrows this live descriptor.
    flock_impl(file.as_raw_fd(), libc::LOCK_UN)
}

#[cfg(unix)]
fn flock_impl(file_descriptor: std::os::fd::RawFd, operation: libc::c_int) -> io::Result<()> {
    // SAFETY: callers provide a descriptor and an operation accepted by flock.
    if unsafe { libc::flock(file_descriptor, operation) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
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
mod tasks;
#[cfg(not(coverage))]
pub use tasks::{
    AcquirePlanningArtifactLockTask, ReleasePlanningArtifactLockTask, RenameNoReplaceTask,
};

#[cfg(test)]
mod tests;
