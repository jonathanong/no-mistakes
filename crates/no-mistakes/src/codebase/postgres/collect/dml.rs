use super::{compile_sql_include, matches_sql_include, read_source};
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlKind};
use crate::codebase::postgres::statement_facts::SqlStatementFileFacts;
use crate::codebase::postgres::statements::extract_sql_statement_facts;
use crate::codebase::postgres::types::{PostgresFactError, PostgresSchemaOptions};
use crate::codebase::ts_source::SourceStore;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn collect(
    root: &Path,
    sources: &SourceStore,
    files: &[PathBuf],
    schema_options: &PostgresSchemaOptions,
    embedded: &[EmbeddedSqlFileFacts],
) -> Result<Vec<SqlStatementFileFacts>, PostgresFactError> {
    let globs = compile_sql_include(&schema_options.sql_include)?;
    let mut facts = files
        .par_iter()
        .filter(|path| matches_sql_include(root, path, &globs))
        .map(|path| {
            let source = read_source(path, sources)?;
            let mut file = extract_sql_statement_facts(&source);
            file.path = path.clone();
            Ok(file)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for file in embedded {
        facts.extend(embedded_call_facts(file));
    }
    facts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(facts)
}

fn embedded_call_facts(file: &EmbeddedSqlFileFacts) -> Vec<SqlStatementFileFacts> {
    file.calls
        .iter()
        .filter(|call| call.kind != EmbeddedSqlKind::Dynamic)
        .filter_map(|call| {
            let sql = call.sql_text.as_deref()?;
            let mut facts = extract_sql_statement_facts(sql);
            facts.path = file.path.clone();
            rebase_embedded_lines(&mut facts, call);
            Some(facts)
        })
        .collect()
}

fn rebase_embedded_lines(facts: &mut SqlStatementFileFacts, call: &EmbeddedSqlCall) {
    let base = match call.kind {
        EmbeddedSqlKind::Inline => call.line,
        _ => call.declaration_line.unwrap_or(call.line),
    } as usize;
    facts.origin_line = base;
    let shift = base.saturating_sub(1);
    for insert in &mut facts.inserts {
        insert.line = insert.line.saturating_add(shift);
    }
    for select in &mut facts.selects {
        select.line = select.line.saturating_add(shift);
    }
    for trigger in &mut facts.triggers {
        trigger.line = trigger.line.saturating_add(shift);
    }
}
