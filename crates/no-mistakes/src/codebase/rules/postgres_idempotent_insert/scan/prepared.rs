use super::{finding, is_embedded_source, CompiledOptions, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{
    judge_file_with_insert_indices, judge_idempotent_subcheck, postgres_sql_paths,
    record_sql_policy_verdicts, IdempotentCatalog, SqlStatementFileFacts,
};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use std::path::{Path, PathBuf};

pub(crate) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &CheckFactMap,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let sql_paths = postgres_sql_paths(root, files, &opts.schema)?;
    let schema = sql_paths
        .iter()
        .map(|path| facts.postgres_readable_schema(path))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    let mut embedded = Vec::new();
    let mut statements: Vec<&SqlStatementFileFacts> = Vec::new();
    for path in &sql_paths {
        if let Some(file) = facts.postgres_readable_statements(path, None)? {
            statements.extend(file);
        }
    }
    for path in files.iter().filter(|path| is_embedded_source(path)) {
        let file = facts.embedded_sql(path, &opts.embedded)?;
        embedded.push(file);
        if let Some(projected) = facts.postgres_readable_statements(path, Some(&opts.embedded))? {
            statements.extend(projected);
        }
    }
    // Legacy collection sorted SQL files and embedded calls by file before
    // deduplication; retain that order for unchanged finding attribution.
    statements.sort_by(|left, right| left.path.cmp(&right.path));
    let triggers = statements
        .iter()
        .flat_map(|file| file.triggers.iter().cloned())
        .collect::<Vec<_>>();
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
                    statements: facts.postgres_statements(&file.path, Some(&opts.embedded))?,
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
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    if opts.scan_embedded {
        for file in &embedded {
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
            judge_file_with_insert_indices(&recoverable, &catalog)
        } else {
            judge_file_with_insert_indices(file, &catalog)
        };
        let source = crate::codebase::rules::read_source(sources, &file.path);
        for (index, line, message) in judged {
            let site = index.map_or(
                crate::codebase::postgres::statements::SqlFactSite::Origin,
                crate::codebase::postgres::statements::SqlFactSite::Insert,
            );
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
    Ok(findings)
}
