use super::options::PackagePair;
use super::RULE_ID;
use crate::codebase::dependencies::graph::ImportTarget;
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspace_resolution::{IndexedWorkspaceMap, WorkspacePackage};
use crate::config::v2::Unanalyzable;
use std::path::Path;

pub(super) struct SpecifierCheck<'a> {
    pub(super) file: &'a Path,
    pub(super) relative_file: &'a str,
    pub(super) owner: &'a WorkspacePackage,
    pub(super) message: &'a str,
    pub(super) allow: &'a [PackagePair],
    pub(super) policy: Unanalyzable,
}

pub(super) fn stays_in_owner_package(
    file: &Path,
    specifier: &str,
    owner: &WorkspacePackage,
    workspace: &IndexedWorkspaceMap,
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

pub(super) fn check_resolved_or_relative_target(
    root: &Path,
    workspace: &IndexedWorkspaceMap,
    ctx: &SpecifierCheck<'_>,
    line: u32,
    specifier: &str,
    target: Option<&ImportTarget>,
    diagnostics: &mut AnalysisDiagnostics,
) {
    if let Some(ImportTarget::File(target)) = target {
        report_resolved_target(root, workspace, ctx, line, specifier, target, diagnostics);
        return;
    }
    if stays_in_owner_package(ctx.file, specifier, ctx.owner, workspace) {
        return;
    }
    diagnostics.unanalyzable(
        ctx.policy,
        unanalyzable(
            ctx.relative_file,
            line,
            specifier,
            "the relative specifier has no unique file target. Run resolve-check and fix resolution.",
        ),
    );
}

fn report_resolved_target(
    root: &Path,
    workspace: &IndexedWorkspaceMap,
    ctx: &SpecifierCheck<'_>,
    line: u32,
    specifier: &str,
    target: &Path,
    diagnostics: &mut AnalysisDiagnostics,
) {
    let target_owner = workspace.nearest_package(target);
    let crosses_boundary = !target.starts_with(&ctx.owner.dir)
        || target_owner.is_some_and(|pkg| pkg.dir != ctx.owner.dir);
    let allowed = target_owner.is_some_and(|pkg| {
        ctx.allow
            .iter()
            .any(|pair| pair.matches(&ctx.owner.name, &pkg.name))
    });
    if crosses_boundary && !allowed {
        diagnostics.findings.push(finding(
            ctx.relative_file,
            line,
            specifier,
            Some(relative_slash_path(root, target)),
            ctx.message.to_owned(),
        ));
    }
}

pub(super) fn unanalyzable(file: &str, line: u32, specifier: &str, reason: &str) -> RuleFinding {
    finding(
        file,
        line,
        specifier,
        None,
        format!("Unanalyzable import: {reason}"),
    )
}

pub(super) fn is_relative(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}

fn finding(
    file: &str,
    line: u32,
    specifier: &str,
    target: Option<String>,
    message: String,
) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: file.to_owned(),
        line: line as usize,
        message,
        import: Some(specifier.to_owned()),
        target,
        causes: Vec::new(),
    }
}
