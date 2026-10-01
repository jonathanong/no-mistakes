use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{
    collect_postgres_facts, sql_file_offset_uses, sql_offset_uses, OffsetUse, PostgresSchemaOptions,
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
    for path in files {
        let rel = relative_slash_path(root, path);
        if !rel.ends_with(".sql") || !opts.sql_include.is_match(&rel) {
            continue;
        }
        let Some(sql) = crate::codebase::rules::read_source(sources, path) else {
            continue;
        };
        findings.extend(findings_for_sql(&rel, &sql));
    }
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
    match sql_offset_uses(sql) {
        Ok(uses) => uses
            .into_iter()
            .map(|use_| finding(file, call.line as usize, use_))
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn findings_for_sql(file: &str, sql: &str) -> Vec<RuleFinding> {
    sql_file_offset_uses(sql)
        .into_iter()
        .map(|(line, use_)| finding(file, line, use_))
        .collect()
}

fn finding(file: &str, line: usize, use_: OffsetUse) -> RuleFinding {
    let text = match use_ {
        OffsetUse::Zero => {
            "OFFSET 0 used as an optimizer fence; use a MATERIALIZED CTE (WITH x AS MATERIALIZED (...))"
        }
        OffsetUse::Other => {
            "do not use SQL OFFSET; use cursor pagination, LIMIT + 1, COUNT, EXISTS, or ROW_NUMBER() instead"
        }
    };
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {text}"),
        import: None,
        target: Some("offset".to_string()),
    }
}
