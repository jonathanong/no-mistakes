use crate::codebase::check_facts::{
    sql_source_prerequisites::{SqlProjection, SqlSourceKey},
    CheckFactMap,
};
use crate::codebase::postgres::{EmbeddedSqlOptions, TrustedSqlTag};
use crate::config::v2::{Effort, NoMistakesConfig};
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    import_specifier: Option<String>,
    executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    trusted_sql_tags: Vec<TrustedSqlTag>,
    sql_literal_helpers: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    test_globs: Vec<String>,
    helper_roots: Vec<String>,
}

pub(super) fn files(
    root: &Path,
    config: &NoMistakesConfig,
    consumer: &str,
    files: &[PathBuf],
    facts: Option<&CheckFactMap>,
) -> Result<Vec<PathBuf>> {
    let mut demanded = BTreeSet::new();
    for (ordinal, rule) in config.rule_applications(consumer).into_iter().enumerate() {
        if consumer == "postgres-sql-statement-policy" {
            let scoped = NoMistakesConfig {
                rules: vec![rule.clone()],
                ..config.clone()
            };
            if crate::codebase::postgres::configured_embedded_sql_options(&scoped, &[consumer])?
                .is_empty()
            {
                continue;
            }
        }
        let mut options: Options = rule.try_rule_options()?;
        if consumer == "postgres-no-test-state-mutation" {
            options.include = options
                .test_globs
                .into_iter()
                .chain(options.helper_roots)
                .collect();
        }
        let paths = crate::codebase::rules::RuleGlobFilter::new(
            consumer,
            &options.include,
            &options.exclude,
        )?;
        let profile = EmbeddedSqlOptions::configured(
            options.import_specifier.as_deref().unwrap_or_default(),
            &options.executor_names,
        )
        .with_scoped_executors(
            &options.executor_factory_names,
            &options.executor_type_names,
        )
        .with_trusted_sql_tags(&options.trusted_sql_tags)
        .with_sql_literal_helpers(if consumer == "postgres-require-query-annotation" {
            &options.sql_literal_helpers
        } else {
            &[]
        });
        let effort = config.effective_rule_analysis(rule)?.effort;
        let low_syntax = effort == Effort::Low
            && [
                "postgres-no-unbounded-sql-identifier",
                "postgres-no-mutable-sql-registry",
                "postgres-no-offset",
                "postgres-explicit-columns",
                "postgres-required-predicates",
                "postgres-no-transformed-uuid-predicates",
                "postgres-no-test-state-mutation",
            ]
            .contains(&consumer);
        let mut keys = Vec::new();
        if low_syntax {
            let index = rule
                .resolved_application_index
                .map(|index| index.saturating_sub(1))
                .unwrap_or(ordinal);
            keys.push(SqlSourceKey::application(
                SqlProjection::LowSyntax,
                &profile,
                consumer,
                index,
            ));
        } else {
            keys.push(SqlSourceKey::profile(SqlProjection::Embedded, &profile));
        }
        if consumer == "postgres-require-query-annotation" {
            keys.push(SqlSourceKey::profile(
                if effort == Effort::Low {
                    SqlProjection::AnnotationHeads
                } else {
                    SqlProjection::AnnotationMax
                },
                &profile,
            ));
        }
        for path in crate::codebase::rules::selected_rule_files(root, config, rule, files)? {
            if !crate::codebase::dependencies::extract::is_indexable(&path)
                || !paths.includes(&crate::codebase::ts_source::relative_slash_path(
                    root, &path,
                ))
            {
                continue;
            }
            if let Some(file) = facts.and_then(|facts| facts.ts.get(&path)) {
                for key in &keys {
                    demanded.extend(file.sql_source_prerequisites.failures(key).cloned());
                }
            }
            demanded.insert(path);
        }
    }
    Ok(demanded.into_iter().collect())
}

#[cfg(test)]
mod tests;
