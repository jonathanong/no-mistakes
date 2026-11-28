use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_LOCK_VIOLATION};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandle, LockFileEx, UnlockFileEx, BY_HANDLE_FILE_INFORMATION,
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT,
    LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
};
use windows_sys::Win32::System::IO::OVERLAPPED;

pub(super) fn acquire_planning_artifact_lock(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let metadata = file.metadata()?;
    let opened = file_state(&file)?;
    if !metadata.is_file() || !opened.is_regular_file_with_one_link() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "planning artifact lock must be a regular file with one link",
        ));
    }
    lock(&file)?;
    let path_file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let path_metadata = path_file.metadata()?;
    let path_state = file_state(&path_file)?;
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

pub(super) fn unlock_planning_artifact_lock(file: &File) -> io::Result<()> {
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

#[derive(Clone, Copy, PartialEq, Eq)]
struct FileIdentity {
    volume_serial_number: u32,
    file_index: u64,
}

struct FileState {
    identity: FileIdentity,
    number_of_links: u32,
    attributes: u32,
}

impl FileState {
    fn is_regular_file_with_one_link(&self) -> bool {
        self.number_of_links == 1
            && self.attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) == 0
    }
}

fn file_state(file: &File) -> io::Result<FileState> {
    // SAFETY: zero initializes all output fields before the Win32 call writes them.
    let mut information = unsafe { std::mem::zeroed::<BY_HANDLE_FILE_INFORMATION>() };
    // SAFETY: the File owns a live Windows handle and the output value remains live
    // and writable for the duration of the call.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(FileState {
        identity: FileIdentity {
            volume_serial_number: information.dwVolumeSerialNumber,
            file_index: (u64::from(information.nFileIndexHigh) << 32)
                | u64::from(information.nFileIndexLow),
        },
        number_of_links: information.nNumberOfLinks,
        attributes: information.dwFileAttributes,
    })
}

fn lock(file: &File) -> io::Result<()> {
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
