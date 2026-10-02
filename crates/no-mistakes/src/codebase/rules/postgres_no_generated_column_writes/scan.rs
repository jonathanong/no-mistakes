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
    let tables = super::catalog::live_tables(&facts.schema);
    let catalog = super::catalog::catalog_from_tables(&tables, &opts.extra_generated_columns);
    let trigger =
        super::catalog::trigger_catalog_from_tables(&tables, &opts.trigger_maintained_columns);
    let mut combined = trigger.clone();
    combined.extend_from(&catalog);
    let mut findings =
        super::catalog::stale_extra_findings_from_tables(&tables, &opts.extra_generated_columns);
    findings.extend(super::catalog::stale_trigger_findings(
        &tables,
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
                &combined,
                &catalog,
            ));
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("sql") {
            findings.extend(scan_sql_file(path, &rel, sources, &combined, &catalog));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_embedded(
    file: &str,
    facts: Option<&EmbeddedSqlFileFacts>,
    catalog: &GeneratedTableColumns,
    generated: &GeneratedTableColumns,
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
        extend_writes(&mut findings, file, sql, catalog, generated, |_, _| line);
    }
    findings
}

fn scan_sql_file(
    path: &Path,
    file: &str,
    sources: &SourceStore,
    catalog: &GeneratedTableColumns,
    generated: &GeneratedTableColumns,
) -> Vec<RuleFinding> {
    let Some(source) = crate::codebase::rules::read_source(sources, path) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for (line, statement) in crate::codebase::postgres::top_level_statements(&source) {
        extend_writes(
            &mut findings,
            file,
            &statement,
            catalog,
            generated,
            |_, _| line,
        );
    }
    findings
}

fn extend_writes(
    findings: &mut Vec<RuleFinding>,
    file: &str,
    sql: &str,
    catalog: &GeneratedTableColumns,
    generated: &GeneratedTableColumns,
    line_for: impl Fn(&str, &str) -> usize,
) {
    // Both column kinds share this parse; the generated catalog only classifies hits.
    for write in find_generated_column_writes(sql, catalog) {
        let render = if generated
            .get_exact(&write.table)
            .or_else(|| generated.get(&write.table))
            .is_some_and(|table| table.generated.contains(&write.column.to_ascii_lowercase()))
        {
            finding
        } else {
            trigger_finding
        };
        findings.push(render(
            file,
            line_for(&write.table, &write.column),
            &write.table,
            &write.column,
        ));
    }
}

#[cfg(test)]
mod tests;
