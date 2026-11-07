use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::postgres::sql_identifier_syntax::plan::SqlIdentifierScan;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

mod calls;

#[derive(Clone)]
#[doc(hidden)]
pub struct ExecutorWarningScan {
    scan: SqlIdentifierScan,
    label: String,
    import_specifier: String,
    omitted: Vec<String>,
    exclusions: Vec<String>,
}

impl ExecutorWarningScan {
    pub(crate) fn includes(&self, root: &Path, path: &Path) -> bool {
        self.scan.includes(root, path)
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    import_specifier: Option<String>,
    executor_names: Option<Vec<String>>,
    exclude_executor_names: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    test_globs: Vec<String>,
    helper_roots: Vec<String>,
}

#[doc(hidden)]
pub fn configure_executor_warnings(
    root: &Path,
    config: &NoMistakesConfig,
    plan: &mut CheckFactPlan,
) -> Result<()> {
    let mut scans = Vec::new();
    for (index, rule) in config.rules.iter().enumerate().filter(|(_, rule)| {
        rule.enabled
            && config.rule_has_effective_target(rule)
            && crate::config::v2::schema::postgres_executors::ALL_RULE_IDS
                .contains(&rule.rule.as_str())
    }) {
        let options: Options = rule.try_rule_options()?;
        let import_specifier = options.import_specifier.unwrap_or_default();
        let exclusions = options.exclude_executor_names;
        let mut omitted = if let Some(names) = options
            .executor_names
            .filter(|names| !names.is_empty() && !import_specifier.is_empty())
        {
            ["query", "read", "write"]
                .into_iter()
                .filter(|name| !names.iter().any(|configured| configured == name))
                .map(str::to_string)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        omitted.extend(exclusions.iter().cloned());
        omitted.sort();
        omitted.dedup();
        if omitted.is_empty() {
            continue;
        }
        let include = if rule.rule == "postgres-no-test-state-mutation" {
            options
                .test_globs
                .into_iter()
                .chain(options.helper_roots)
                .collect()
        } else {
            options.include
        };
        let label = if rule.projects.is_empty() {
            rule.name.as_deref().unwrap_or(&rule.rule).to_string()
        } else {
            format!("{} (projects: {})", rule.rule, rule.projects.join(", "))
        };
        scans.push(ExecutorWarningScan {
            scan: SqlIdentifierScan::new(root, config, rule, index, &include, &options.exclude)?,
            label,
            import_specifier,
            omitted,
            exclusions,
        });
    }
    plan.postgres_executor_warning_scans = Arc::new(scans);
    Ok(())
}

#[doc(hidden)]
pub fn executor_config_warnings(root: &Path, facts: &CheckFactMap) -> Vec<String> {
    let mut warnings = Vec::new();
    for scan in facts.postgres_executor_warning_scans.iter() {
        let examples = facts
            .files()
            .par_iter()
            .filter(|path| scan.includes(root, path))
            .filter_map(|path| {
                let file = facts.ts.get(path)?;
                let module = file.ts.module_bindings.as_deref()?;
                let calls = calls::omitted_calls(module, &file.ts.call_sites, scan);
                let relative = crate::codebase::ts_source::relative_slash_path(root, path);
                Some(
                    calls
                        .into_iter()
                        .map(|(name, line)| (name, (relative.clone(), line)))
                        .collect::<Vec<_>>(),
                )
            })
            .flatten()
            .collect::<Vec<_>>();
        let mut first = BTreeMap::new();
        for (name, site) in examples {
            first
                .entry(name)
                .and_modify(|existing| {
                    if site < *existing {
                        *existing = site.clone();
                    }
                })
                .or_insert(site);
        }
        for name in &scan.exclusions {
            if !["query", "read", "write"].contains(&name.as_str()) || !first.contains_key(name) {
                warnings.push(format!(
                    "{}: excludeExecutorNames entry `{name}` is unused: it is not a default executor [query, read, write], or is never imported and called from the configured module in scoped files. Remove the entry or correct its spelling and scope. Unused exclusions can hide configuration mistakes.",
                    scan.label
                ));
            }
        }
        for (name, (file, line)) in first {
            if scan.exclusions.contains(&name) {
                continue;
            }
            warnings.push(format!(
                "{}: executorNames replaces the defaults [query, read, write] and omits `{name}`, imported from `{}` and called at {file}:{line}; this call is not selected by this executor list. Use extraExecutorNames instead to extend the defaults, add `{name}` to executorNames, or list `{name}` in excludeExecutorNames to acknowledge an intentional exclusion.",
                scan.label, scan.import_specifier
            ));
        }
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

#[cfg(test)]
mod tests;
