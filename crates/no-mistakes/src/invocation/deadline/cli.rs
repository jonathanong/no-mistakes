use super::*;

impl DeadlineGuard {
    pub(in crate::invocation) fn install_for_cli_with_output(
        timeout: Option<Duration>,
        limit: Option<Deadline>,
        output: crate::invocation::CliErrorOutput,
    ) -> Result<Self> {
        let mut guard = Self::install_for_invocation_with_deadline(timeout, None, limit)?;
        let active = active_deadline()
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .copied();
        let Some(deadline) = active else {
            return Ok(guard);
        };
        let expires_at = deadline.expires_at;
        let timeout = deadline.timeout;
        let cancel = Arc::new(AtomicBool::new(false));
        let watcher = Arc::clone(&cancel);
        let observer = crate::diagnostics::current();
        std::thread::Builder::new()
            .name("no-mistakes-timeout".to_string())
            .spawn(move || watch::watch_deadline(watcher, expires_at, timeout, observer, output))
            .context("starting command timeout watchdog")?;
        guard.cancel_watch = Some(cancel);
        Ok(guard)
    }
}
