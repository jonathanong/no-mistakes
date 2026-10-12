use super::*;

/// Run every configured `check` domain and serialize the stable public report.
pub fn check_json(root: &Path) -> Result<String> {
    crate::ast::with_request_parse_cache(|| {
        let results = crate::check_runner::run_all_with_evidence(
            root.to_path_buf(),
            None,
            None,
            false,
            false,
        )?;
        Ok(serde_json::to_string(&crate::check_runner::json_value(
            &results,
        ))?)
    })
}

/// Run the aggregate check with a scoped observer and return its internal
/// diagnostics without writing stderr.
pub fn check_json_observed(
    root: &Path,
    verbose: bool,
) -> Result<(String, crate::diagnostics::DiagnosticsSnapshot)> {
    let observer = crate::diagnostics::InvocationObserver::new(verbose);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        check_json(root)?
    };
    Ok((output, observer.snapshot()))
}

/// Run the same multi-report engine used by the asynchronous N-API task.
pub fn analyze_project_json(options_json: String) -> napi::Result<String> {
    crate::ast::with_request_parse_cache(|| {
        crate::napi_api::analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            options_json,
        ))
    })
}

/// Run the multi-report engine with an explicitly scoped observer. The N-API
/// response remains identical; diagnostics are returned only to the harness.
pub fn analyze_project_json_observed(
    options_json: String,
) -> napi::Result<(String, crate::diagnostics::DiagnosticsSnapshot)> {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json(options_json)?
    };
    Ok((output, observer.snapshot()))
}
