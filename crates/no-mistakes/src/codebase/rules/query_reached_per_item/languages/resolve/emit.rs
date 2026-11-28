use super::super::Ctx;
use crate::codebase::rules::{FindingCause, RuleFinding};
use crate::config::v2::Unanalyzable;

const RULE_ID: &str = super::super::super::RULE_ID;
const UNPROVEN_DETAIL: &str =
    "the target is not a configured sink or a local function whose calls are proven";

pub(super) fn proven(
    file: &str,
    line: u32,
    callee: &str,
    construct: &str,
    kind: &str,
    path: &str,
    message: &str,
) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: line as usize,
        message: format!("{message}: {construct} `{callee}` reaches `{kind}` via {path}"),
        import: Some(callee.to_string()),
        target: Some(format!("{kind}: {path}")),
    }
}

pub(super) fn unproven(
    file: &str,
    line: u32,
    callee: &str,
    construct: &str,
    ctx: &Ctx<'_>,
) -> Option<RuleFinding> {
    if ctx.unanalyzable == Unanalyzable::Ignore {
        return None;
    }
    let report = ctx.unanalyzable == Unanalyzable::Report;
    Some(RuleFinding {
        causes: if report {
            Vec::new()
        } else {
            vec![FindingCause {
                kind: "unresolved-dispatch",
                file: file.to_string(),
                line: Some(line as usize),
                callee: Some(callee.to_string()),
                detail: UNPROVEN_DETAIL.to_string(),
            }]
        },
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: line as usize,
        message: format!(
            "per-item call is unanalyzable: {construct} `{callee}` has no proven target"
        ),
        import: Some(callee.to_string()),
        target: None,
    })
}

pub(super) fn suppressed(
    ctx: &Ctx<'_>,
    file: &str,
    callee: &str,
    line: u32,
    function: &str,
) -> bool {
    let matches = ctx
        .allow
        .iter()
        .map(|entry| allowance_matches(entry, file, callee, line, function));
    if let Some(usage) = ctx.usage {
        usage.suppress(matches)
    } else {
        matches.into_iter().any(|matched| matched)
    }
}

pub(super) fn allowance_matches(
    entry: &super::super::super::config::Allowance,
    file: &str,
    callee: &str,
    line: u32,
    function: &str,
) -> bool {
    entry.file == file
        && entry.callee.as_ref().is_none_or(|value| value == callee)
        && entry.line.is_none_or(|value| value == line)
        && entry
            .function
            .as_deref()
            .is_none_or(|name| name == function)
}
