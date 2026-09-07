use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{collect_postgres_facts, EmbeddedSqlKind};
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
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &opts.embedded,
    )
    .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut findings = Vec::new();
    for file in &facts.embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
                findings.push(finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for shape policy",
                ));
            }
        }
    }
    for file in &facts.statements {
        let rel = relative_slash_path(root, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            findings.push(finding(
                &rel,
                1,
                "SQL could not be analyzed for shape policy",
            ));
            continue;
        }
        if !opts.ban_exists_set_op {
            continue;
        }
        for select in &file.selects {
            for exists in &select.exists_set_operations {
                if exists.correlated {
                    findings.push(finding(
                        &rel,
                        exists.line.max(1),
                        "do not wrap a set operation in a correlated EXISTS",
                    ));
                }
            }
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn finding(file: &str, line: usize, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some("correlated-exists-set-operation".to_string()),
    }
}
