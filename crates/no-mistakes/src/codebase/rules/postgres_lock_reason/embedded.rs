use super::comment::ReasonStatus;
use super::comments_index::CommentIndex;
use super::findings::{
    is_lock_category, load_ts_comments, matches_sql_helper, push_reason_finding,
    push_undeclared_executors, unanalyzable_finding, FileOutcome, PARSE_GAP, ROUTINE_GAP,
};
use super::options::CompiledOptions;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlKind, SqlStatementFileFacts};
use crate::codebase::rules::postgres_lock_inventory::embedded::Positions;
use crate::codebase::ts_source::SourceStore;
use anyhow::Result;
use std::path::Path;

struct EmbeddedCtx<'a> {
    options: &'a CompiledOptions,
    path: &'a Path,
    rel: &'a str,
    sources: &'a SourceStore,
    ts_comments: &'a mut Option<CommentIndex>,
    out: &'a mut FileOutcome,
}

pub(super) fn collect(
    options: &CompiledOptions,
    path: &Path,
    rel: &str,
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
    ts_comments: &mut Option<CommentIndex>,
    out: &mut FileOutcome,
) -> Result<()> {
    let embedded = facts.embedded_sql(path, &options.embedded)?;
    let mut statements = facts.postgres_statements(path, Some(&options.embedded))?;
    push_undeclared_executors(
        rel,
        &embedded.undeclared_executor_types,
        &mut out.unanalyzable,
    );
    let mut ctx = EmbeddedCtx {
        options,
        path,
        rel,
        sources,
        ts_comments,
        out,
    };
    for call in &embedded.calls {
        let version_count = if call.variants.is_empty() {
            usize::from(call.sql_text.is_some())
        } else {
            call.variants.len()
        };
        let (versions, remaining) = statements.split_at(version_count);
        statements = remaining;
        if call.variants.is_empty()
            && (versions.is_empty() || call.kind == EmbeddedSqlKind::Dynamic)
        {
            ctx.out.unanalyzable.push(unanalyzable_finding(
                ctx.rel,
                call.line as usize,
                "executor SQL could not be resolved",
            ));
            continue;
        }
        for (variant_index, statement) in versions.iter().enumerate() {
            check_variant(&mut ctx, call, variant_index, statement);
        }
    }
    debug_assert!(statements.is_empty());
    Ok(())
}

fn check_variant(
    ctx: &mut EmbeddedCtx<'_>,
    call: &EmbeddedSqlCall,
    variant_index: usize,
    statement: &SqlStatementFileFacts,
) {
    if call
        .variants
        .get(variant_index)
        .is_some_and(|variant| variant.variants_truncated)
    {
        ctx.out.unanalyzable.push(unanalyzable_finding(
            ctx.rel,
            call.line as usize,
            "executor SQL alternatives are incomplete",
        ));
        return;
    }
    for (site, line) in statement.parse_failure_sites() {
        let imported = statement
            .variant_locations
            .as_ref()
            .and_then(|locations| locations.position(site))
            .is_some_and(|position| position.source_path.is_some());
        let report_line = if imported { call.line as usize } else { line };
        ctx.out
            .unanalyzable
            .push(unanalyzable_finding(ctx.rel, report_line, PARSE_GAP));
    }
    let mut positions = None;
    for &line in &statement.lock_inventory_unanalyzed {
        let (_, report_line, _) = positions
            .get_or_insert_with(|| Positions::new(call, variant_index))
            .source(line, 1);
        ctx.out
            .unanalyzable
            .push(unanalyzable_finding(ctx.rel, report_line, ROUTINE_GAP));
    }
    if statement.parse_failed {
        return;
    }
    let has_locks = statement
        .lock_inventory
        .iter()
        .any(|lock| is_lock_category(&lock.category))
        || statement
            .function_calls
            .iter()
            .any(|function| matches_sql_helper(ctx.options, function));
    if !has_locks {
        return;
    }
    let sql_comments = CommentIndex::for_sql(variant_sql_text(call, variant_index));
    let ts_index = load_ts_comments(ctx.path, ctx.sources, ctx.ts_comments);
    let call_status = ts_index.map_or(ReasonStatus::Absent, |comments| {
        comments.status_for_ts_line(call.line as usize)
    });
    let positions = positions.get_or_insert_with(|| Positions::new(call, variant_index));
    for lock in statement
        .lock_inventory
        .iter()
        .filter(|lock| is_lock_category(&lock.category))
    {
        let (source_file, report_line, _) = positions.source(lock.line, lock.column);
        let (_, stmt_source_line, _) =
            positions.source(lock.statement_start_line, lock.statement_start_column);
        let mut status = sql_comments
            .status_for_sql_statement(
                lock.statement_start_line,
                lock.statement_start_column,
                lock.statement_end_line,
            )
            .merge(call_status);
        if source_file.is_none() && stmt_source_line != call.line as usize {
            if let Some(comments) = ts_index {
                status = status.merge(comments.status_for_ts_line(stmt_source_line));
            }
        }
        push_reason_finding(
            ctx.rel,
            report_line,
            &lock.category,
            status,
            &mut ctx.out.findings,
        );
    }
    for (index, function) in statement.function_calls.iter().enumerate() {
        if !matches_sql_helper(ctx.options, function) {
            continue;
        }
        let imported = statement
            .variant_locations
            .as_ref()
            .and_then(|locations| locations.position(SqlFactSite::Function(index)))
            .filter(|position| position.source_path.is_some());
        let report_line = if imported.is_some() {
            call.line as usize
        } else {
            positions.source(function.line, function.column).1
        };
        let mut status = sql_comments
            .status_for_sql_line(function.line, function.column)
            .merge(call_status);
        if imported.is_none() && report_line != call.line as usize {
            if let Some(comments) = ts_index {
                status = status.merge(comments.status_for_ts_line(report_line));
            }
        }
        push_reason_finding(
            ctx.rel,
            report_line,
            "helper",
            status,
            &mut ctx.out.findings,
        );
    }
}

fn variant_sql_text(call: &EmbeddedSqlCall, variant_index: usize) -> &str {
    call.variants
        .get(variant_index)
        .filter(|_| !call.mirrors_single_variant())
        .map(|variant| variant.sql_text.as_str())
        .unwrap_or_else(|| call.sql_text.as_deref().unwrap_or_default())
}
