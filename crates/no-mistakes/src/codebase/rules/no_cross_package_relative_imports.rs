use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::{DepGraph, ImportTarget, TsFactLookup};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
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
) -> Result<Vec<RuleFinding>> {
    let inputs = Inputs {
        root,
        shared,
        graph,
        workspace,
    };
    super::run_rule_applications(config, RULE_ID, |rule| {
        let files = super::selected_rule_files(root, config, rule, shared.graph_file_universe())?;
        let message = rule
            .message
            .as_deref()
            .map_or_else(|| FIX.to_owned(), |prefix| format!("{prefix}: {FIX}"));
        let results: Vec<_> = files
            .par_iter()
            .map(|file| inputs.check_file(file, &message))
            .collect();
        let mut findings = Vec::new();
        for result in results {
            if let Some(output) = session.capture_check_result(RULE_ID, result)? {
                findings.extend(output);
            }
        }
        Ok(findings)
    })
}

struct Inputs<'a> {
    root: &'a Path,
    shared: &'a CheckFactMap,
    graph: &'a DepGraph,
    workspace: &'a IndexedWorkspaceMap,
}

impl Inputs<'_> {
    fn check_file(&self, file: &Path, message: &str) -> Result<Vec<RuleFinding>> {
        let Some((owner, facts)) = self
            .workspace
            .nearest_package(file)
            .zip(self.shared.get_ts_facts(file))
        else {
            return Ok(Vec::new());
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

        Ok(facts
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
                    file: relative_slash_path(self.root, file),
                    line: import.line as usize,
                    message: message.to_owned(),
                    import: Some(import.specifier.clone()),
                    target: Some(relative_slash_path(self.root, target)),
                    causes: Vec::new(),
                })
            })
            .collect())
    }
}

fn is_relative(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
