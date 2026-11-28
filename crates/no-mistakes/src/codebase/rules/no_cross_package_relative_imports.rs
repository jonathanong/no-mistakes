use super::analysis_policy::AnalysisDiagnostics;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::{DepGraph, ImportTarget, TsFactLookup};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use no_mistakes_rule_kernel::finding::RuleFinding;
use rayon::prelude::*;
use std::path::Path;

pub const RULE_ID: &str = "no-cross-package-relative-imports";
const FIX: &str = "Use a declared package import through the target package's public exports instead of a relative path across the workspace package boundary.";

pub(super) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &CheckFactMap,
    graph: &DepGraph,
    workspace: &IndexedWorkspaceMap,
    session: &AnalysisSession,
) -> Result<AnalysisDiagnostics> {
    let inputs = Inputs {
        root,
        shared,
        graph,
        workspace,
    };
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let files = super::selected_rule_files(root, config, rule, shared.graph_file_universe())?;
        let message = rule
            .message
            .as_deref()
            .map_or_else(|| FIX.to_owned(), |prefix| format!("{prefix}: {FIX}"));
        let policy = config.effective_rule_analysis(rule)?.unanalyzable;
        let results: Vec<_> = files
            .par_iter()
            .map(|file| inputs.check_file(file, &message, policy))
            .collect();
        for result in results {
            if let Some(output) = session.capture_check_result(RULE_ID, result)? {
                diagnostics.append(output);
            }
        }
    }
    super::sort_findings(&mut diagnostics.findings);
    super::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

struct Inputs<'a> {
    root: &'a Path,
    shared: &'a CheckFactMap,
    graph: &'a DepGraph,
    workspace: &'a IndexedWorkspaceMap,
}

impl Inputs<'_> {
    fn check_file(
        &self,
        file: &Path,
        message: &str,
        policy: crate::config::v2::Unanalyzable,
    ) -> Result<AnalysisDiagnostics> {
        let Some((owner, facts)) = self
            .workspace
            .nearest_package(file)
            .zip(self.shared.get_ts_facts(file))
        else {
            return Ok(AnalysisDiagnostics::default());
        };
        if let Some(error) = facts
            .parse_error
            .as_deref()
            .or(facts.operational_error.as_deref())
        {
            return Err(
                CheckInputFailure::source(relative_slash_path(self.root, file), error).into(),
            );
        }

        let relative_file = relative_slash_path(self.root, file);
        let mut diagnostics = AnalysisDiagnostics::default();
        diagnostics.findings.extend(
            facts
                .imports
                .iter()
                .filter(|import| {
                    !import.computed
                        && import.kind != ImportKind::RequireResolve
                        && is_relative(&import.specifier)
                })
                .filter_map(|import| {
                    let Some(ImportTarget::File(target)) =
                        self.graph.import_target(file, &import.specifier)
                    else {
                        return None;
                    };
                    let crosses_boundary = !target.starts_with(&owner.dir)
                        || self
                            .workspace
                            .nearest_package(target)
                            .is_some_and(|target_owner| target_owner.dir != owner.dir);
                    crosses_boundary.then(|| RuleFinding {
                        rule: RULE_ID.to_owned(),
                        file: relative_file.clone(),
                        line: import.line as usize,
                        message: message.to_owned(),
                        import: Some(import.specifier.clone()),
                        target: Some(relative_slash_path(self.root, target)),
                        causes: Vec::new(),
                    })
                }),
        );
        self.record_vitest_calls(
            &vitest::VitestSite {
                file,
                relative_file: &relative_file,
                owner,
                message,
                policy,
            },
            facts,
            &mut diagnostics,
        );
        Ok(diagnostics)
    }
}

fn is_relative(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}

mod vitest;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
