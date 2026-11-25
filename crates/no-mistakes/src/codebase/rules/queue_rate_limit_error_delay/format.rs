use super::options::Target;
use super::RULE_ID;
use crate::codebase::dependencies::graph::ResolvedCallSite;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::RuleDef;
use std::path::Path;

pub(super) fn missing_finding(
    root: &Path,
    application: &RuleDef,
    file: &Path,
    site: &ResolvedCallSite,
    target: &Target,
    detail: &str,
) -> RuleFinding {
    let body = format!(
        "`{}` ({}) {detail}; throwing a rate-limit error with no delay drops the provider's Retry-After — pass the positive delay duration",
        site.source_callee,
        target.label()
    );
    let message = match application.message.as_deref() {
        Some(prefix) => format!("{prefix}: {body}"),
        None => body,
    };
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, file),
        line: site.line.max(1) as usize,
        message,
        import: Some(site.source_callee.clone()),
        target: Some(target.label()),
    }
}

pub(super) fn unanalyzable_finding(
    root: &Path,
    application: &RuleDef,
    file: &Path,
    line: u32,
    site: Option<&ResolvedCallSite>,
    detail: &str,
) -> RuleFinding {
    let body = format!(
        "{RULE_ID} could not verify delay ({detail}); pass a statically visible positive delay or suppress with a reason"
    );
    let message = match application.message.as_deref() {
        Some(prefix) => format!("{prefix}: {body}"),
        None => body,
    };
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, file),
        line: line.max(1) as usize,
        message,
        import: site.map(|s| s.source_callee.clone()),
        target: Some("unanalyzable".to_string()),
    }
}
