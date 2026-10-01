mod catalog;
mod check;
mod columns;

use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{collect_postgres_facts, EmbeddedSqlKind, SchemaCatalog};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Context;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> anyhow::Result<Vec<RuleFinding>> {
    if opts.relations.is_empty() && !opts.partition_keys {
        return Ok(Vec::new());
    }
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_dml: true,
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &opts.embedded,
    )
    .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let catalog = load_catalog(root, opts, sources, prepared)?;
    let mut findings = Vec::new();
    if let Some(catalog) = &catalog {
        if opts.partition_keys {
            if let Some(path) = &opts.schema_catalog_path {
                findings.extend(catalog::catalog_findings(path, catalog, opts));
            }
        }
    }
    for file in &facts.embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
                findings.push(check::sql_finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for required predicates",
                    Some("unanalyzable"),
                    None,
                ));
            }
        }
    }
    for file in &facts.statements {
        let rel = relative_slash_path(root, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            findings.push(check::sql_finding(
                &rel,
                1,
                "SQL could not be analyzed for required predicates",
                Some("unanalyzable"),
                None,
            ));
            continue;
        }
        findings.extend(check::statement_findings(
            &rel,
            file,
            opts,
            catalog.as_deref(),
        ));
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn load_catalog(
    root: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> anyhow::Result<Option<Arc<SchemaCatalog>>> {
    let Some(path) = &opts.schema_catalog_path else {
        return Ok(None);
    };
    if let Some(prepared) = prepared {
        let key = crate::codebase::postgres::normalize_schema_catalog_path(path)?
            .to_string_lossy()
            .into_owned();
        if let Some(entry) = prepared.postgres_schema_catalogs.get(&key) {
            return match entry {
                Ok(catalog) => Ok(Some(Arc::clone(catalog))),
                Err(error) => Err(anyhow::anyhow!(error.to_string())),
            };
        }
    }
    Ok(Some(Arc::new(SchemaCatalog::load(root, path, sources)?)))
}
