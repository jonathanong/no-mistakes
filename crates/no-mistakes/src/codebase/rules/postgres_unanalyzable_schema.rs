//! Diagnostics for migration chunks dropped by lenient PostgreSQL parsing.
use super::{analysis_policy::AnalysisDiagnostics, selected_rule_files, RuleFinding};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{postgres_sql_paths, PostgresSchemaOptions};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Rules that consume migration facts from the lenient SQL parser. Catalog
/// snapshot rules load JSONC and cannot contain a synthetic SQL marker.
pub const RULE_IDS: &[&str] = &[
    "postgres-constraint-validate",
    "postgres-fk-index",
    "postgres-generated-column-predicates",
    "postgres-idempotent-insert",
    "postgres-identifier-length",
    "postgres-no-add-column",
    "postgres-no-generated-column-writes",
    "postgres-redundant-index",
    "postgres-require-fk-on-delete",
    "postgres-require-named-constraints",
    "postgres-sql-statement-policy",
];

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ScopeOptions {
    sql_include: Vec<String>,
    sql_include_locked: bool,
    include: Vec<String>,
    exclude: Vec<String>,
}

pub fn diagnostics(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: &CheckFactMap,
) -> Result<AnalysisDiagnostics> {
    diagnostics_for_ids(root, config, files, facts, RULE_IDS)
}

pub(crate) fn diagnostics_for_rule(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: &CheckFactMap,
    rule_id: &str,
) -> Result<AnalysisDiagnostics> {
    if !RULE_IDS.contains(&rule_id) {
        return Ok(AnalysisDiagnostics::default());
    }
    diagnostics_for_ids(root, config, files, facts, &[rule_id])
}

fn diagnostics_for_ids(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: &CheckFactMap,
    rule_ids: &[&str],
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule_id in rule_ids {
        for rule in config.rule_applications(rule_id) {
            let policy = config.effective_rule_analysis(rule)?.unanalyzable;
            if policy == Unanalyzable::Ignore {
                continue;
            }
            let options: ScopeOptions = rule.try_rule_options()?;
            let mut selected = selected_rule_files(root, config, rule, files)?;
            if *rule_id == "postgres-idempotent-insert" {
                let include = super::path_filter::GlobMatcher::new(
                    &options.include,
                    "postgres-idempotent-insert include",
                )?;
                let exclude = super::path_filter::GlobMatcher::new(
                    &options.exclude,
                    "postgres-idempotent-insert exclude",
                )?;
                selected.retain(|path| {
                    let rel = relative_slash_path(root, path);
                    (include.is_empty() || include.is_match(&rel))
                        && (exclude.is_empty() || !exclude.is_match(&rel))
                });
            }
            let schema = super::schema_options(&options.sql_include, options.sql_include_locked);
            append_rule(
                root,
                rule_id,
                policy,
                &selected,
                &schema,
                facts,
                &mut output,
            )?;
        }
    }
    output.findings.sort();
    output.advisories.sort();
    Ok(output)
}

fn append_rule(
    root: &Path,
    rule_id: &str,
    policy: Unanalyzable,
    files: &[PathBuf],
    schema: &PostgresSchemaOptions,
    facts: &CheckFactMap,
    output: &mut AnalysisDiagnostics,
) -> Result<()> {
    for path in postgres_sql_paths(root, files, schema)? {
        let file = facts.postgres_schema_file(&path)?;
        let rel = relative_slash_path(root, &path);
        for statement in &file.unanalyzable_statements {
            let line = statement.line.max(1);
            output.unanalyzable(
                policy,
                RuleFinding {
                    rule: rule_id.to_owned(),
                    file: rel.clone(),
                    line,
                    message: format!(
                        "{rel}:{line}: SQL statement `{}` could not be analyzed for {rule_id}; rewrite it as supported static PostgreSQL SQL",
                        statement.head
                    ),
                    import: None,
                    target: Some(statement.head.clone()),
                },
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
