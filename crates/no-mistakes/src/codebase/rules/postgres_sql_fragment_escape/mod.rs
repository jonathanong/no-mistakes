//! Cross-file writes, rebinds, and namespace escapes of exported SQL fragments.
mod alias;
mod dynamic;
mod fragments;
mod modules;
mod names;
mod resolve;
mod scan;
mod source_text;
mod tags;
mod text;
mod visible;
mod writes;

use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

pub const RULE_ID: &str = "postgres-sql-fragment-escape";

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    trusted_sql_tags: Vec<crate::codebase::postgres::embedded::TrustedSqlTag>,
}

pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(Vec::new());
    }
    let graph = graph.expect("postgres-sql-fragment-escape requires canonical graph");
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut findings = Vec::new();
    for application in applications {
        let opts: Options = application.try_rule_options()?;
        let filter = super::path_filter::RulePathFilter::new(&root, config, application)?;
        let mut next = scan::scan(&root, &opts.trusted_sql_tags, graph, facts, sources);
        next.retain(|finding| filter.is_match(&root.join(&finding.file)));
        findings.extend(next);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}
