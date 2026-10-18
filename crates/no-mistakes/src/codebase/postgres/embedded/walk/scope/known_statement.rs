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

pub(crate) fn has_known_leading_statement(mut sql: &str) -> bool {
    loop {
        sql = sql.trim_start();
        if let Some(comment) = sql.strip_prefix("--") {
            let Some((_, rest)) = comment.split_once('\n') else {
                return false;
            };
            sql = rest;
            continue;
        }
        if let Some(comment) = sql.strip_prefix("/*") {
            let Some((_, rest)) = comment.split_once("*/") else {
                return false;
            };
            sql = rest;
            continue;
        }
        break;
    }

    let keyword = sql
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .next()
        .unwrap_or_default();
    ["select", "update", "insert", "delete", "merge"]
        .iter()
        .any(|candidate| keyword.eq_ignore_ascii_case(candidate))
}
