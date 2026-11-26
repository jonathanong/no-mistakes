use super::{is_relative, Inputs, RULE_ID};
use crate::codebase::dependencies::graph::{vitest_import_calls, ImportTarget, VitestReceiver};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::workspaces::WorkspacePackage;
use crate::config::v2::Unanalyzable;
use no_mistakes_rule_kernel::finding::RuleFinding;
use std::path::Path;

impl Inputs<'_> {
    pub(super) fn record_vitest_calls(
        &self,
        file: &Path,
        relative_file: &str,
        owner: &WorkspacePackage,
        facts: &TsFileFacts,
        message: &str,
        policy: Unanalyzable,
        diagnostics: &mut AnalysisDiagnostics,
    ) {
        for call in vitest_import_calls(facts) {
            if !(call.computed || is_relative(call.specifier)) {
                continue;
            }
            if facts
                .imports
                .iter()
                .any(|import| import.line == call.line && import.specifier == call.specifier)
            {
                continue;
            }
            match call.receiver {
                VitestReceiver::Direct => self.record_direct_vitest_call(
                    file,
                    relative_file,
                    owner,
                    call.line,
                    call.specifier,
                    call.computed,
                    message,
                    policy,
                    diagnostics,
                ),
                VitestReceiver::Unproven { imported_from } => {
                    if imported_from.is_some_and(|source| {
                        self.graph.import_target(file, source) == Some(&ImportTarget::External)
                    }) || (!call.computed
                        && stays_in_owner_package(file, call.specifier, owner, self.workspace))
                    {
                        continue;
                    }
                    diagnostics.unanalyzable(
                        policy,
                        unanalyzable(
                            relative_file,
                            call.line,
                            call.specifier,
                            "the receiver of importActual or importMock is unresolved. Import vi directly from Vitest.",
                        ),
                    );
                }
            }
        }
    }

    fn record_direct_vitest_call(
        &self,
        file: &Path,
        relative_file: &str,
        owner: &WorkspacePackage,
        line: u32,
        specifier: &str,
        computed: bool,
        message: &str,
        policy: Unanalyzable,
        diagnostics: &mut AnalysisDiagnostics,
    ) {
        if computed {
            diagnostics.unanalyzable(
                policy,
                unanalyzable(
                    relative_file,
                    line,
                    specifier,
                    "the specifier is computed. Use a literal import specifier.",
                ),
            );
            return;
        }
        if stays_in_owner_package(file, specifier, owner, self.workspace) {
            return;
        }
        let Some(ImportTarget::File(target)) = self.graph.import_target(file, specifier) else {
            diagnostics.unanalyzable(
                policy,
                unanalyzable(
                    relative_file,
                    line,
                    specifier,
                    "the relative specifier has no unique file target. Run resolve-check and fix resolution.",
                ),
            );
            return;
        };
        let crosses_boundary = !target.starts_with(&owner.dir)
            || self
                .workspace
                .nearest_package(target)
                .is_some_and(|target_owner| target_owner.dir != owner.dir);
        if crosses_boundary {
            diagnostics.findings.push(RuleFinding {
                rule: RULE_ID.to_owned(),
                file: relative_file.to_owned(),
                line: line as usize,
                message: message.to_owned(),
                import: Some(specifier.to_owned()),
                target: Some(crate::codebase::ts_source::relative_slash_path(
                    self.root, target,
                )),
                causes: Vec::new(),
            });
        }
    }
}

pub(super) fn stays_in_owner_package(
    file: &Path,
    specifier: &str,
    owner: &WorkspacePackage,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> bool {
    let path_part = specifier.split(['?', '#']).next().unwrap_or(specifier);
    let Some(dir) = file.parent() else {
        return false;
    };
    let candidate = normalize_path(&dir.join(path_part));
    candidate.starts_with(&owner.dir)
        && workspace
            .nearest_package(&candidate)
            .is_some_and(|pkg| pkg.dir == owner.dir)
}

fn unanalyzable(file: &str, line: u32, specifier: &str, reason: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: file.to_owned(),
        line: line as usize,
        message: format!("Unanalyzable import: {reason}"),
        import: Some(specifier.to_owned()),
        target: None,
        causes: Vec::new(),
    }
}
