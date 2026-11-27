use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;

pub(super) fn files(
    root: &Path,
    config: &NoMistakesConfig,
    consumer: &str,
    candidates: &[PathBuf],
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
) -> Result<Vec<PathBuf>> {
    let selected = match consumer {
        FINITE_SET_CONSISTENCY => {
            crate::codebase::rules::finite_set_consistency::call_site_fact_files_from_inventory(
                root,
                config,
                sources.inventory(),
            )?
        }
        PRODUCTION_DEPENDENCY_DECLARATIONS => {
            crate::codebase::rules::production_dependency_declarations::inputs::required_sources(
                root, config, candidates, sources,
            )?
        }
        FORBIDDEN_LINT_SUPPRESSIONS => {
            crate::codebase::rules::forbidden_lint_suppressions::required_files(
                root, config, candidates,
            )?
            .iter()
            .cloned()
            .collect()
        }
        POSTGRES_EXPLAIN_WRITE_ROLLBACK => match facts {
            Some(facts) => candidates
                .iter()
                .filter(|path| facts.postgres().explain_write_rollback.contains_key(*path))
                .cloned()
                .collect(),
            None => {
                postgres_explain_write_rollback::required_files(root, config, candidates, sources)?
                    .iter()
                    .cloned()
                    .collect()
            }
        },
        REQUIRED_COMPANION_IMPORTS => facts
            .and_then(|facts| {
                facts
                    .companion_import_plans
                    .iter()
                    .find(|plan| plan.matches(root, config))
            })
            .map(|plan| plan.import_paths.clone())
            .unwrap_or_default(),
        _ if crate::codebase::postgres::PREPARED_EMBEDDED_SQL_RULE_IDS.contains(&consumer) => {
            super::sql_demand::files(root, config, consumer, candidates, facts)?
        }
        _ => Vec::new(),
    };
    Ok(selected)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
