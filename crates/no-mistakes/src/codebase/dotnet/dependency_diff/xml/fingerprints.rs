use super::{DependencyItem, DotnetDependencyDiagnostic};
use std::collections::BTreeMap;

pub(in crate::codebase::dotnet) fn dependency_map(
    items: Vec<DependencyItem>,
) -> Result<BTreeMap<String, String>, DotnetDependencyDiagnostic> {
    items
        .into_iter()
        .map(|item| {
            Ok((
                item.name,
                super::super::super::dependency_fingerprint::dependency_fingerprint(&item.source)?,
            ))
        })
        .collect()
}
