//! Unstable N-API adapters used only by the checked-in performance harness.

/// Run the same multi-report engine used by the asynchronous N-API task.
pub fn analyze_project_json(options_json: String) -> napi::Result<String> {
    no_mistakes::ast::with_request_parse_cache(|| {
        crate::analyze_project_json_impl(crate::options::test_json_arg(options_json))
    })
}

/// Run the multi-report engine with an explicitly scoped observer. The N-API
/// response remains identical; diagnostics are returned only to the harness.
pub fn analyze_project_json_observed(
    options_json: String,
) -> napi::Result<(String, no_mistakes::diagnostics::DiagnosticsSnapshot)> {
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json(options_json)?
    };
    Ok((output, observer.snapshot()))
}

#[cfg(test)]
mod tests;
