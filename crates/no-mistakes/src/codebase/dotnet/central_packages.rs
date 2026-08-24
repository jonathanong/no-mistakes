use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) fn central_package_imports(
    central: &Path,
    source: &str,
    central_files: &BTreeSet<PathBuf>,
) -> Vec<PathBuf> {
    static IMPORT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?is)<Import\b[^>]*>").expect("valid static MSBuild Import regex")
    });
    let Some(source) = without_xml_ignored_regions(source) else {
        return central_ancestor_files(central, central_files);
    };
    IMPORT
        .captures_iter(&source)
        .filter_map(|capture| {
            capture
                .get(0)
                .and_then(|tag| import_project_value(tag.as_str()))
        })
        .filter_map(|project| central_package_import_target(central, project, central_files))
        .collect()
}

pub(crate) fn central_ancestor_files(
    central: &Path,
    central_files: &BTreeSet<PathBuf>,
) -> Vec<PathBuf> {
    let Some(central_dir) = central.parent() else {
        return Vec::new();
    };
    central_files
        .iter()
        .filter(|candidate| *candidate != central)
        .filter(|candidate| {
            candidate
                .parent()
                .is_some_and(|parent| central_dir.starts_with(parent))
        })
        .cloned()
        .collect()
}

fn import_project_value(tag: &str) -> Option<&str> {
    static PROJECT_ATTRIBUTE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?is)\bProject\s*=\s*").expect("valid MSBuild Project attribute regex")
    });
    let value = &tag[PROJECT_ATTRIBUTE.find(tag)?.end()..];
    let quote = value.chars().next()?;
    let value = value.strip_prefix(quote)?;
    Some(value.split_once(quote)?.0)
}

fn central_package_import_target(
    central: &Path,
    project: &str,
    central_files: &BTreeSet<PathBuf>,
) -> Option<PathBuf> {
    let central_dir = central.parent()?;
    if project.contains("GetPathOfFileAbove")
        && project.contains("Directory.Packages.props")
        && project.contains("MSBuildThisFileDirectory")
        && project.contains("..")
    {
        return central_files
            .iter()
            .filter(|candidate| *candidate != central)
            .filter(|candidate| {
                candidate
                    .parent()
                    .is_some_and(|parent| central_dir.starts_with(parent))
            })
            .max_by_key(|candidate| {
                candidate
                    .parent()
                    .map_or(0, |parent| parent.components().count())
            })
            .cloned();
    }
    if project.contains("$(") {
        return None;
    }
    let target = super::normalize_path(&central_dir.join(project.replace('\\', "/")));
    central_files.contains(&target).then_some(target)
}

fn without_xml_ignored_regions(source: &str) -> Option<String> {
    let mut remaining = source;
    let mut result = String::with_capacity(source.len());
    while let Some((start, terminator, prefix_len)) = ignored_region_start(remaining) {
        result.push_str(&remaining[..start]);
        let after_start = &remaining[start + prefix_len..];
        let Some(end) = after_start.find(terminator) else {
            return None;
        };
        remaining = &after_start[end + terminator.len()..];
    }
    result.push_str(remaining);
    Some(result)
}

fn ignored_region_start(source: &str) -> Option<(usize, &str, usize)> {
    match (source.find("<!--"), source.find("<![CDATA[")) {
        (Some(comment), Some(cdata)) if comment < cdata => Some((comment, "-->", 4)),
        (Some(_), Some(cdata)) => Some((cdata, "]]>", 9)),
        (Some(comment), None) => Some((comment, "-->", 4)),
        (None, Some(cdata)) => Some((cdata, "]]>", 9)),
        (None, None) => None,
    }
}
