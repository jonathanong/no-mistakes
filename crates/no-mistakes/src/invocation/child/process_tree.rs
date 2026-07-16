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

pub(crate) struct ProcessTree {
    #[cfg(unix)]
    signal_handle: Option<signal_hook::iterator::Handle>,
    #[cfg(unix)]
    signal_thread: Option<std::thread::JoinHandle<()>>,
    #[cfg(windows)]
    job: windows_sys::Win32::Foundation::HANDLE,
}

impl ProcessTree {
    #[cfg(unix)]
    pub(crate) fn attach(child: &std::process::Child, forward_parent_signals: bool) -> Self {
        let process_group = child.id() as i32;
        let listener = forward_parent_signals
            .then(|| {
                signal_hook::iterator::Signals::new([
                    signal_hook::consts::SIGINT,
                    signal_hook::consts::SIGTERM,
                ])
            })
            .transpose()
            .ok()
            .flatten()
            .map(|signals| {
                let handle = signals.handle();
                // CLI invocations forward terminal signals to the isolated child
                // group, then preserve the CLI's default termination semantics.
                let thread = spawn_signal_listener(
                    signals,
                    process_group,
                    signal_hook::low_level::emulate_default_handler,
                );
                (handle, thread)
            });
        let (signal_handle, signal_thread) = listener
            .map(|(handle, thread)| (Some(handle), Some(thread)))
            .unwrap_or((None, None));
        Self {
            signal_handle,
            signal_thread,
        }
    }

    #[cfg(windows)]
    pub(crate) fn attach(
        child: &std::process::Child,
        _forward_parent_signals: bool,
    ) -> std::io::Result<Self> {
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
    pub(crate) fn attach(
        child: &std::process::Child,
        _forward_parent_signals: bool,
    ) -> std::io::Result<Self> {
        let _ = child;
        Ok(Self {})
    }

    pub(crate) fn terminate(&self, child: &mut std::process::Child) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            use nix::errno::Errno;
            use nix::sys::signal::{killpg, Signal};
            use nix::unistd::Pid;

            match killpg(Pid::from_raw(child.id() as i32), Signal::SIGKILL) {
                Ok(()) | Err(Errno::ESRCH) => Ok(()),
                Err(error) => Err(std::io::Error::from_raw_os_error(error as i32)),
            }
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

#[cfg(unix)]
pub(crate) fn forward_signal(process_group: i32, signal: i32) {
    unsafe {
        nix::libc::kill(-process_group, signal);
    }
}

#[cfg(unix)]
pub(crate) fn spawn_signal_listener<R>(
    mut signals: signal_hook::iterator::Signals,
    process_group: i32,
    terminate_parent: impl FnOnce(i32) -> R + Send + 'static,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Some(signal) = signals.forever().next() {
            forward_signal(process_group, signal);
            let _ = terminate_parent(signal);
        }
    })
}

#[cfg(unix)]
impl Drop for ProcessTree {
    fn drop(&mut self) {
        if let Some(handle) = self.signal_handle.take() {
            handle.close();
        }
        if let Some(thread) = self.signal_thread.take() {
            let _ = thread.join();
        }
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
