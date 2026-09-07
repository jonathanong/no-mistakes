use super::{compile_sql_include, matches_sql_include, read_source};
use crate::codebase::postgres::embedded::{EmbeddedSqlFileFacts, EmbeddedSqlKind};
use crate::codebase::postgres::statement_facts::SqlStatementFileFacts;
use crate::codebase::postgres::statements::extract_sql_statement_facts;
use crate::codebase::postgres::types::{PostgresFactError, PostgresSchemaOptions};
use crate::codebase::ts_source::SourceStore;
use std::path::{Path, PathBuf};

pub(super) fn collect(
    root: &Path,
    sources: &SourceStore,
    files: &[PathBuf],
    schema_options: &PostgresSchemaOptions,
    embedded: &[EmbeddedSqlFileFacts],
) -> Result<Vec<SqlStatementFileFacts>, PostgresFactError> {
    let globs = compile_sql_include(&schema_options.sql_include)?;
    let mut facts = Vec::new();
    for path in files {
        if !matches_sql_include(root, path, &globs) {
            continue;
        }
        let source = read_source(path, sources)?;
        let mut file = extract_sql_statement_facts(&source);
        file.path = path.clone();
        facts.push(file);
    }
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
            Some(facts)
        })
        .collect()
}
