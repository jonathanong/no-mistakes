use super::RuleFinding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod included_rule;
pub(crate) use included_rule::run_included_postgres_rule;

/// Prepare the standalone rule's schema catalog using the request's source store.
pub(crate) fn prepare_schema_rule_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &Arc<SourceStore>,
    rule_id: &str,
) -> Result<CheckFactMap> {
    let catalog_paths =
        crate::codebase::postgres::configured_schema_catalog_paths(config, &[rule_id])?;
    Ok(crate::codebase::postgres::prepare_embedded_sql_facts(
        root,
        all_files,
        Arc::clone(sources),
        Vec::new(),
        catalog_paths,
    ))
}

/// Prepare SQL facts for one standalone rule with its own declared fact demand.
pub(crate) fn prepare_sql_rule_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &Arc<SourceStore>,
    rule_id: &str,
) -> Result<CheckFactMap> {
    crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        all_files,
        Arc::clone(sources),
        config,
        &[rule_id],
    )
}

pub(crate) fn check_prepared_schema_rule(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &Arc<SourceStore>,
    rule_id: &str,
    check: impl FnOnce(
        &Path,
        &NoMistakesConfig,
        &[PathBuf],
        &SourceStore,
        &CheckFactMap,
    ) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    let facts = prepare_schema_rule_facts(root, config, all_files, sources, rule_id)?;
    check(root, config, all_files, sources, &facts)
}

pub(crate) fn check_prepared_sql_rule(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &Arc<SourceStore>,
    rule_id: &str,
    check: impl FnOnce(
        &Path,
        &NoMistakesConfig,
        &[PathBuf],
        &SourceStore,
        &CheckFactMap,
    ) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    let prepared_config = if rule_id == "postgres-no-offset" {
        super::analysis_policy::prepare_requested_rule_config(
            root, rule_id, config, all_files, sources,
        )?
    } else {
        None
    };
    let config = prepared_config.as_ref().unwrap_or(config);
    let facts = prepare_sql_rule_facts(root, config, all_files, sources, rule_id)?;
    let mut findings = check(root, config, all_files, sources, &facts)?;
    findings.extend(super::postgres_undeclared_executors::check(
        root, config, rule_id, all_files, &facts,
    )?);
    super::sort_findings(&mut findings);
    Ok(findings)
}

/// Keep option parsing ahead of this lookup so configuration errors retain
/// their historical precedence over path filtering and catalog failures.
pub(crate) fn selected_schema_catalog<'a>(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    facts: &'a CheckFactMap,
    path: &str,
) -> Result<Option<&'a crate::codebase::postgres::SchemaCatalog>> {
    let filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
    if !filter.is_match(Path::new(path)) {
        return Ok(None);
    }
    facts.postgres_schema_catalog(path).map(Some)
}

pub(crate) fn run_rule_applications(
    config: &NoMistakesConfig,
    rule_id: &str,
    mut check: impl FnMut(&RuleDef) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(rule_id) {
        findings.extend(check(rule)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

/// Evaluate a file-scoped SQL rule against one prepared inventory and fact map.
pub(crate) fn run_prepared_file_rule<C>(
    context: super::PreparedRuleContext<'_>,
    rule_id: &str,
    compile: impl Fn(&RuleDef) -> Result<C>,
    scan: impl Fn(&Path, &C, &[PathBuf], &SourceStore, &CheckFactMap) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    let super::PreparedRuleContext {
        root,
        config,
        all_files,
        sources,
        facts,
    } = context;
    run_rule_applications(config, rule_id, |rule| {
        let compiled = compile(rule)?;
        let files = super::selected_rule_files(root, config, rule, all_files)?;
        scan(root, &compiled, &files, sources, facts)
    })
}

pub(crate) fn check_with_source_store(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    check: impl FnOnce(
        &Path,
        &NoMistakesConfig,
        &[PathBuf],
        &Arc<SourceStore>,
    ) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check(root, config, all_files, &sources)
}

pub(crate) fn check_with_borrowed_source_store(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    check: impl FnOnce(&Path, &NoMistakesConfig, &[PathBuf], &SourceStore) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    check_with_source_store(root, config, all_files, |root, config, files, sources| {
        check(root, config, files, sources)
    })
}

pub(crate) fn schema_options(
    sql_include: &[String],
    locked: bool,
) -> crate::codebase::postgres::PostgresSchemaOptions {
    crate::codebase::postgres::PostgresSchemaOptions {
        sql_include: crate::codebase::postgres::effective_sql_include(sql_include, locked),
    }
}
