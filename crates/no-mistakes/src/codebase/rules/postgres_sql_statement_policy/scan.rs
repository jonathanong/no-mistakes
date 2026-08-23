use super::{sql_rel, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::collect_postgres_facts;
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_schema: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &Default::default(),
    )
    .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut findings = Vec::new();
    for file in &facts.schema {
        let rel = sql_rel(root, &file.path);
        for statement in &file.statement_kinds {
            if !opts.banned.contains(&statement.kind) {
                continue;
            }
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: statement.line.max(1),
                message: format!(
                    "{rel}:{}: SQL files matching this rule must not use {}",
                    statement.line.max(1),
                    statement.kind
                ),
                import: None,
                target: Some(statement.kind.clone()),
            });
        }
    }
    Ok(findings)
}
