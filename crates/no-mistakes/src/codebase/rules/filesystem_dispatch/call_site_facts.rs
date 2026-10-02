use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use super::finite_set_consistency;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactMap, CheckFactPlan,
};
use crate::codebase::ts_source::facts::TsFactPlan;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;

pub(super) fn prepare_call_site_facts(
    root: &Path,
    config: &NoMistakesConfig,
    sources: &Arc<SourceStore>,
) -> Result<Option<CheckFactMap>> {
    let call_site_files = finite_set_consistency::try_required_call_site_fact_files(root, config)?;
    let catalog_paths = crate::codebase::postgres::configured_schema_catalog_paths(
        config,
        crate::codebase::postgres::SCHEMA_CATALOG_RULE_IDS,
    )?;
    let needs_call_sites = !call_site_files.is_empty();
    let mut plan = CheckFactPlan {
        graph: TsFactPlan {
            call_sites: needs_call_sites,
            ..Default::default()
        },
        postgres_schema_catalog_paths: catalog_paths,
        ..Default::default()
    };
    crate::codebase::postgres::configure_prepared_postgres_plan(config, &mut plan)?;
    plan.embedded_sql_options = crate::codebase::postgres::configured_embedded_sql_options(
        config,
        crate::codebase::postgres::PREPARED_EMBEDDED_SQL_RULE_IDS,
    )?;
    plan.embedded_sql = !plan.embedded_sql_options.is_empty();
    if !needs_call_sites
        && plan.postgres_schema_catalog_paths.is_empty()
        && !plan.postgres_schema
        && !plan.postgres_dml
        && !plan.embedded_sql
    {
        return Ok(None);
    }
    let mut files = call_site_files;
    if plan.postgres_schema || plan.postgres_dml {
        files.extend(crate::codebase::postgres::postgres_sql_paths(
            root,
            &sources.inventory().target_file_paths(),
            &crate::codebase::postgres::PostgresSchemaOptions {
                sql_include: plan.postgres_sql_include.clone(),
            },
        )?);
    }
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

#[cfg(test)]
mod tests;
