use super::format::{missing_finding, unanalyzable_finding};
use super::judge::{call_arguments, judge_arguments, DelayVerdict};
use super::matching::{candidate_targets, target_matches};
use super::options::Target;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::no_single_enqueue_in_loop::modules::ResolvedModules;
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use crate::config::v2::Unanalyzable;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct ScanInput<'a> {
    pub(super) root: &'a Path,
    pub(super) config: &'a NoMistakesConfig,
    pub(super) application: &'a RuleDef,
    pub(super) policy: Unanalyzable,
    pub(super) targets: &'a [Target],
    pub(super) graph: &'a DepGraph,
    pub(super) facts: &'a CheckFactMap,
    pub(super) resolved: &'a ResolvedModules,
}

pub(super) fn scan(input: &ScanInput<'_>) -> Result<AnalysisDiagnostics> {
    let ScanInput {
        root,
        config,
        application,
        policy,
        targets,
        graph,
        facts,
        resolved,
    } = *input;
    let selected = selected_files(root, config, application, facts)?;
    let mut diagnostics = AnalysisDiagnostics::default();
    collect_parse_failures(
        root,
        application,
        policy,
        &selected,
        facts,
        &mut diagnostics,
    );
    for site in graph.resolved_call_sites() {
        if !matches!(
            site.invocation,
            InvocationKind::Construct | InvocationKind::Call
        ) {
            continue;
        }
        let file = crate::codebase::ts_resolver::normalize_path(&site.file);
        if !selected.contains(&file) {
            continue;
        }
        let Some(ts_facts) = facts.get_ts_facts(&file) else {
            continue;
        };
        let call_args = call_arguments(ts_facts, site.offset);
        if !site.has_unknown_target() {
            for target in targets
                .iter()
                .filter(|target| target_matches(graph, target, site, resolved))
            {
                match judge_arguments(call_args.values, call_args.absent, target) {
                    DelayVerdict::Valid => {}
                    DelayVerdict::Missing(detail) => {
                        diagnostics.findings.push(missing_finding(
                            root,
                            application,
                            &file,
                            site,
                            target,
                            &detail,
                        ));
                    }
                    DelayVerdict::Unanalyzable(detail) => {
                        let finding = unanalyzable_finding(
                            root,
                            application,
                            &file,
                            site.line,
                            Some(site),
                            &detail,
                        );
                        diagnostics.unanalyzable(policy, finding);
                    }
                }
            }
            continue;
        }
        let suspect = candidate_targets(graph, targets, site, ts_facts, resolved);
        if suspect.is_empty() {
            continue;
        }
        let all_valid = suspect.iter().all(|target| {
            matches!(
                judge_arguments(call_args.values, call_args.absent, target),
                DelayVerdict::Valid
            )
        });
        if !all_valid {
            let detail = format!(
                "callee `{}` could not be resolved to determine whether it is a configured rate-limit error",
                site.source_callee
            );
            let finding =
                unanalyzable_finding(root, application, &file, site.line, Some(site), &detail);
            diagnostics.unanalyzable(policy, finding);
        }
    }
    Ok(diagnostics)
}

fn collect_parse_failures(
    root: &Path,
    application: &RuleDef,
    policy: Unanalyzable,
    selected: &HashSet<PathBuf>,
    facts: &CheckFactMap,
    diagnostics: &mut AnalysisDiagnostics,
) {
    let mut files: Vec<&PathBuf> = selected.iter().collect();
    files.sort();
    for file in files {
        let Some(ts_facts) = facts.get_ts_facts(file) else {
            continue;
        };
        let reason = ts_facts
            .parse_error
            .as_deref()
            .or(ts_facts.operational_error.as_deref());
        if let Some(reason) = reason {
            let detail = format!("file could not be parsed: {reason}");
            diagnostics.unanalyzable(
                policy,
                unanalyzable_finding(root, application, file, 1, None, &detail),
            );
        }
    }
}

fn selected_files(
    root: &Path,
    config: &NoMistakesConfig,
    application: &RuleDef,
    facts: &CheckFactMap,
) -> Result<HashSet<PathBuf>> {
    let files =
        super::super::selected_rule_files(root, config, application, facts.graph_file_universe())?;
    Ok(files
        .into_iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
        .collect())
}
