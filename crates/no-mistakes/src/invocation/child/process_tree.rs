use std::process::Command;

pub(crate) fn configure_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(not(unix))]
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
    pub(crate) fn attach(child: &std::process::Child) -> Self {
        let process_group = child.id() as i32;
        let listener = signal_hook::iterator::Signals::new([
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGTERM,
        ])
        .ok()
        .map(|mut signals| {
            let handle = signals.handle();
            let thread = std::thread::spawn(move || {
                if let Some(signal) = signals.forever().next() {
                    forward_signal(process_group, signal);
                    // Forwarding must not replace the invoking process's normal
                    // signal semantics. This runs outside the OS signal handler,
                    // so restoring and emulating the default action is safe.
                    let _ = signal_hook::low_level::emulate_default_handler(signal);
                }
            });
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
