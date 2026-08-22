use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{
    collect_postgres_facts, sql_has_offset_clause, PostgresSchemaOptions,
};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &PostgresSchemaOptions::default(),
        &opts.embedded,
    )
    .with_context(|| format!("{RULE_ID} failed to collect embedded SQL facts"))?;
    let mut findings = Vec::new();
    for file in facts.embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            findings.extend(findings_for_call(&rel, call));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

pub(super) fn findings_for_call(
    file: &str,
    call: &crate::codebase::postgres::EmbeddedSqlCall,
) -> Vec<RuleFinding> {
    let Some(sql) = call.sql_text.as_deref() else {
        return Vec::new();
    };
    match sql_has_offset_clause(sql) {
        Ok(true) => vec![RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.to_string(),
            line: call.line as usize,
            message: format!(
                "{file}:{}: do not use SQL OFFSET; use cursor pagination, LIMIT + 1, COUNT, EXISTS, or ROW_NUMBER() instead",
                call.line
            ),
            import: None,
            target: Some("offset".to_string()),
        }],
        Ok(false) | Err(_) => Vec::new(),
    }
}
