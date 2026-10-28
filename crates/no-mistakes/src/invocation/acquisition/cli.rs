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
        Self::acquire_resolving_path_with_output(options, true, lock_path, output)
    }
}
