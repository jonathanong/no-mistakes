use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{
    collect_postgres_facts, judge_file_with_prepared_triggers, EmbeddedSqlFileFacts,
    IdempotentCatalog, InstalledTriggers, SqlStatementFileFacts, SqlTriggerFact,
};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Context;
use std::path::{Path, PathBuf};
mod prepared;
pub(super) use prepared::scan as prepared;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_dml: true,
            postgres_schema: true,
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &opts.embedded,
    )
    .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let triggers = if opts.check_triggers {
        with_trigger_paths(facts.statements.iter())
    } else {
        Vec::new()
    };
    let catalog = IdempotentCatalog {
        schema: &facts.schema,
        triggers: &triggers,
        replay_safe: &opts.replay_safe,
        check_convergence: opts.check_convergence,
        check_volatility: opts.check_volatility,
        check_arbiter: opts.check_arbiter,
        check_triggers: opts.check_triggers,
        check_generated: opts.check_generated,
        trigger_writes: &opts.trigger_writes,
    };
    let installed = catalog
        .check_triggers
        .then(|| InstalledTriggers::new(catalog.triggers));
    Ok(collect_findings(
        root,
        opts,
        facts.embedded.iter(),
        facts.statements.iter(),
        sources,
        &catalog,
        installed.as_ref(),
    ))
}

fn collect_findings<'a>(
    root: &Path,
    opts: &CompiledOptions,
    embedded: impl IntoIterator<Item = &'a EmbeddedSqlFileFacts>,
    statements: impl IntoIterator<Item = &'a SqlStatementFileFacts>,
    sources: &crate::codebase::ts_source::SourceStore,
    catalog: &IdempotentCatalog<'_>,
    installed: Option<&InstalledTriggers<'_>>,
) -> crate::codebase::rules::PostgresFindings {
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    if opts.scan_embedded {
        for file in embedded {
            let rel = relative_slash_path(root, &file.path);
            for call in &file.calls {
                if opts.fail_unanalyzable && call.is_unanalyzable() {
                    findings.push(finding(
                        &rel,
                        call.line.max(1) as usize,
                        "executed INSERT SQL is not statically recoverable",
                    ));
                }
            }
        }
    }
    for file in statements {
        if !opts.scan_embedded && is_embedded_source(&file.path) {
            continue;
        }
        let rel = relative_slash_path(root, &file.path);
        let judged = if file.parse_failed && !opts.fail_unanalyzable {
            let mut recoverable = file.clone();
            recoverable.parse_failed = false;
            judge_file_with_prepared_triggers(&recoverable, catalog, installed)
        } else {
            judge_file_with_prepared_triggers(file, catalog, installed)
        };
        let source = crate::codebase::rules::read_source(sources, &file.path);
        for (site, line, message) in judged {
            dedup.push(
                &mut findings,
                file,
                site,
                source.as_deref(),
                finding(&rel, line, &message),
            );
        }
    }
    findings.sort();
    findings
}

pub(super) fn with_trigger_paths<'a>(
    files: impl IntoIterator<Item = &'a SqlStatementFileFacts>,
) -> Vec<SqlTriggerFact> {
    files
        .into_iter()
        .flat_map(|file| {
            file.triggers.iter().cloned().map(|mut trigger| {
                if trigger.source_path.as_os_str().is_empty() {
                    trigger.source_path.clone_from(&file.path);
                }
                trigger
            })
        })
        .collect()
}

fn is_embedded_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("js" | "ts" | "mjs" | "mts" | "cjs" | "cts" | "tsx" | "jsx")
    )
}

fn finding(file: &str, line: usize, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some("insert".to_string()),
    }
}
