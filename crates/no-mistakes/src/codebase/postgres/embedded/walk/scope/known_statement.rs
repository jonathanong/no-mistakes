use super::{BindingState, EmbeddedSqlKind};

pub(in crate::codebase::postgres::embedded::walk) fn mark_binding_dynamic_keep_known_statement(
    binding: &mut BindingState,
) {
    binding.executor_alias = None;
    binding.kind = EmbeddedSqlKind::Dynamic;
    binding.condition_key = None;
    binding.variants = None;
    if binding
        .sql
        .as_deref()
        .is_none_or(|sql| !has_known_leading_statement(sql))
    {
        binding.sql = None;
    }
}

pub(crate) fn has_known_leading_statement(sql: &str) -> bool {
    is_known_keyword(leading_keyword(sql))
}

fn is_known_keyword(keyword: &str) -> bool {
    ["select", "update", "insert", "delete", "merge"]
        .iter()
        .any(|candidate| keyword.eq_ignore_ascii_case(candidate))
}

// Advisory visibility only: these statements do not become executor candidates.
pub(crate) fn has_diagnostic_leading_statement(sql: &str) -> bool {
    let keyword = leading_keyword(sql);
    is_known_keyword(keyword)
        || [
            "with",
            "begin",
            "commit",
            "rollback",
            "savepoint",
            "release",
            "create",
            "alter",
            "drop",
            "truncate",
            "explain",
            "deallocate",
            "prepare",
            "refresh",
            "show",
            "set",
        ]
        .iter()
        .any(|candidate| keyword.eq_ignore_ascii_case(candidate))
}

fn leading_keyword(mut sql: &str) -> &str {
    loop {
        sql = sql.trim_start();
        if let Some(comment) = sql.strip_prefix("--") {
            let Some((_, rest)) = comment.split_once('\n') else {
                return "";
            };
            sql = rest;
            continue;
        }
        if let Some(comment) = sql.strip_prefix("/*") {
            let Some((_, rest)) = comment.split_once("*/") else {
                return "";
            };
            sql = rest;
            continue;
        }
        break;
    }

    sql.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .next()
        .unwrap_or_default()
}
