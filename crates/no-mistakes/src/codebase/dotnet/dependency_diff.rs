use std::collections::{BTreeMap, BTreeSet};

mod lock;
mod projection;
mod xml;
pub(crate) use lock::dependency_only_lockfile_change;
pub(in crate::codebase::dotnet) use xml::parse_open_tag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DotnetDependencyDiff {
    pub(crate) dependency_only: bool,
    pub(crate) formatting_only: bool,
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

fn diff_result(
    before: (String, BTreeMap<String, String>, String),
    after: (String, BTreeMap<String, String>, String),
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
        formatting_only: before.2 == after.2,
        changed_dependencies,
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "dependency_diff/tests.rs"]
mod tests;
