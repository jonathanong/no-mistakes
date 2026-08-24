use std::collections::{BTreeMap, BTreeSet};

mod lock;
mod projection;
mod xml;
pub(in crate::codebase::dotnet) use xml::parse_open_tag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DotnetDependencyDiff {
    pub(crate) dependency_only: bool,
    pub(crate) changed_dependencies: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DotnetDependencyDiagnostic {
    UnsupportedDynamicDeclaration,
    MalformedXml,
    MalformedLockfile,
    UnsupportedLockSchema,
}

/// Returns true only when the static dependency declarations are the complete
/// semantic change. MSBuild conditions, imports, and property expansion are
/// intentionally diagnostic rather than guessed.
pub(crate) fn dependency_only_project_change(
    before: &str,
    after: &str,
) -> Result<DotnetDependencyDiff, DotnetDependencyDiagnostic> {
    let before = projection::project_projection(before)?;
    let after = projection::project_projection(after)?;
    Ok(diff_result(before, after))
}

pub(crate) fn dependency_only_central_packages_change(
    before: &str,
    after: &str,
) -> Result<DotnetDependencyDiff, DotnetDependencyDiagnostic> {
    let before = projection::central_projection(before)?;
    let after = projection::central_projection(after)?;
    Ok(diff_result(before, after))
}

pub(crate) fn dependency_only_lockfile_change(
    before: &str,
    after: &str,
) -> Result<DotnetDependencyDiff, DotnetDependencyDiagnostic> {
    let before = lock::lock_projection(before)?;
    let after = lock::lock_projection(after)?;
    let changed_dependencies = before
        .keys()
        .chain(after.keys())
        .filter(|key| before.get(*key) != after.get(*key))
        .cloned()
        .collect();
    Ok(DotnetDependencyDiff {
        dependency_only: before != after,
        changed_dependencies,
    })
}

fn diff_result(
    before: (String, BTreeMap<String, String>),
    after: (String, BTreeMap<String, String>),
) -> DotnetDependencyDiff {
    let changed_dependencies = before
        .1
        .keys()
        .chain(after.1.keys())
        .filter(|key| before.1.get(*key) != after.1.get(*key))
        .cloned()
        .collect();
    DotnetDependencyDiff {
        dependency_only: before.0 == after.0 && before.1 != after.1,
        changed_dependencies,
    }
}

#[cfg(test)]
#[path = "dependency_diff_tests.rs"]
mod dependency_diff_tests;
