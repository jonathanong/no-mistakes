use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::postgres::sql_identifier_syntax::plan::SqlIdentifierScan;
use crate::codebase::ts_source::SourceStore;
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
}

impl ExecutorWarningScan {
    pub(crate) fn includes(&self, root: &Path, path: &Path) -> bool {
        self.scan.includes(root, path)
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    import_specifier: String,
    executor_names: Option<Vec<String>>,
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
        let Some(names) = options.executor_names else {
            continue;
        };
        if names.is_empty() || options.import_specifier.is_empty() {
            continue;
        }
        let omitted = ["query", "read", "write"]
            .into_iter()
            .filter(|name| !names.iter().any(|configured| configured == name))
            .map(str::to_string)
            .collect::<Vec<_>>();
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
            import_specifier: options.import_specifier,
            omitted,
        });
    }
    plan.postgres_executor_warning_scans = Arc::new(scans);
    Ok(())
}

#[doc(hidden)]
pub fn executor_config_warnings(
    root: &Path,
    facts: &CheckFactMap,
    sources: &SourceStore,
) -> Vec<String> {
    let mut warnings = Vec::new();
    for scan in facts.postgres_executor_warning_scans.iter() {
        let examples = facts
            .files()
            .par_iter()
            .filter(|path| scan.includes(root, path))
            .filter_map(|path| {
                let file = facts.ts.get(path)?;
                let module = file.ts.module_bindings.as_deref()?;
                let source = sources.read_path(path).ok()?;
                let calls = calls::omitted_calls(module, &file.ts.call_sites, &source, scan);
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
        for (name, (file, line)) in first {
            warnings.push(format!(
                "{}: executorNames replaces the defaults [query, read, write] and omits `{name}`, imported from `{}` and called at {file}:{line}; this call is not selected by this executor list. Use extraExecutorNames instead to extend the defaults, or add `{name}` to executorNames.",
                scan.label, scan.import_specifier
            ));
        }
    }
    warnings.sort();
    warnings.dedup();
    warnings
}
