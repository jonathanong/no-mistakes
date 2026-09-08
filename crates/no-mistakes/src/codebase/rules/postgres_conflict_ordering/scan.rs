mod analysis;
mod substitute;

use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{
    collect_postgres_facts, postgres_sql_paths, EmbeddedSqlKind, PostgresSchemaOptions,
    SchemaCatalog,
};
use crate::codebase::rules::postgres_lock_ordering::directive::has_safe_directive;
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
    let catalog = SchemaCatalog::load(root, &opts.schema_catalog_path, sources)?;
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
        let source = crate::codebase::rules::read_source(sources, &file.path).unwrap_or_default();
        for call in file.calls {
            if has_safe_directive(
                &source,
                call.line,
                call.sql_text.as_deref().unwrap_or_default(),
                &opts.safe_directive,
            ) {
                continue;
            }
            if call.kind == EmbeddedSqlKind::Dynamic {
                if opts.fail_unanalyzable
                    && call
                        .sql_text
                        .as_deref()
                        .is_some_and(analysis::contains_insert)
                {
                    findings.push(analysis::finding(
                        &rel,
                        call.line as usize,
                        "unanalyzable-sql",
                        "keep dynamic INSERT SQL statically parseable so canonical ON CONFLICT ordering can be checked",
                    ));
                }
                continue;
            }
            let Some(sql) = call.sql_text.as_deref() else {
                continue;
            };
            findings.extend(analysis::findings_for_sql(
                &rel,
                call.line as usize,
                sql,
                &catalog,
                opts.fail_unanalyzable,
            ));
        }
    }
    if let Some(sql_sources) = &opts.sql_sources {
        for path in postgres_sql_paths(root, files, sql_sources)? {
            let rel = relative_slash_path(root, &path);
            let source = crate::codebase::rules::read_source(sources, &path).unwrap_or_default();
            if has_safe_directive(&source, 1, &source, &opts.safe_directive) {
                continue;
            }
            findings.extend(analysis::findings_for_sql(
                &rel,
                1,
                &source,
                &catalog,
                opts.fail_unanalyzable,
            ));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}
