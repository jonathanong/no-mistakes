use super::*;

impl DeadlineGuard {
    /// CLI invocations enforce the deadline even when native analysis is inside
    /// a CPU-bound rayon section that has not reached the next `check_timeout`.
    pub(in crate::invocation) fn install_for_cli(timeout: Option<Duration>) -> Result<Self> {
        Self::install_for_cli_with_deadline(timeout, None)
    }

    pub(in crate::invocation) fn install_for_cli_with_deadline(
        timeout: Option<Duration>,
        limit: Option<Deadline>,
    ) -> Result<Self> {
        Self::install_for_cli_with_output(
            timeout,
            limit,
            crate::invocation::CliErrorOutput::new(false),
        )
    }
}
