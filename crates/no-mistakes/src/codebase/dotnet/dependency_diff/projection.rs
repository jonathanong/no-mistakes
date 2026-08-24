use super::xml::{dependency_items, dependency_map, normalize_xml, validate_xml};
use super::DotnetDependencyDiagnostic;
use std::collections::BTreeMap;

pub(super) fn project_projection(
    source: &str,
) -> Result<(String, BTreeMap<String, String>, String), DotnetDependencyDiagnostic> {
    project_without_dependency_items(source, &["PackageReference", "ProjectReference"])
}

pub(super) fn central_projection(
    source: &str,
) -> Result<(String, BTreeMap<String, String>, String), DotnetDependencyDiagnostic> {
    project_without_dependency_items(source, &["PackageVersion"])
}

fn project_without_dependency_items(
    source: &str,
    tags: &[&str],
) -> Result<(String, BTreeMap<String, String>, String), DotnetDependencyDiagnostic> {
    validate_xml(source)?;
    let document = super::super::dependency_fingerprint::full_document_fingerprint(source)?;
    let mut projection = source.to_string();
    let dependencies = dependency_items(source, tags)?;
    for item in dependencies.iter().rev() {
        projection.replace_range(item.start..item.end, "");
    }
    Ok((
        normalize_xml(&projection),
        dependency_map(dependencies)?,
        document,
    ))
}
