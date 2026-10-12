use super::{PreparedCompanionImportPlan, RuleFinding, RULE_ID};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::SourceStore;
use crate::fx::{FxHashMap, FxHashSet};
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub(super) fn findings(
    prepared: &PreparedCompanionImportPlan,
    facts: &dyn TsFactLookup,
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let imports: FxHashMap<&Path, FxHashSet<&str>> = prepared
        .import_paths
        .par_iter()
        .map(|file| {
            let imports = facts.get_production_imports(file).ok_or_else(|| {
                anyhow::anyhow!(
                    "missing prepared companion import facts for {}",
                    file.display()
                )
            })?;
            if let Some(observer) = sources.observer() {
                observer.increment("companion_imports.literal_projections", 1);
            }
            Ok((
                file.as_path(),
                imports
                    .iter()
                    .filter(|import| !import.computed)
                    .map(|import| import.specifier.as_str())
                    .collect(),
            ))
        })
        .collect::<Result<_>>()?;
    let mut selections: FxHashMap<usize, FxHashSet<&str>> = FxHashMap::default();
    for check in &prepared.checks {
        selections
            .entry(std::sync::Arc::as_ptr(&check.companions) as usize)
            .or_insert_with(|| {
                if let Some(observer) = sources.observer() {
                    observer.increment("companion_imports.literal_unions", 1);
                }
                check
                    .companions
                    .iter()
                    .flat_map(|file| {
                        imports
                            .get(file.as_path())
                            .into_iter()
                            .flat_map(|imports| imports.iter().copied())
                    })
                    .collect()
            });
    }
    let mut findings = prepared
        .checks
        .par_iter()
        .filter_map(|check| {
            if selections[&(std::sync::Arc::as_ptr(&check.companions) as usize)]
                .contains(check.expected.as_str())
            {
                return None;
            }
            let message = if check.companions.is_empty() {
                "no companion file found importing"
            } else {
                "companion files do not import"
            };
            Some(RuleFinding {
                rule: RULE_ID.to_string(),
                file: check.rel.clone(),
                line: 1,
                message: format!("{}: {} {}", check.rel, message, check.expected),
                import: None,
                target: Some(check.expected.clone()),
            })
        })
        .collect::<Vec<_>>();
    super::super::sort_findings(&mut findings);
    Ok(findings)
}
