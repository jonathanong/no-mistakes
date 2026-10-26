use super::CompiledOptions;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::rules::config_diagnostic::ConfigAnchor;
use crate::codebase::ts_source::relative_slash_path;
use history::{events_before, snapshots, Catalogs};
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &dyn SqlFactContext,
    anchor: &ConfigAnchor,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let prepared = facts.prepared();
    let schema_files = crate::codebase::postgres::postgres_sql_paths(root, files, &opts.schema)
        .map_err(|error| {
            anyhow::anyhow!(
                "{} failed to collect PostgreSQL facts: {error}",
                super::RULE_ID
            )
        })?;
    let schema: Vec<_> = schema_files
        .iter()
        .map(|path| prepared.postgres_readable_schema(path))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    let tables = super::catalog::live_tables(&schema);
    let finals = Catalogs::build(&tables, opts);
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    findings.extend(super::catalog::stale_extra_findings_from_tables(
        &tables,
        &opts.extra_generated_columns,
        anchor,
    ));
    findings.extend(super::catalog::stale_trigger_findings(
        &tables,
        &opts.trigger_maintained_columns,
        anchor,
    ));
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    let has_history = schema.iter().any(|file| !file.table_events.is_empty());
    if !finals.combined.is_empty() || has_history || facts.wants_sql_verdicts() {
        for path in files {
            if !opts.includes_dml(root, path) {
                continue;
            }
            let profile = is_indexable(path).then_some(&opts.embedded);
            if profile.is_none() && path.extension().and_then(|ext| ext.to_str()) != Some("sql") {
                continue;
            }
            // Source/TS parse failures keep the rule's existing per-file tolerance.
            let Some(statements) = prepared.postgres_readable_statements(path, profile)? else {
                continue;
            };
            // Migrations that define schema are matched against the catalog as of each write.
            let history = schema
                .iter()
                .find(|file| file.path == *path && !file.table_events.is_empty());
            let snapshots = history
                .map(|file| snapshots(&schema, file, statements, opts))
                .unwrap_or_default();
            let lookup = |line: usize| match history {
                Some(file) => snapshots.get(&events_before(file, line)).unwrap_or(&finals),
                None => &finals,
            };
            let rel = relative_slash_path(root, path);
            if let Some(profile) = profile.filter(|_| facts.wants_sql_verdicts()) {
                let embedded = prepared.embedded_sql(path, profile)?;
                verdicts::record(
                    crate::codebase::postgres::SqlPolicyVerdictContext {
                        facts,
                        embedded,
                        statements,
                    },
                    &rel,
                    profile,
                    &lookup,
                    !opts.trigger_maintained_columns.is_empty(),
                    opts.advise_unanalyzable,
                );
            }
            let source = crate::codebase::rules::read_source(sources, path);
            extend_writes(
                &mut findings,
                &rel,
                statements,
                &mut dedup,
                source.as_deref(),
                &lookup,
            );
            if let Some(embedded) = profile
                .filter(|_| opts.fail_unanalyzable)
                .and_then(|profile| prepared.embedded_sql(path, profile).ok())
            {
                extend_unanalyzable(&mut findings, &rel, embedded, statements, &lookup);
            }
        }
    }
    findings.sort();
    Ok(findings)
}

mod findings;
use findings::{extend_unanalyzable, extend_writes};
mod history;
#[cfg(test)]
mod tests;
mod verdicts;
