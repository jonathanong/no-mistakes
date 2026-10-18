use super::*;

impl ParsedProgramCache {
    /// Exposes OXC's recovered program, diagnostic, and fatal-panic status.
    pub(crate) fn with_recovered_program_status_observed<T>(
        &self,
        path: &Path,
        source: Arc<str>,
        on_parse: impl FnOnce(),
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>, bool) -> T,
    ) -> Result<T, String> {
        self.with_cached_program(path, source, ParseMode::Standard, on_parse, |cached| {
            Ok(cached.with_dependent(|owner, parsed| {
                analyze(
                    &parsed.program,
                    owner.source.as_ref(),
                    parsed.diagnostic_error.clone(),
                    parsed.panic_error.is_some(),
                )
            }))
        })
    }

    pub(crate) fn with_recovered_typescript_program_observed<T>(
        &self,
        path: &Path,
        source: Arc<str>,
        on_parse: impl FnOnce(),
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>) -> T,
    ) -> Result<T, String> {
        self.with_cached_program(
            path,
            source,
            ParseMode::TypeScriptFallback,
            on_parse,
            |cached| {
                Ok(cached.with_dependent(|owner, parsed| {
                    analyze(
                        &parsed.program,
                        owner.source.as_ref(),
                        parsed.diagnostic_error.clone(),
                    )
                }))
            },
        )
    }

    pub(crate) fn with_legacy_symbols_program_observed<T>(
        &self,
        path: &Path,
        source: Arc<str>,
        on_parse: impl FnOnce(),
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>) -> T,
    ) -> Result<T, String> {
        let mode = if legacy_symbols_share_standard_parse(path) {
            ParseMode::Standard
        } else {
            ParseMode::LegacySymbols
        };
        self.with_cached_program(path, source, mode, on_parse, |cached| {
            cached.with_dependent(|owner, parsed| match &parsed.panic_error {
                Some(error) => Err(error.clone()),
                None => Ok(analyze(
                    &parsed.program,
                    owner.source.as_ref(),
                    parsed.diagnostic_error.clone(),
                )),
            })
        })
    }
}
