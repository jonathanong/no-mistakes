use super::{PreparedRuleContext, RuleFinding};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::{bail, ensure, Result};
use options::CompiledOptions;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod baseline;
mod embedded;
pub(in crate::codebase::rules) mod options;
mod report;
mod scan;
#[cfg(test)]
mod tests;
mod types;
pub use types::{
    LockInventoryBaseline, LockInventoryBaselineSite, LockInventoryReport, LockInventorySite,
    UnanalyzedLockSite,
};

pub const RULE_ID: &str = "postgres-lock-inventory";
pub(crate) const DEFAULT_BASELINE_FILE: &str = ".no-mistakes/postgres-locks.json";
super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

pub(crate) fn standalone_application(config: &NoMistakesConfig) -> Result<&RuleDef> {
    let rules = config.rule_applications(RULE_ID);
    let [rule] = rules.as_slice() else {
        bail!("configure exactly one enabled {RULE_ID} application to run no-mistakes postgres lock-inventory");
    };
    Ok(*rule)
}

pub(crate) fn inventory_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    facts: &CheckFactMap,
) -> Result<LockInventoryReport> {
    let rule = standalone_application(config)?;
    collect_application(
        &PreparedRuleContext::new(root, config, all_files, sources, facts),
        0,
        rule,
        &CompiledOptions::from_rule(rule)?,
    )
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let context = PreparedRuleContext::new(root, config, all_files, sources, facts);
    let rules = config.rule_applications(RULE_ID);
    let mut baselines = BTreeSet::new();
    let applications = rules
        .into_iter()
        .map(|rule| {
            let options = CompiledOptions::from_rule(rule)?;
            ensure!(
                baselines.insert(options.baseline_file.clone()),
                "configure a distinct {RULE_ID} baselineFile for each application ({})",
                options.baseline_file
            );
            Ok((rule, options))
        })
        .collect::<Result<Vec<_>>>()?;
    let application_findings = applications
        .par_iter()
        .enumerate()
        .map(
            |(index, (rule, options))| match collect_application(&context, index, rule, options) {
                Ok(report) => Ok(report_findings(report)),
                Err(error) if error.is::<baseline::InvalidBaseline>() => Ok(vec![finding(
                    options.baseline_file.clone(),
                    1,
                    format!(
                        "{error:#}. Repair the baseline before comparing PostgreSQL lock sites"
                    ),
                    "invalid-baseline".into(),
                )]),
                Err(error) => Err(error),
            },
        )
        .collect::<Result<Vec<_>>>()?;
    let mut findings: Vec<_> = application_findings.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn collect_application(
    context: &PreparedRuleContext<'_>,
    index: usize,
    rule: &RuleDef,
    options: &CompiledOptions,
) -> Result<LockInventoryReport> {
    let PreparedRuleContext {
        root,
        config,
        all_files,
        sources,
        facts,
    } = *context;
    let baseline = baseline::read_baseline(root, &options.baseline_file, sources)?;
    let mut files =
        super::selected_rule_files_with_sources(root, config, rule, all_files, sources)?;
    files.retain(|path| options.paths.includes(&relative_slash_path(root, path)));
    files.sort_unstable();
    files.dedup();
    let view = facts.for_sql_application(index, rule.name.as_deref());
    let mut report = scan::collect(
        root,
        options,
        &files,
        view.as_ref()
            .map_or(facts as &dyn SqlFactContext, |view| view),
    )?;
    baseline::compare(&mut report, baseline.as_ref());
    Ok(report)
}

fn report_findings(report: LockInventoryReport) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for site in report.added {
        findings.push(finding(
            site.file,
            site.line,
            format!("new PostgreSQL lock site ({}, {}). Remove the lock or record the reviewed site in the baseline", site.category, site.key),
            site.category,
        ));
    }
    for gaps in report.unanalyzed.chunk_by(PartialEq::eq) {
        for (index, gap) in gaps.iter().enumerate() {
            let reason = if gaps.len() == 1 {
                gap.reason.clone()
            } else {
                format!("{}, occurrence {} of {}", gap.reason, index + 1, gaps.len())
            };
            findings.push(finding(
                gap.file.clone(),
                gap.line,
                format!("incomplete PostgreSQL lock inventory ({reason}). Make the configured SQL or helper call statically analyzable"),
                "unanalyzable".into(),
            ));
        }
    }
    if !report.removed.is_empty() {
        findings.push(finding(
            report.baseline_file,
            1,
            format!("stale PostgreSQL lock baseline contains {} removed site(s). Run `no-mistakes postgres lock-inventory --write-baseline` after resolving any new sites", report.removed.len()),
            "stale-baseline".into(),
        ));
    }
    findings
}

fn finding(file: String, line: usize, message: String, target: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file,
        line,
        message,
        target: Some(target),
        import: None,
        causes: Vec::new(),
    }
}
