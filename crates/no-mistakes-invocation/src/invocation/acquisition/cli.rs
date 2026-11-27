use super::*;

impl InvocationGuard {
    pub fn acquire_for_cli(options: InvocationOptions) -> Result<Self> {
        Self::acquire_for_cli_with_json(options, false)
    }

    /// CLI-only output preference for hard deadline failures.
    #[doc(hidden)]
    pub fn acquire_for_cli_with_json(options: InvocationOptions, json: bool) -> Result<Self> {
        Self::acquire_for_cli_with_output(options, CliErrorOutput::new(json))
    }

    #[doc(hidden)]
    pub fn acquire_for_cli_with_output(
        options: InvocationOptions,
        output: CliErrorOutput,
    ) -> Result<Self> {
        Self::acquire_resolving_lock(options, true, || Ok(LockTarget::Current), output)
    }
    #[doc(hidden)]
    pub fn acquire_for_cli_at_path(
        options: InvocationOptions,
        path: &std::path::Path,
    ) -> Result<Self> {
        Self::acquire_resolving_path(options, true, || Ok(path.to_path_buf()))
    }
}
