use super::RuleFinding;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::ts_source::SourceStore;
use anyhow::Result;
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

pub(crate) fn run_sql_rule_with_glob_filters<O: DeserializeOwned + Default, C>(
    context: super::PreparedRuleContext<'_>,
    rule_id: &str,
    compile: impl Fn(&O) -> Result<C> + Sync,
    patterns: impl for<'a> Fn(&'a O) -> (&'a [String], &'a [String]) + Sync,
    scan: impl Fn(
            &Path,
            &C,
            &[PathBuf],
            &SourceStore,
            &dyn SqlFactContext,
        ) -> Result<super::PostgresFindings>
        + Sync,
) -> Result<Vec<RuleFinding>> {
    let super::PreparedRuleContext {
        root,
        config,
        all_files,
        sources,
        facts,
    } = context;
    let all: Result<Vec<super::PostgresFindings>> = config
        .rule_applications(rule_id)
        .into_par_iter()
        .enumerate()
        .map(|(index, rule)| {
            let opts: O = rule.try_rule_options()?;
            let compiled = compile(&opts)?;
            let files = super::selected_rule_files(root, config, rule, all_files)?;
            let (include, exclude) = patterns(&opts);
            let include =
                super::path_filter::GlobMatcher::new(include, &format!("{rule_id} include"))?;
            let exclude =
                super::path_filter::GlobMatcher::new(exclude, &format!("{rule_id} exclude"))?;
            let files: Vec<PathBuf> = files
                .into_iter()
                .filter(|path| {
                    let rel = crate::codebase::ts_source::relative_slash_path(root, path);
                    (include.is_empty() || include.is_match(&rel))
                        && (exclude.is_empty() || !exclude.is_match(&rel))
                })
                .collect();
            let view = if matches!(
                rule_id,
                "postgres-explicit-columns" | "postgres-required-predicates"
            ) && config.effective_rule_analysis(rule)?.effort
                == crate::config::v2::Effort::Low
            {
                Some(
                    facts
                        .for_sql_application_with_advice(index, rule.name.as_deref())
                        .with_low_sql(rule_id),
                )
            } else {
                facts.for_sql_application(index, rule.name.as_deref())
            };
            scan(
                root,
                &compiled,
                &files,
                sources,
                view.as_ref()
                    .map_or(facts as &dyn SqlFactContext, |view| view),
            )
        })
        .collect();
    let findings: super::PostgresFindings = all?.into_iter().flatten().collect();
    Ok(findings.finish())
}
