use super::{is_relative, Inputs, RULE_ID};
use crate::codebase::dependencies::graph::{vitest_import_calls, ImportTarget, VitestReceiver};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::workspaces::WorkspacePackage;
use crate::config::v2::Unanalyzable;
use no_mistakes_rule_kernel::finding::RuleFinding;
use std::path::Path;

pub(super) struct VitestSite<'a> {
    pub(super) file: &'a Path,
    pub(super) relative_file: &'a str,
    pub(super) owner: &'a WorkspacePackage,
    pub(super) message: &'a str,
    pub(super) policy: Unanalyzable,
}

impl Inputs<'_> {
    pub(super) fn record_vitest_calls(
        &self,
        site: &VitestSite<'_>,
        facts: &TsFileFacts,
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
                    site,
                    call.line,
                    call.specifier,
                    call.computed,
                    diagnostics,
                ),
                VitestReceiver::Unproven { imported_from } => {
                    if imported_from.is_some_and(|source| {
                        self.graph.import_target(site.file, source) == Some(&ImportTarget::External)
                    }) || (!call.computed
                        && stays_in_owner_package(
                            site.file,
                            call.specifier,
                            site.owner,
                            self.workspace,
                        ))
                    {
                        continue;
                    }
                    diagnostics.unanalyzable(
                        site.policy,
                        unanalyzable(
                            site.relative_file,
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
        site: &VitestSite<'_>,
        line: u32,
        specifier: &str,
        computed: bool,
        diagnostics: &mut AnalysisDiagnostics,
    ) {
        if computed {
            diagnostics.unanalyzable(
                site.policy,
                unanalyzable(
                    site.relative_file,
                    line,
                    specifier,
                    "the specifier is computed. Use a literal import specifier.",
                ),
            );
            return;
        }
        if stays_in_owner_package(site.file, specifier, site.owner, self.workspace) {
            return;
        }
        let Some(ImportTarget::File(target)) = self.graph.import_target(site.file, specifier)
        else {
            diagnostics.unanalyzable(
                site.policy,
                unanalyzable(
                    site.relative_file,
                    line,
                    specifier,
                    "the relative specifier has no unique file target. Run resolve-check and fix resolution.",
                ),
            );
            return;
        };
        let crosses_boundary = !target.starts_with(&site.owner.dir)
            || self
                .workspace
                .nearest_package(target)
                .is_some_and(|target_owner| target_owner.dir != site.owner.dir);
        if crosses_boundary {
            diagnostics.findings.push(RuleFinding {
                rule: RULE_ID.to_owned(),
                file: site.relative_file.to_owned(),
                line: line as usize,
                message: site.message.to_owned(),
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
