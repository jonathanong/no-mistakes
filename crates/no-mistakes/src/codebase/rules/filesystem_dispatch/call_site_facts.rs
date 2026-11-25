use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::finite_set_consistency;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactMap, CheckFactPlan,
};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;

pub(super) fn prepare_call_site_facts(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
) -> Result<Option<CheckFactMap>> {
    let call_site_files = finite_set_consistency::try_required_call_site_fact_files(root, config)?;
    let catalog_paths = crate::codebase::postgres::configured_schema_catalog_paths(
        config,
        crate::codebase::postgres::SCHEMA_CATALOG_RULE_IDS,
    )?;
    let mut needs_call_sites = !call_site_files.is_empty();
    let mut plan = CheckFactPlan {
        production_import_paths:
            super::production_dependency_declarations::required_import_fact_files(
                root,
                config,
                &sources.inventory().target_file_paths(),
            )?,
        postgres_schema_catalog_paths: catalog_paths,
        ..Default::default()
    };
    plan.lint_suppression_paths =
        crate::codebase::rules::forbidden_lint_suppressions::required_files(root, config, files)?;
    plan.postgres_explain_write_rollback_paths =
        super::postgres_explain_write_rollback::required_files(root, config, files, sources)?;
    plan.require_call_site_paths(&call_site_files);
    plan.vitest_default_exclude_paths =
        super::vitest_default_excludes::required_files(root, config, files)?;
    plan.test_reliability_paths =
        super::test_reliability::required_files(root, config, files, sources)?;
    plan.playwright_fixture_support = plan
        .test_reliability_paths
        .values()
        .any(|demand| demand.includes_playwright_tests());
    let lock_inventory_helpers = crate::codebase::postgres::helper_call_site_files(
        root,
        config,
        &sources.inventory().target_file_paths(),
    )?;
    plan.require_call_site_paths(&lock_inventory_helpers);
    needs_call_sites |= !lock_inventory_helpers.is_empty();
    super::required_companion_imports::configure_fact_plan(
        root, config, files, sources, &mut plan,
    )?;
    if !plan.production_import_paths.is_empty() {
        plan.graph.imports = true;
    }
    crate::codebase::postgres::configure_prepared_postgres_plan(root, config, &mut plan)?;
    crate::codebase::postgres::assign_prepared_embedded_sql_options(config, &mut plan)?;
    plan.embedded_sql = !plan.embedded_sql_options.is_empty();
    if plan.lint_suppression_paths.is_empty()
        && plan.vitest_default_exclude_paths.is_empty()
        && plan.test_reliability_paths.is_empty()
        && !needs_call_sites
        && plan.companion_import_plans.is_empty()
        && plan.production_import_paths.is_empty()
        && plan.postgres_schema_catalog_paths.is_empty()
        && plan.postgres_explain_write_rollback_paths.is_empty()
        && !plan.postgres_schema
        && !plan.postgres_dml
        && !plan.embedded_sql
    {
        return Ok(None);
    }
    let mut files = call_site_files;
    files.extend(lock_inventory_helpers);
    files.extend(plan.lint_suppression_paths.iter().cloned());
    files.extend(plan.vitest_default_exclude_paths.iter().cloned());
    files.extend(plan.test_reliability_paths.keys().cloned());
    files.extend(plan.production_import_paths.iter().cloned());
    files.extend(plan.postgres_explain_write_rollback_paths.iter().cloned());
    if plan.postgres_schema || plan.postgres_dml {
        files.extend(crate::codebase::postgres::postgres_sql_paths(
            root,
            &sources.inventory().target_file_paths(),
            &crate::codebase::postgres::PostgresSchemaOptions {
                sql_include: plan.postgres_sql_include.clone(),
            },
        )?);
    }
    files.extend(crate::codebase::postgres::prepared::write_sql_paths(
        root,
        &sources.inventory().target_file_paths(),
        &plan,
    ));
    if plan.embedded_sql {
        files.extend(
            sources
                .inventory()
                .target_file_paths()
                .into_iter()
                .filter(|path| crate::codebase::dependencies::extract::is_indexable(path)),
        );
    }
    files.sort();
    files.dedup();
    Ok(Some(
        collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            files,
            Vec::new(),
            plan,
            None,
            Arc::clone(sources),
        ),
    ))
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
