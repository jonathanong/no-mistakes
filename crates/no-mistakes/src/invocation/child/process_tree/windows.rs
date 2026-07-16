use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME};

pub(super) fn resume_initial_thread(process_id: u32) -> std::io::Result<()> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error());
    }

    let result = find_and_resume_thread(snapshot, process_id);
    unsafe { CloseHandle(snapshot) };
    result
}

fn find_and_resume_thread(
    snapshot: windows_sys::Win32::Foundation::HANDLE,
    process_id: u32,
) -> std::io::Result<()> {
    let mut entry = THREADENTRY32 {
        dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
        ..THREADENTRY32::default()
    };
    if unsafe { Thread32First(snapshot, &mut entry) } == 0 {
        return Err(std::io::Error::last_os_error());
    }

    loop {
        if entry.th32OwnerProcessID == process_id {
            return resume_thread(entry.th32ThreadID);
        }
        if unsafe { Thread32Next(snapshot, &mut entry) } == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "could not find the suspended child process's initial thread",
            ));
        }
    }
}

fn resume_thread(thread_id: u32) -> std::io::Result<()> {
    let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, thread_id) };
    if thread.is_null() {
        return Err(std::io::Error::last_os_error());
    }
    let resume_result = unsafe { ResumeThread(thread) };
    let resume_error = (resume_result == u32::MAX).then(std::io::Error::last_os_error);
    unsafe { CloseHandle(thread) };
    if let Some(error) = resume_error {
        return Err(error);
    }
    if resume_result != 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("suspended child process had unexpected thread suspend count {resume_result}"),
        ));
    }
    Ok(())
}
