use super::{collect_findings, is_embedded_source, CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    judge_idempotent_subcheck, postgres_sql_paths, record_sql_policy_verdicts, IdempotentCatalog,
    SqlStatementFileFacts,
};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use std::path::{Path, PathBuf};

pub(crate) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let prepared = facts.prepared();
    let sql_paths = postgres_sql_paths(root, files, &opts.schema)?;
    let schema = sql_paths
        .iter()
        .map(|path| prepared.postgres_readable_schema(path))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    let mut embedded = Vec::new();
    let mut statements: Vec<&SqlStatementFileFacts> = Vec::new();
    for path in &sql_paths {
        if let Some(file) = prepared.postgres_readable_statements(path, None)? {
            statements.extend(file);
        }
    }
    for path in files.iter().filter(|path| is_embedded_source(path)) {
        let file = prepared.embedded_sql(path, &opts.embedded)?;
        embedded.push(file);
        if let Some(projected) =
            prepared.postgres_readable_statements(path, Some(&opts.embedded))?
        {
            statements.extend(projected);
        }
    }
    // Legacy collection sorted SQL files and embedded calls by file before
    // deduplication; retain that order for unchanged finding attribution.
    statements.sort_by(|left, right| left.path.cmp(&right.path));
    let triggers = super::with_trigger_paths(statements.iter().copied());
    let catalog = IdempotentCatalog {
        schema: &schema,
        triggers: &triggers,
        replay_safe: &opts.replay_safe,
        check_convergence: opts.check_convergence,
        check_volatility: opts.check_volatility,
        check_arbiter: opts.check_arbiter,
        check_triggers: opts.check_triggers,
        check_generated: opts.check_generated,
        trigger_writes: &opts.trigger_writes,
    };
    if opts.scan_embedded && facts.wants_sql_verdicts() {
        let mut checks = vec!["replay-guard".to_string()];
        for (enabled, name) in [
            (opts.check_arbiter, "arbiter"),
            (opts.check_convergence, "convergence"),
            (opts.check_volatility, "volatility"),
            (opts.check_generated, "generated-arbiter"),
            (opts.check_triggers, "trigger-replay"),
        ] {
            if enabled {
                checks.push(name.to_string());
            }
        }
        for file in &embedded {
            record_sql_policy_verdicts(
                crate::codebase::postgres::SqlPolicyVerdictContext {
                    facts,
                    embedded: file,
                    statements: prepared.postgres_statements(&file.path, Some(&opts.embedded))?,
                },
                RULE_ID,
                &relative_slash_path(root, &file.path),
                &opts.embedded,
                &checks,
                opts.advise_unanalyzable,
                |check, _, statement| judge_idempotent_subcheck(check, statement, &catalog),
            );
        }
    }
    Ok(collect_findings(
        root, opts, embedded, statements, sources, &catalog,
    ))
}
