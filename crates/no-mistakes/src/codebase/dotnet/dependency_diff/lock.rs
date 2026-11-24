use super::{DotnetDependencyDiagnostic, DotnetDependencyDiff};
use crate::codebase::keyed_diff::{diff_entries, nuget_entries};
use serde_json::Value;

pub(crate) fn dependency_only_lockfile_change(
    before: &str,
    after: &str,
) -> Result<DotnetDependencyDiff, DotnetDependencyDiagnostic> {
    let mut before = parse_lockfile(before)?;
    let old_entries =
        nuget_entries(&before).map_err(|_| DotnetDependencyDiagnostic::UnsupportedLockSchema)?;
    let mut after = parse_lockfile(after)?;
    let new_entries =
        nuget_entries(&after).map_err(|_| DotnetDependencyDiagnostic::UnsupportedLockSchema)?;
    let changes = diff_entries(&old_entries, &new_entries);
    let formatting_only = before == after;
    before
        .as_object_mut()
        .expect("validated NuGet root")
        .remove("dependencies");
    after
        .as_object_mut()
        .expect("validated NuGet root")
        .remove("dependencies");
    Ok(DotnetDependencyDiff {
        dependency_only: before == after && !changes.is_empty(),
        formatting_only,
        changed_dependencies: changes.all_ids().collect(),
    })
}

fn parse_lockfile(source: &str) -> Result<Value, DotnetDependencyDiagnostic> {
    serde_json::from_str(source).map_err(|_| DotnetDependencyDiagnostic::MalformedLockfile)
}
