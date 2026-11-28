use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::dependencies::extract::{is_indexable, ImportKind};
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspace_resolution::IndexedWorkspaceMap;
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

mod boundary;
mod options;
mod vitest;
use boundary::{check_resolved_or_relative_target, is_relative, unanalyzable, SpecifierCheck};
use options::PackagePair;

pub(in crate::codebase::rules) use options::accept_declared_options;
pub(super) use options::validate as validate_options;

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
        let allow = options::compile(rule, workspace)?;
        let policy = config.effective_rule_analysis(rule)?.unanalyzable;
        let files = super::selected_rule_files(root, config, rule, shared.graph_file_universe())?;
        let message = rule
            .message
            .as_deref()
            .map_or_else(|| FIX.to_owned(), |prefix| format!("{prefix}: {FIX}"));
        let results: Vec<_> = files
            .par_iter()
            .filter(|file| is_indexable(file))
            .map(|file| inputs.check_file(file, &message, &allow, policy))
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
        allow: &[PackagePair],
        policy: Unanalyzable,
    ) -> Result<AnalysisDiagnostics> {
        let Some(owner) = self.workspace.nearest_package(file) else {
            return Ok(AnalysisDiagnostics::default());
        };
        let relative_file = relative_slash_path(self.root, file);
        let source_failure = |error: &str| {
            CheckInputFailure::source(&relative_file, format!("Unanalyzable source: {error}"))
        };
        let facts = self
            .shared
            .get_ts_facts(file)
            .ok_or_else(|| source_failure("prepared import facts are missing"))?;
        if let Some(error) = facts
            .parse_error
            .as_deref()
            .or(facts.operational_error.as_deref())
        {
            return Err(source_failure(error).into());
        }
        let ctx = SpecifierCheck {
            file,
            relative_file: &relative_file,
            owner,
            message,
            allow,
            policy,
        };
        let mut diagnostics = AnalysisDiagnostics::default();
        for import in facts
            .imports
            .iter()
            .filter(|import| import.kind != ImportKind::RequireResolve)
            .filter(|import| import.computed || is_relative(&import.specifier))
        {
            self.check_specifier(
                &ctx,
                import.line,
                &import.specifier,
                import.computed,
                &mut diagnostics,
            );
        }
        self.record_vitest_calls(&ctx, facts, &mut diagnostics);
        Ok(diagnostics)
    }

    pub(super) fn check_specifier(
        &self,
        ctx: &SpecifierCheck<'_>,
        line: u32,
        specifier: &str,
        computed: bool,
        diagnostics: &mut AnalysisDiagnostics,
    ) {
        if computed {
            diagnostics.unanalyzable(
                ctx.policy,
                unanalyzable(
                    ctx.relative_file,
                    line,
                    specifier,
                    "the specifier is computed. Use a literal import specifier.",
                ),
            );
            return;
        }
        check_resolved_or_relative_target(
            self.root,
            self.workspace,
            ctx,
            line,
            specifier,
            self.graph.import_target(ctx.file, specifier),
            diagnostics,
        );
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
