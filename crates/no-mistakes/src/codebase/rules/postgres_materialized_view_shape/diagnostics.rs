use super::{compile, scan, Compiled, RuleFinding, RULE_ID};
use crate::codebase::{
    check_facts::CheckFactMap, rules::analysis_policy::AnalysisDiagnostics, ts_source::SourceStore,
};
use crate::config::v2::{
    schema::{RuleDef, RuleSeverity},
    NoMistakesConfig,
};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn diagnostics_with_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let applications: Result<Vec<_>> = config
        .rule_applications(RULE_ID)
        .into_iter()
        .map(|rule| Ok((rule, compile(&rule.try_rule_options()?)?)))
        .collect();
    let applications = applications?;
    if applications.is_empty() {
        return Ok(AnalysisDiagnostics::default());
    }
    let standalone;
    let facts = match facts {
        Some(facts) => facts,
        None => {
            standalone =
                super::super::prepare_schema_rule_facts(root, config, files, sources, RULE_ID)?;
            &standalone
        }
    };
    let batches: Result<Vec<_>> = applications
        .into_par_iter()
        .map(|(rule, compiled)| {
            let report = compiled.report && rule.severity != RuleSeverity::Advisory;
            let mut findings = scan_application(root, config, rule, compiled, facts)?;
            if !defer_suppression {
                super::super::suppress_rule_findings_with_sources(root, &mut findings, sources);
            }
            let mut output = AnalysisDiagnostics::default();
            if report {
                output.findings = findings;
            } else {
                output.advisories = findings;
            }
            Ok(output)
        })
        .collect();
    let mut output = AnalysisDiagnostics::default();
    for batch in batches? {
        output.append(batch);
    }
    super::super::sort_findings(&mut output.findings);
    super::super::sort_findings(&mut output.advisories);
    Ok(output)
}

pub(super) fn scan_application(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    compiled: Compiled,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let Some(catalog) =
        super::super::selected_schema_catalog(root, config, rule, facts, &compiled.path)?
    else {
        return Ok(Vec::new());
    };
    let mut findings = scan::scan(compiled, catalog);
    if let Some(message) = &rule.message {
        for finding in &mut findings {
            finding.message = format!("{message} {}", finding.message);
        }
    }
    Ok(findings)
}
