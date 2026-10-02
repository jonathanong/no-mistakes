use super::{finding, trigger_finding, CompiledOptions};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::postgres::dml::{find_generated_column_writes, GeneratedTableColumns};
use crate::codebase::postgres::{collect_postgres_facts, EmbeddedSqlFileFacts};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Context;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
    let readable: Vec<PathBuf> = files
        .iter()
        .filter(|path| sources.read_path(path).is_ok())
        .cloned()
        .collect();
    let facts = collect_postgres_facts(
        root,
        sources,
        &readable,
        &CheckFactPlan {
            postgres_schema: true,
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &opts.embedded,
    )
    .with_context(|| format!("{} failed to collect PostgreSQL facts", super::RULE_ID))?;
    let catalog = super::catalog::catalog_from_facts(&facts.schema, &opts.extra_generated_columns);
    let trigger =
        super::catalog::trigger_catalog_from_facts(&facts.schema, &opts.trigger_maintained_columns);
    let mut findings =
        super::catalog::stale_extra_findings(&facts.schema, &opts.extra_generated_columns);
    findings.extend(super::catalog::stale_trigger_findings(
        &facts.schema,
        &opts.trigger_maintained_columns,
    ));
    if catalog.is_empty() && trigger.is_empty() {
        crate::codebase::rules::sort_findings(&mut findings);
        return Ok(findings);
    }
    let embedded_by_path: HashMap<&Path, &EmbeddedSqlFileFacts> = facts
        .embedded
        .iter()
        .map(|file| (file.path.as_path(), file))
        .collect();
    for path in files {
        if !opts.includes_dml(root, path) {
            continue;
        }
        let rel = relative_slash_path(root, path);
        if is_indexable(path) {
            findings.extend(scan_embedded(
                &rel,
                embedded_by_path.get(path.as_path()).copied(),
                &catalog,
                &trigger,
            ));
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("sql") {
            findings.extend(scan_sql_file(path, &rel, sources, &catalog, &trigger));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_embedded(
    file: &str,
    facts: Option<&EmbeddedSqlFileFacts>,
    catalog: &GeneratedTableColumns,
    trigger: &GeneratedTableColumns,
) -> Vec<RuleFinding> {
    let Some(facts) = facts else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for call in &facts.calls {
        let Some(sql) = call.sql_text.as_deref() else {
            continue;
        };
        let line = call.line.max(1) as usize;
        extend_writes(&mut findings, file, sql, catalog, trigger, |_, _| line);
    }
    findings
}

fn scan_sql_file(
    path: &Path,
    file: &str,
    sources: &SourceStore,
    catalog: &GeneratedTableColumns,
    trigger: &GeneratedTableColumns,
) -> Vec<RuleFinding> {
    let Some(source) = crate::codebase::rules::read_source(sources, path) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    extend_writes(
        &mut findings,
        file,
        &source,
        catalog,
        trigger,
        |table, column| line_for_write(&source, table, column),
    );
    findings
}

fn extend_writes(
    findings: &mut Vec<RuleFinding>,
    file: &str,
    sql: &str,
    catalog: &GeneratedTableColumns,
    trigger: &GeneratedTableColumns,
    line_for: impl Fn(&str, &str) -> usize,
) {
    for write in find_generated_column_writes(sql, catalog) {
        findings.push(finding(
            file,
            line_for(&write.table, &write.column),
            &write.table,
            &write.column,
        ));
    }
    for write in find_generated_column_writes(sql, trigger) {
        findings.push(trigger_finding(
            file,
            line_for(&write.table, &write.column),
            &write.table,
            &write.column,
        ));
    }
}

fn line_for_write(source: &str, table: &str, column: &str) -> usize {
    source
        .lines()
        .enumerate()
        .find(|(_, line)| {
            contains_ignore_ascii_case(line, column) || contains_ignore_ascii_case(line, table)
        })
        .map(|(index, _)| index + 1)
        .unwrap_or(1)
}

fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

#[cfg(test)]
mod tests;
