#[cfg(unix)]
pub(crate) fn configure_process_group(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(windows)]
pub(crate) fn configure_process_group(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;

    // The child must not execute until `ProcessTree::attach` assigns it to the
    // job, otherwise an immediately spawned descendant can escape the job.
    command.creation_flags(CREATE_SUSPENDED);
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn configure_process_group(command: &mut std::process::Command) {
    let _ = command;
}

#[cfg(unix)]
pub(crate) mod signals;

#[cfg(unix)]
pub(crate) use signals::ParentSignalForwardingGuard;

#[cfg(not(unix))]
pub(crate) struct ParentSignalForwardingGuard;

#[cfg(not(unix))]
impl ParentSignalForwardingGuard {
    pub(crate) fn install(_enabled: bool) -> std::io::Result<Self> {
        Ok(Self)
    }
}

pub(crate) struct ProcessTree {
    #[cfg(unix)]
    registration: Option<signals::GroupRegistration>,
    #[cfg(windows)]
    job: windows_sys::Win32::Foundation::HANDLE,
}

impl ProcessTree {
    #[cfg(unix)]
    pub(crate) fn attach(child: &std::process::Child) -> Self {
        let process_group = child.id() as i32;
        let registration = signals::register_process_group(process_group);
        Self { registration }
    }

    #[cfg(windows)]
    pub(crate) fn attach(child: &std::process::Child) -> std::io::Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };

        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&limits).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        } == 0
        {
            let error = std::io::Error::last_os_error();
            unsafe { windows_sys::Win32::Foundation::CloseHandle(job) };
            return Err(error);
        }
        if unsafe { AssignProcessToJobObject(job, child.as_raw_handle()) } == 0 {
            let error = std::io::Error::last_os_error();
            unsafe { windows_sys::Win32::Foundation::CloseHandle(job) };
            return Err(error);
        }
        if let Err(error) = resume_initial_thread(child.id()) {
            // Closing a kill-on-close job also cleans up the still-suspended
            // child before the caller performs its best-effort cleanup.
            unsafe { windows_sys::Win32::Foundation::CloseHandle(job) };
            return Err(error);
        }
        Ok(Self { job })
    }

    #[cfg(not(any(unix, windows)))]
    pub(crate) fn attach(child: &std::process::Child) -> std::io::Result<Self> {
        let _ = child;
        Ok(Self {})
    }

    pub(crate) fn terminate(&self, child: &mut std::process::Child) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{killpg, Signal};
            use nix::unistd::Pid;

            classify_killpg_result(killpg(Pid::from_raw(child.id() as i32), Signal::SIGKILL))
        }
        #[cfg(windows)]
        {
            let _ = child;
            if unsafe { windows_sys::Win32::System::JobObjects::TerminateJobObject(self.job, 1) }
                == 0
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        }
        #[cfg(not(any(unix, windows)))]
        {
            child.kill()
        }
    }
}

/// Maps a `killpg` outcome to the `terminate()` contract: success and "the
/// process group is already gone" (`ESRCH`, the overwhelmingly common case —
/// the child already exited) both count as a successful termination; any
/// other signal-delivery failure (e.g. `EPERM`) surfaces as an `io::Error`
/// preserving the OS error code. Split out as a pure function — separate
/// from the syscall itself — so this mapping is unit-testable with a
/// synthetic `Err`, since a real `killpg` failure isn't reliably producible
/// in a portable test.
#[cfg(unix)]
fn classify_killpg_result(result: Result<(), nix::errno::Errno>) -> std::io::Result<()> {
    use nix::errno::Errno;
    match result {
        Ok(()) | Err(Errno::ESRCH) => Ok(()),
        Err(error) => Err(std::io::Error::from_raw_os_error(error as i32)),
    }
}

#[cfg(unix)]
impl Drop for ProcessTree {
    fn drop(&mut self) {
        let _ = self.registration.take();
    }
}

#[cfg(windows)]
impl Drop for ProcessTree {
    fn drop(&mut self) {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(self.job) };
    }
}
#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows::resume_initial_thread;

#[cfg(all(test, unix))]
#[path = "process_tree/tests.rs"]
mod tests;
