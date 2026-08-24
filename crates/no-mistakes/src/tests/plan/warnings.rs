use super::{warning_key, Warning, WarningKey};
use crate::tests::prepared_plan::PreparedTestPlanRequest;
use std::collections::HashSet;

pub(super) fn extend_analysis_warnings(
    prepared: &PreparedTestPlanRequest,
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<WarningKey>,
) {
    for warning in prepared
        .lockfile_analysis
        .warnings
        .iter()
        .chain(prepared.package_manifest_analysis.warnings.iter())
        .chain(prepared.swift_resolved_analysis.warnings.iter())
        .chain(prepared.swift_manifest_analysis.warnings.iter())
        .chain(prepared.dotnet_dependency_analysis.warnings.iter())
    {
        if warnings_seen.insert(warning_key(warning)) {
            warnings.push(warning.clone());
        }
    }
}
