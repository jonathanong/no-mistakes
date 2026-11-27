//! Cross-file writes, rebinds, and namespace escapes of exported SQL fragments.
mod alias;
mod ambient;
mod dynamic;
mod fact_reads;
mod fragments;
mod gap_reachability;
mod import_resolution;
mod load_gap;
pub(crate) use import_resolution::FragmentImportResolution;
mod load_lex;
mod load_windows;
mod modules;
mod mutate;
mod names;
mod namespace_uses;
mod relevance;
mod resolve;
mod scan;
mod scan_file;
mod source_text;
mod tags;
mod text;
mod visible;
mod writes;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "tests/mod.rs"]
mod coverage_tests;

use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::check_outcome::CheckFailure;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::postgres::embedded::TrustedSqlTag;
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

pub(crate) struct FragmentEscapeCheck {
    pub findings: Vec<RuleFinding>,
    pub gaps: Vec<CheckFailure>,
}

pub const RULE_ID: &str = "postgres-sql-fragment-escape";

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(in crate::codebase::rules) struct Options {
    trusted_sql_tags: Vec<TrustedSqlTag>,
}

impl Options {
    pub(in crate::codebase::rules) fn validate(&self) -> Result<()> {
        for (index, tag) in self.trusted_sql_tags.iter().enumerate() {
            for (field, value) in [("module", &tag.module), ("name", &tag.name)] {
                anyhow::ensure!(
                    !value.trim().is_empty(),
                    "{RULE_ID}: options.trustedSqlTags[{index}].{field} must be nonblank"
                );
            }
        }
        Ok(())
    }
}

pub(crate) fn demands_analysis(config: &NoMistakesConfig) -> bool {
    config.rule_applications(RULE_ID).iter().any(|application| {
        application
            .try_rule_options::<Options>()
            .map_or(true, |options| has_trusted_tags(&options.trusted_sql_tags))
    })
}

fn has_trusted_tags(tags: &[TrustedSqlTag]) -> bool {
    tags.iter()
        .any(|tag| !tag.module.is_empty() && !tag.name.is_empty())
}

pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
    resolution: Option<&FragmentImportResolution<'_>>,
) -> Result<FragmentEscapeCheck> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(FragmentEscapeCheck {
            findings: Vec::new(),
            gaps: Vec::new(),
        });
    }
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut findings = Vec::new();
    let mut gaps = Vec::new();
    for application in applications {
        let opts: Options = application.try_rule_options()?;
        let filter = super::path_filter::RulePathFilter::new(&root, config, application)?;
        let mut next = scan::scan(
            &root,
            &opts.trusted_sql_tags,
            graph,
            facts,
            sources,
            resolution.map(|projection| projection as &dyn import_resolution::FragmentResolver),
        );
        next.findings
            .retain(|finding| filter.is_match(&root.join(&finding.file)));
        next.gaps
            .retain(|gap| gap.file.as_ref().is_none_or(|file| filter.is_match(file)));
        findings.extend(next.findings);
        gaps.extend(next.gaps);
    }
    super::sort_findings(&mut findings);
    gaps.sort();
    gaps.dedup();
    Ok(FragmentEscapeCheck { findings, gaps })
}
