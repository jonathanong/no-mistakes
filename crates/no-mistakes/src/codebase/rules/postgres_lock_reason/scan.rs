use super::comments_index::CommentIndex;
use super::findings::{
    is_lock_category, load_ts_comments, matches_sql_helper, push_reason_finding,
    statement_has_lock_work, unanalyzable_finding, FileOutcome, PARSE_GAP, ROUTINE_GAP,
};
use super::options::CompiledOptions;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::postgres::{compile_sql_include, matches_sql_include, SqlStatementFileFacts};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn collect(
    root: &Path,
    options: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
) -> Result<(Vec<RuleFinding>, Vec<RuleFinding>)> {
    let sql = compile_sql_include(&options.schema.sql_include)?;
    let rows = files
        .par_iter()
        .filter_map(|path| {
            let rel = relative_slash_path(root, path);
            if path.extension().is_some_and(|ext| ext == "sql")
                && matches_sql_include(root, path, &sql)
            {
                Some(native(options, path, &rel, sources, facts))
            } else if is_indexable(path) && (options.has_helpers() || options.scans_executors()) {
                Some(typescript(options, path, &rel, sources, facts))
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>>>()?;
    let mut findings = Vec::new();
    let mut unanalyzable = Vec::new();
    for row in rows {
        findings.extend(row.findings);
        unanalyzable.extend(row.unanalyzable);
    }
    Ok((findings, unanalyzable))
}

fn native(
    options: &CompiledOptions,
    path: &Path,
    rel: &str,
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
) -> Result<FileOutcome> {
    let mut out = FileOutcome::default();
    let Some(statements) = facts.prepared().postgres_readable_statements(path, None)? else {
        out.unanalyzable
            .push(unanalyzable_finding(rel, 1, "SQL source could not be read"));
        return Ok(out);
    };
    let mut comment_index: Option<CommentIndex> = None;
    for statement in statements {
        record_sql_gaps(rel, statement, &mut out.unanalyzable);
        if statement.parse_failed || !statement_has_lock_work(options, statement) {
            continue;
        }
        let comments = match &comment_index {
            Some(index) => index,
            None => {
                let Ok(source) = sources.read_source_path(path) else {
                    out.unanalyzable.push(unanalyzable_finding(
                        rel,
                        1,
                        "SQL source could not be read",
                    ));
                    return Ok(out);
                };
                comment_index.insert(CommentIndex::for_sql(&source))
            }
        };
        for lock in statement
            .lock_inventory
            .iter()
            .filter(|lock| is_lock_category(&lock.category))
        {
            let status = comments.status_for_sql_statement(
                lock.statement_start_line,
                lock.statement_start_column,
                lock.statement_end_line,
            );
            push_reason_finding(rel, lock.line, &lock.category, status, &mut out.findings);
        }
        for call in statement
            .function_calls
            .iter()
            .filter(|call| matches_sql_helper(options, call))
        {
            let status = comments.status_for_sql_line(call.line, call.column);
            push_reason_finding(rel, call.line, "helper", status, &mut out.findings);
        }
    }
    Ok(out)
}

fn typescript(
    options: &CompiledOptions,
    path: &Path,
    rel: &str,
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
) -> Result<FileOutcome> {
    let mut out = FileOutcome::default();
    let Some(file) = facts
        .prepared()
        .ts
        .get(path)
        .filter(|file| file.parse_error.is_none())
    else {
        out.unanalyzable.push(unanalyzable_finding(
            rel,
            1,
            "TypeScript source could not be analyzed",
        ));
        return Ok(out);
    };
    let mut ts_comments: Option<CommentIndex> = None;
    if options.has_helpers() {
        for call in &file.ts.call_sites {
            let terminal = call.callee.rsplit('.').next().unwrap_or(&call.callee);
            if !options.matches_helper(&call.callee, terminal) {
                continue;
            }
            let Some(comments) = load_ts_comments(path, sources, &mut ts_comments) else {
                out.unanalyzable.push(unanalyzable_finding(
                    rel,
                    call.line as usize,
                    "TypeScript source could not be read",
                ));
                return Ok(out);
            };
            let status = comments.status_for_ts_line(call.line as usize);
            push_reason_finding(rel, call.line as usize, "helper", status, &mut out.findings);
        }
    }
    if options.scans_executors() {
        super::embedded::collect(
            options,
            path,
            rel,
            sources,
            facts,
            &mut ts_comments,
            &mut out,
        )?;
    }
    Ok(out)
}

fn record_sql_gaps(rel: &str, statement: &SqlStatementFileFacts, out: &mut Vec<RuleFinding>) {
    for (_, line) in statement.parse_failure_sites() {
        out.push(unanalyzable_finding(rel, line, PARSE_GAP));
    }
    for &line in &statement.lock_inventory_unanalyzed {
        out.push(unanalyzable_finding(rel, line, ROUTINE_GAP));
    }
}
