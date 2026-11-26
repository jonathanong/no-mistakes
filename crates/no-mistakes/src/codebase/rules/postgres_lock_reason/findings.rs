use super::comment::{reason_message, ReasonStatus};
use super::comments_index::CommentIndex;
use super::options::CompiledOptions;
use super::{finding, RULE_ID};
use crate::codebase::postgres::{
    SqlFunctionCallFact, SqlStatementFileFacts, UndeclaredExecutorType,
};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(super) const PARSE_GAP: &str = "SQL could not be parsed completely";
pub(super) const ROUTINE_GAP: &str = "stored routine body could not be resolved";

#[derive(Default)]
pub(super) struct FileOutcome {
    pub(super) findings: Vec<RuleFinding>,
    pub(super) unanalyzable: Vec<RuleFinding>,
}

pub(super) fn load_ts_comments<'a>(
    path: &Path,
    sources: &SourceStore,
    cache: &'a mut Option<CommentIndex>,
) -> Option<&'a CommentIndex> {
    if cache.is_none() {
        let source = sources.read_source_path(path).ok()?;
        *cache = Some(CommentIndex::for_ts(&source));
    }
    cache.as_ref()
}

pub(super) fn is_lock_category(category: &str) -> bool {
    !matches!(category, "nowait" | "skip-locked")
}

pub(super) fn matches_sql_helper(options: &CompiledOptions, call: &SqlFunctionCallFact) -> bool {
    let Some(terminal) = call.name_parts.last() else {
        return false;
    };
    let name = call.name_parts.join(".");
    options.matches_sql_helper(&name, terminal)
}

pub(super) fn statement_has_lock_work(
    options: &CompiledOptions,
    statement: &SqlStatementFileFacts,
) -> bool {
    statement
        .lock_inventory
        .iter()
        .any(|lock| is_lock_category(&lock.category))
        || statement
            .function_calls
            .iter()
            .any(|call| matches_sql_helper(options, call))
}

pub(super) fn push_reason_finding(
    rel: &str,
    line: usize,
    category: &str,
    status: ReasonStatus,
    out: &mut Vec<RuleFinding>,
) {
    if status != ReasonStatus::Valid {
        let line = line.max(1);
        out.push(finding(
            rel.to_owned(),
            line,
            reason_message(rel, line, category, status),
            category.to_owned(),
        ));
    }
}

pub(super) fn unanalyzable_finding(rel: &str, line: usize, reason: &str) -> RuleFinding {
    let line = line.max(1);
    finding(
        rel.to_owned(),
        line,
        format!(
            "{rel}:{line}: cannot verify {RULE_ID} ({reason}). Make the configured SQL or helper call statically analyzable, suppress with `no-mistakes-disable-next-line {RULE_ID}: reason`, or set options.unanalyzable to advise or ignore"
        ),
        "unanalyzable".into(),
    )
}

pub(super) fn push_undeclared_executors(
    rel: &str,
    undeclared: &[UndeclaredExecutorType],
    out: &mut Vec<RuleFinding>,
) {
    for item in undeclared {
        let reason = if item.configured_type {
            format!(
                "configured executor receiver `{}` cannot be statically resolved",
                item.receiver_name
            )
        } else if item.known_type {
            format!(
                "undeclared executor type `{}` from `{}`",
                item.type_name, item.module
            )
        } else {
            format!(
                "receiver `{}` has no declared executor type",
                item.receiver_name
            )
        };
        out.push(unanalyzable_finding(rel, item.line as usize, &reason));
    }
}
