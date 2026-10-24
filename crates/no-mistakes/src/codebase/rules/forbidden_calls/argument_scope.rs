//! Check-only argument demand; ordinary graph facts retain their global guarantee.
use super::super::path_filter::RulePathFilter;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use std::path::Path;
use std::sync::Arc;

#[doc(hidden)]
pub struct ArgumentFactScope {
    // None is a conservative fallback: malformed filters keep their historical
    // validation boundary rather than introducing an earlier planning error.
    filters: Option<Vec<Arc<RulePathFilter>>>,
}

pub(super) fn requires_arguments(application: &RuleDef) -> bool {
    application
        .options
        .get("targets")
        .and_then(serde_yaml::Value::as_sequence)
        .is_some_and(|targets| {
            targets.iter().any(|target| {
                target
                    .get("whenArguments")
                    .is_some_and(|value| !value.is_null())
            })
        })
}

impl ArgumentFactScope {
    pub(crate) fn applies(&self, path: &Path) -> bool {
        self.filters
            .as_ref()
            .is_none_or(|filters| filters.iter().any(|filter| filter.is_match(path)))
    }

    pub(crate) fn merge(left: Option<&Arc<Self>>, right: Option<&Arc<Self>>) -> Option<Arc<Self>> {
        match (left, right) {
            (None, None) => None,
            (Some(scope), None) | (None, Some(scope)) => Some(Arc::clone(scope)),
            (Some(left), Some(right)) if Arc::ptr_eq(left, right) => Some(Arc::clone(left)),
            (Some(left), Some(right)) => Some(Arc::new(Self {
                filters: left
                    .filters
                    .as_ref()
                    .zip(right.filters.as_ref())
                    .map(|(left, right)| left.iter().chain(right).map(Arc::clone).collect()),
            })),
        }
    }
}

fn for_config(root: &Path, config: &NoMistakesConfig) -> Option<Arc<ArgumentFactScope>> {
    let applications = config.rule_applications(super::RULE_ID);
    if !applications
        .iter()
        .any(|application| requires_arguments(application))
    {
        return None;
    }
    let filters = applications
        .into_iter()
        .filter(|application| requires_arguments(application))
        .map(|application| RulePathFilter::new(root, config, application).map(Arc::new))
        .collect::<anyhow::Result<Vec<_>>>()
        .ok();
    Some(Arc::new(ArgumentFactScope { filters }))
}

/// Declare only the argument facts the configured policy can consume. Broad
/// `graph.call_arguments` demand from another consumer remains authoritative.
#[doc(hidden)]
pub fn configure_argument_facts(root: &Path, config: &NoMistakesConfig, plan: &mut CheckFactPlan) {
    let requested = for_config(root, config);
    plan.forbidden_call_arguments =
        ArgumentFactScope::merge(plan.forbidden_call_arguments.as_ref(), requested.as_ref());
}

pub(crate) fn argument_facts_covered(
    root: &Path,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
) -> bool {
    if facts.graph_plan().call_arguments {
        return true;
    }
    let Some(required) = for_config(root, config) else {
        return true;
    };
    facts
        .graph_file_universe()
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .filter(|path| required.applies(path))
        .all(|path| {
            facts.ts.get(path).is_some_and(|file| {
                let ts = &file.ts;
                // These inputs deliberately produce Unknown argument evidence, even
                // with unrestricted demand. Valid inputs must contain the projection.
                ts.operational_error.is_some()
                    || ts.parse_error.is_some()
                    || ts.fatal_parse_error
                    || ts.module_bindings.as_ref().is_some_and(|module| {
                        !module.diagnostics.is_empty() || module.call_arguments.is_some()
                    })
            })
        })
}

#[cfg(test)]
mod tests;
