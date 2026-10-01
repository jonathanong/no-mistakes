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
    if !needs_call_sites && catalog_paths.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            call_site_files,
            Vec::new(),
            CheckFactPlan {
                graph: TsFactPlan {
                    call_sites: needs_call_sites,
                    ..Default::default()
                },
                postgres_schema_catalog_paths: catalog_paths,
                ..Default::default()
            },
            None,
            Arc::clone(sources),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::v2::schema::{RuleDef, RuleScope};
    use std::path::PathBuf;

    #[test]
    fn schema_catalog_rules_prepare_a_catalog_without_call_sites() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres/column-requires-trigger");
        let mut config = NoMistakesConfig::default();
        config.rules.push(RuleDef {
            rule: "postgres-column-requires-trigger".to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("schemaCatalogPath: schema.json\n").unwrap(),
            ..RuleDef::default()
        });
        let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
            &root,
            &[root.join("schema.json")],
        );
        let sources = snapshot.source_store_for(&root);
        let facts = prepare_call_site_facts(&root, &config, &sources)
            .unwrap()
            .expect("catalog demand");
        assert!(facts.postgres_schema_catalog("schema.json").is_ok());
    }
}
