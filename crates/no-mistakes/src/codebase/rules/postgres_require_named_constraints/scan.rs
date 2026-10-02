use super::{sql_rel, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::collect_prepared_schema_facts;
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    _sources: &SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<Vec<RuleFinding>> {
    let facts = collect_prepared_schema_facts(root, files, &opts.schema, facts)
        .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut findings = Vec::new();
    for file in &facts.schema {
        let rel = sql_rel(root, &file.path);
        for constraint in &file.unnamed_constraints {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: constraint.line.max(1),
                message: format!(
                    "{rel}:{}: ALTER TABLE ADD {} constraints must have an explicit name for validation pairing",
                    constraint.line.max(1),
                    constraint.kind
                ),
                import: None,
                target: Some(format!("{}.{}", constraint.table_name, constraint.kind)),
            });
        }
    }
    Ok(findings)
}
