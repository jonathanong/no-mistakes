use super::{CompiledOptions, LockInventoryReport};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::postgres::{
    compile_sql_include, matches_sql_include, SqlFunctionCallFact, SqlStatementFileFacts,
};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct FileInventory {
    pub(super) sites: Vec<Occurrence>,
    pub(super) gaps: Vec<(usize, &'static str)>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Occurrence {
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) category: String,
    pub(super) fingerprint: String,
}

pub(super) fn collect(
    root: &Path,
    options: &CompiledOptions,
    files: &[PathBuf],
    facts: &dyn SqlFactContext,
) -> Result<LockInventoryReport> {
    let sql = compile_sql_include(&options.schema.sql_include)?;
    let rows = files
        .par_iter()
        .filter_map(|path| {
            if path.extension().is_some_and(|ext| ext == "sql")
                && matches_sql_include(root, path, &sql)
            {
                Some(native(options, path, facts))
            } else if is_indexable(path) && (options.has_helpers() || options.scans_executors()) {
                Some(typescript(options, path, facts))
            } else {
                None
            }
            .map(|row| row.map(|row| (relative_slash_path(root, path), row)))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(super::report::build(options, rows))
}

fn native(
    options: &CompiledOptions,
    path: &Path,
    facts: &dyn SqlFactContext,
) -> Result<FileInventory> {
    let mut row = FileInventory::default();
    let Some(statements) = facts.prepared().postgres_readable_statements(path, None)? else {
        row.gaps.push((1, "SQL source could not be read"));
        return Ok(row);
    };
    for statement in statements {
        if !record_parse_failures(statement, &mut row, std::convert::identity) {
            row.sites.extend(sql_sites(options, statement));
        }
    }
    Ok(row)
}

fn typescript(
    options: &CompiledOptions,
    path: &Path,
    facts: &dyn SqlFactContext,
) -> Result<FileInventory> {
    let mut row = FileInventory::default();
    let Some(file) = facts
        .prepared()
        .ts
        .get(path)
        .filter(|file| file.parse_error.is_none())
    else {
        row.gaps
            .push((1, "TypeScript source could not be analyzed"));
        return Ok(row);
    };
    if options.has_helpers() {
        for call in &file.ts.call_sites {
            let terminal = call.callee.rsplit('.').next().unwrap_or(&call.callee);
            if options.matches_helper(&call.callee, terminal) {
                row.sites.push(Occurrence {
                    line: call.line as usize,
                    column: call.offset as usize,
                    category: "helper".into(),
                    fingerprint: format!(
                        "ts:{}:{}:{}",
                        call.callee,
                        call.arg_count,
                        call.args.join(",")
                    ),
                });
            }
        }
    }
    if options.scans_executors() {
        super::embedded::collect(options, path, facts, &mut row)?;
    }
    Ok(row)
}

pub(super) fn record_parse_failures(
    statement: &SqlStatementFileFacts,
    row: &mut FileInventory,
    mut routine_source_line: impl FnMut(usize) -> usize,
) -> bool {
    row.gaps.extend(
        statement
            .parse_failure_sites()
            .map(|(_, line)| (line, "SQL could not be parsed completely")),
    );
    row.gaps
        .extend(statement.lock_inventory_unanalyzed.iter().map(|&line| {
            (
                routine_source_line(line),
                "stored routine body could not be resolved",
            )
        }));
    statement.parse_failed || !statement.lock_inventory_unanalyzed.is_empty()
}

pub(super) fn sql_sites<'a>(
    options: &'a CompiledOptions,
    statement: &'a SqlStatementFileFacts,
) -> impl Iterator<Item = Occurrence> + 'a {
    statement
        .lock_inventory
        .iter()
        .map(|lock| Occurrence {
            line: lock.line,
            column: lock.column,
            category: lock.category.clone(),
            fingerprint: lock.fingerprint.clone(),
        })
        .chain(
            statement
                .function_calls
                .iter()
                .filter_map(|call| sql_helper(options, call)),
        )
}

pub(super) fn sql_helper(
    options: &CompiledOptions,
    call: &SqlFunctionCallFact,
) -> Option<Occurrence> {
    let terminal = call.name_parts.last()?;
    let name = call.name_parts.join(".");
    options
        .matches_sql_helper(&name, terminal)
        .then(|| Occurrence {
            line: call.line,
            column: call.column,
            category: "helper".into(),
            fingerprint: format!("sql:{:?}", call.name_parts),
        })
}
