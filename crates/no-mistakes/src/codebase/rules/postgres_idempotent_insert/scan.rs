use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{
    collect_postgres_facts, judge_file, EmbeddedSqlKind, IdempotentCatalog,
};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
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
    let triggers: Vec<_> = facts
        .statements
        .iter()
        .flat_map(|file| file.triggers.iter().cloned())
        .collect();
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
    let mut findings = Vec::new();
    if opts.scan_embedded {
        for file in &facts.embedded {
            let rel = relative_slash_path(root, &file.path);
            for call in &file.calls {
                if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
                    findings.push(finding(
                        &rel,
                        call.line.max(1) as usize,
                        "executed INSERT SQL is not statically recoverable",
                    ));
                }
            }
        }
    }
    for file in &facts.statements {
        if !opts.scan_embedded && !is_sql_path(&file.path) {
            continue;
        }
        let rel = relative_slash_path(root, &file.path);
        for (line, message) in judge_file(file, &catalog) {
            findings.push(finding(&rel, line, &message));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn is_sql_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("sql"))
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
