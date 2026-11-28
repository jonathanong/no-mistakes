use super::{is_relative, Inputs, RULE_ID};
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::{vitest_import_calls, ImportTarget, VitestReceiver};
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
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
            if facts.imports.iter().any(|import| {
                import.kind != ImportKind::RequireResolve
                    && import.line == call.line
                    && import.specifier == call.specifier
            }) {
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
        record_direct_vitest_target(
            self.root,
            self.workspace,
            site,
            line,
            specifier,
            self.graph.import_target(site.file, specifier),
            diagnostics,
        );
    }
}

pub(super) fn record_direct_vitest_target(
    root: &Path,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    site: &VitestSite<'_>,
    line: u32,
    specifier: &str,
    target: Option<&ImportTarget>,
    diagnostics: &mut AnalysisDiagnostics,
) {
    if let Some(ImportTarget::File(target)) = target {
        if super::crosses_boundary(target, site.owner, workspace) {
            diagnostics.findings.push(RuleFinding {
                rule: RULE_ID.to_owned(),
                file: site.relative_file.to_owned(),
                line: line as usize,
                message: site.message.to_owned(),
                import: Some(specifier.to_owned()),
                target: Some(crate::codebase::ts_source::relative_slash_path(
                    root, target,
                )),
                causes: Vec::new(),
            });
        }
        return;
    }
    if stays_in_owner_package(site.file, specifier, site.owner, workspace) {
        return;
    }
    diagnostics.unanalyzable(
        site.policy,
        unanalyzable(
            site.relative_file,
            line,
            specifier,
            "the relative specifier has no unique file target. Run resolve-check and fix resolution.",
        ),
    );
}

pub(super) fn stays_in_owner_package(
    file: &Path,
    specifier: &str,
    owner: &WorkspacePackage,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> bool {
    let path_part = specifier.split(['?', '#']).next().unwrap_or(specifier);
    file.parent().is_some_and(|dir| {
        let candidate = normalize_path(&dir.join(path_part));
        candidate.starts_with(&owner.dir)
            && workspace
                .nearest_package(&candidate)
                .is_some_and(|pkg| pkg.dir == owner.dir)
    })
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
