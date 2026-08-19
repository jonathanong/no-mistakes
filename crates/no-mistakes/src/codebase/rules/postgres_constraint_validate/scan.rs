use super::{sql_rel, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{collect_postgres_facts, SqlNamedConstraint};
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::collections::BTreeMap;
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
    .with_context(|| format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut not_valid = BTreeMap::<String, (String, usize)>::new();
    let mut validated = BTreeMap::<String, (String, usize)>::new();
    for file in &facts.schema {
        let rel = sql_rel(root, &file.path);
        collect_named(&file.not_valid_constraints, &rel, &mut not_valid);
        collect_named(&file.validated_constraints, &rel, &mut validated);
    }
    let mut findings = Vec::new();
    for (key, (file, line)) in &not_valid {
        if validated.contains_key(key) {
            continue;
        }
        findings.push(finding(
            file,
            *line,
            format!(
                "ALTER TABLE ADD CONSTRAINT {name} NOT VALID must have a matching VALIDATE CONSTRAINT",
                name = constraint_name(key)
            ),
            key,
        ));
    }
    for (key, (file, line)) in &validated {
        if not_valid.contains_key(key) {
            continue;
        }
        findings.push(finding(
            file,
            *line,
            format!(
                "VALIDATE CONSTRAINT {name} has no matching named NOT VALID add",
                name = constraint_name(key)
            ),
            key,
        ));
    }
    Ok(findings)
}

fn collect_named(
    constraints: &[SqlNamedConstraint],
    rel: &str,
    into: &mut BTreeMap<String, (String, usize)>,
) {
    for constraint in constraints {
        let key = format!("{}.{}", constraint.table_name, constraint.name);
        into.entry(key)
            .or_insert_with(|| (rel.to_string(), constraint.line.max(1)));
    }
}

fn constraint_name(key: &str) -> &str {
    key.rsplit_once('.').map(|(_, name)| name).unwrap_or(key)
}

fn finding(file: &str, line: usize, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
