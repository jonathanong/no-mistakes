//! Ensure every tracked TypeScript project is registered in CI and local checks.
//!
//! The rule deliberately recognizes only a small, static command grammar. It
//! reports missing registrations instead of guessing through shell indirection
//! or expressions, so its results remain deterministic and actionable.

mod application;
mod command_scan;
mod no_check;
mod workflow;

use super::RuleFinding;
use crate::codebase::ci_workflows::ParsedWorkflowSet;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::{CheckFileArgs, NoMistakesConfig};
use anyhow::Result;
use application::{scan_application, Options};
use command_scan::scan_argv_for_typechecked_projects;
use no_check::non_enforcing_tsconfigs;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use workflow::{ci_typechecked_projects, workflow_load_findings};

pub const RULE_ID: &str = "tsconfig-gate-coverage";

/// Request-owned inputs supplied by the aggregate check runner.
///
/// `tracked_paths` is intentionally an inventory, rather than a
/// [`crate::codebase::ts_resolver::TsConfigCatalog`]: compiler helper configs
/// such as `tsconfig.tools.json` are part of this policy even when they do not
/// own imports for ordinary codebase resolution.
pub(crate) struct PreparedInputs<'a> {
    pub(crate) tracked_paths: &'a [PathBuf],
    pub(crate) workflows: &'a ParsedWorkflowSet,
    /// The owner of all rule source reads, including workflow documents and
    /// effective `compilerOptions.noCheck` resolution for tracked tsconfigs.
    pub(crate) sources: &'a crate::codebase::ts_source::SourceStore,
    /// The loaded config path, used for configuration diagnostics and
    /// suppressions. `None` renders as the conventional `.no-mistakes.yml`.
    pub(crate) config_path: Option<&'a Path>,
}

/// Run the rule with request-prepared tracked paths and workflow documents.
///
/// The check runner is responsible for invoking this only when the rule is
/// configured and for supplying a source-store-backed [`ParsedWorkflowSet`].
pub(crate) fn check_with_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    prepared: PreparedInputs<'_>,
) -> Result<Vec<RuleFinding>> {
    let tracked = tracked_tsconfigs(root, prepared.tracked_paths);
    let non_enforcing = non_enforcing_tsconfigs(root, &tracked, prepared.sources);
    let ci_projects = ci_typechecked_projects(prepared.workflows, &tracked);
    let local_projects = local_typechecked_projects(config);
    let config_file = config_file(root, prepared.config_path);
    let workflow_errors = workflow_load_findings(prepared.workflows);

    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let candidates = tracked
                .iter()
                .filter(|candidate| {
                    let path = root.join(candidate);
                    super::file_allowed_by_roots_and_skip(root, &skip, &path, &target_roots)
                })
                .cloned()
                .collect::<Vec<_>>();
            let candidates = super::path_filter::filter_rule_files(
                root,
                config,
                rule,
                &candidates
                    .iter()
                    .map(|candidate| root.join(candidate))
                    .collect::<Vec<_>>(),
            )?;
            let candidates = candidates
                .iter()
                .map(|path| relative_slash_path(root, path))
                .collect::<BTreeSet<_>>();
            Ok(scan_application(
                &opts,
                &tracked,
                &candidates,
                &ci_projects,
                &local_projects,
                &non_enforcing,
                &config_file,
            ))
        })
        .collect();
    let mut findings = all?.into_iter().flatten().collect::<Vec<_>>();
    findings.extend(workflow_errors);
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn tracked_tsconfigs(root: &Path, paths: &[PathBuf]) -> BTreeSet<String> {
    paths
        .iter()
        .filter_map(|path| {
            let rel = relative_slash_path(root, path);
            command_scan::normalize_repo_relative(&rel)
                .filter(|normalized| is_tsconfig_path(normalized))
        })
        .collect()
}

fn is_tsconfig_path(path: &str) -> bool {
    if path.split('/').any(|component| component == "node_modules") {
        return false;
    }
    let name = path.rsplit('/').next().unwrap_or_default();
    name == "tsconfig.json"
        || name
            .strip_prefix("tsconfig.")
            .is_some_and(|suffix| !suffix.is_empty() && suffix.ends_with(".json"))
}

fn local_typechecked_projects(config: &NoMistakesConfig) -> BTreeSet<String> {
    config
        .checks
        .commands
        .iter()
        .filter(|command| command.always && command.file_args == CheckFileArgs::None)
        .flat_map(|command| scan_argv_for_typechecked_projects(&command.command, "."))
        .collect()
}

fn config_file(root: &Path, config_path: Option<&Path>) -> String {
    config_path.map_or_else(
        || ".no-mistakes.yml".to_string(),
        |path| {
            relative_slash_path(
                &crate::codebase::ts_resolver::normalize_path(root),
                &crate::codebase::ts_resolver::normalize_path(path),
            )
        },
    )
}

#[cfg(test)]
mod tests;
