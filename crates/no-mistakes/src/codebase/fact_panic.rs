use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

pub(crate) fn isolate<T>(
    session: &super::analysis_session::AnalysisSession,
    path: &Path,
    collect: impl FnOnce() -> T,
) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(|| {
        #[cfg(test)]
        tests::inject(path);
        collect()
    }))
    .map_err(|payload| {
        crate::ast::evict_request_parse_cache_path(path);
        let message = format!(
            "fact extraction panicked for {}: {}. Report this analyzer bug; analysis is incomplete",
            path.display(),
            panic_message(payload.as_ref())
        );
        session.record_check_source_failure(path, &message);
        session.record_check_failure(super::check_outcome::CheckFailure {
            consumer: "fact-extraction".into(),
            file: Some(path.to_path_buf()),
            property: None,
            message: message.clone(),
        });
        message
    })
}

fn panic_message(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("non-string panic payload")
}

#[cfg(test)]
pub(crate) mod tests;
