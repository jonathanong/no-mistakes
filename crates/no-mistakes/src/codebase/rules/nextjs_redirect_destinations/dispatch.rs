use super::options::Options;
use super::scan::scan_diagnostics;
use super::RULE_ID;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn collect(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    snapshot: Option<&crate::codebase::ts_source::VisiblePathSnapshot>,
    require_tracked_snapshot: bool,
) -> Result<AnalysisDiagnostics> {
    let all: Result<Vec<AnalysisDiagnostics>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<AnalysisDiagnostics> {
            let opts: Options = rule.try_rule_options()?;
            let analysis = config.effective_rule_analysis(rule)?;
            let target_roots = super::super::target_roots(root, config, rule);
            let skip = super::super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::super::path_filter::filter_rule_files(root, config, rule, &files)?;
            if opts.tracked_routes_only {
                if let Some(snapshot) = snapshot {
                    tracked(TrackedScan {
                        root,
                        config,
                        rule,
                        opts: &opts,
                        analysis,
                        files: &files,
                        sources,
                        snapshot,
                        target_roots: &target_roots,
                        skip: &skip,
                    })
                } else if require_tracked_snapshot {
                    Err(missing_tracked_inventory(root))
                } else {
                    Ok(scan_diagnostics(
                        root,
                        &opts,
                        &files,
                        &files,
                        &target_roots,
                        sources,
                        analysis,
                    ))
                }
            } else {
                Ok(scan_diagnostics(
                    root,
                    &opts,
                    &files,
                    &files,
                    &target_roots,
                    sources,
                    analysis,
                ))
            }
        })
        .collect();
    let mut diagnostics = AnalysisDiagnostics::default();
    for next in all? {
        diagnostics.append(next);
    }
    Ok(diagnostics)
}

struct TrackedScan<'a> {
    root: &'a Path,
    config: &'a NoMistakesConfig,
    rule: &'a crate::config::v2::schema::RuleDef,
    opts: &'a Options,
    analysis: crate::config::v2::EffectiveRuleAnalysis,
    files: &'a [PathBuf],
    sources: &'a crate::codebase::ts_source::SourceStore,
    snapshot: &'a crate::codebase::ts_source::VisiblePathSnapshot,
    target_roots: &'a [PathBuf],
    skip: &'a HashSet<&'a str>,
}

fn tracked(scan: TrackedScan<'_>) -> Result<AnalysisDiagnostics> {
    let snapshot = scan.snapshot;
    let mut diagnostics = AnalysisDiagnostics::default();
    for target_root in scan.target_roots {
        if !snapshot.git_index_available_for(target_root) {
            return Err(missing_tracked_inventory(target_root));
        }
        // A non-Git umbrella's fallback list must never prove membership
        // in its nested project's Git index.
        let inventory = snapshot.git_index_paths_for(target_root);
        let tracked = inventory
            .iter()
            .filter(|path| {
                path.starts_with(target_root)
                    && super::super::file_allowed_by_roots_and_skip(
                        scan.root,
                        scan.skip,
                        path,
                        scan.target_roots,
                    )
            })
            .cloned()
            .collect::<Vec<_>>();
        let tracked = super::super::path_filter::filter_rule_files(
            scan.root,
            scan.config,
            scan.rule,
            &tracked,
        )?;
        let mut target_files = scan.files.to_vec();
        target_files.extend(
            snapshot
                .paths_for(target_root)
                .iter()
                .filter(|path| {
                    super::super::file_allowed_by_roots_and_skip(
                        scan.root,
                        scan.skip,
                        path,
                        scan.target_roots,
                    )
                })
                .cloned(),
        );
        target_files.sort();
        target_files.dedup();
        let target_files = super::super::path_filter::filter_rule_files(
            scan.root,
            scan.config,
            scan.rule,
            &target_files,
        )?;
        // Supplemental scoped configs still read through the one request
        // store, retaining shared success/failure identity.
        diagnostics.append(scan_diagnostics(
            scan.root,
            scan.opts,
            &target_files,
            &tracked,
            std::slice::from_ref(target_root),
            scan.sources,
            scan.analysis,
        ));
    }
    Ok(diagnostics)
}

pub(super) fn missing_tracked_inventory(root: &Path) -> anyhow::Error {
    anyhow::anyhow!(
        "nextjs-redirect-destinations trackedRoutesOnly requires a prepared Git index inventory for {}. The rule fails closed because filesystem pages cannot prove Git index membership. Run against a Git-backed configured project, supply an authoritative tracked list through Rust run_filesystem_rules_with_files(), or set trackedRoutesOnly: false to use filesystem routes.",
        root.display()
    )
}

pub(super) fn finish_findings(mut diagnostics: AnalysisDiagnostics) -> Vec<RuleFinding> {
    super::super::sort_findings(&mut diagnostics.findings);
    diagnostics.findings
}

pub(super) fn finish_advisories(mut diagnostics: AnalysisDiagnostics) -> Vec<RuleFinding> {
    super::super::sort_findings(&mut diagnostics.advisories);
    diagnostics.advisories
}
