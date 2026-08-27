use regex::Regex;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::project_items::default_compile_files;
use super::{msbuild_path, normalize_path, DotnetProjectFacts};

pub(in crate::codebase::dotnet) fn parse_project_static(
    project_path: &Path,
    source: &str,
    all_files: &[PathBuf],
) -> DotnetProjectFacts {
    let source = strip_xml_comments(source);
    let source = source.as_ref();
    let project_dir = project_path.parent().unwrap_or_else(|| Path::new("."));
    let assembly_name = xml_tag(source, "AssemblyName").unwrap_or_else(|| {
        project_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Project")
            .to_string()
    });
    DotnetProjectFacts {
        project_path: project_path.to_path_buf(),
        project_dir: project_dir.to_path_buf(),
        assembly_name: assembly_name.clone(),
        root_namespace: xml_tag(source, "RootNamespace").unwrap_or(assembly_name),
        is_test: is_test_project(source),
        compile_files: static_compile_files(project_dir, source, all_files),
        project_references: static_project_references(project_dir, source),
        package_references: static_package_references(source),
        ..Default::default()
    }
}

fn xml_tag(source: &str, tag: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?is)<{tag}>\s*([^<]+?)\s*</{tag}>")).ok()?;
    re.captures(source)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

fn is_test_project(source: &str) -> bool {
    xml_tag(source, "IsTestProject").is_some_and(|value| value.eq_ignore_ascii_case("true"))
        || source.contains("xunit")
        || source.contains("Microsoft.NET.Test.Sdk")
}

fn static_compile_files(
    project_dir: &Path,
    source: &str,
    all_files: &[PathBuf],
) -> BTreeSet<PathBuf> {
    let mut files = if default_compile_items_enabled(source) {
        default_compile_files(all_files, project_dir)
    } else {
        BTreeSet::new()
    };
    for (operation, path) in static_compile_operations(project_dir, source) {
        match operation {
            StaticCompileOperation::Include => {
                files.insert(path);
            }
            StaticCompileOperation::Remove => {
                files.remove(&path);
            }
        }
    }
    files
}

#[derive(Clone, Copy)]
enum StaticCompileOperation {
    Include,
    Remove,
}

fn static_compile_operations(
    project_dir: &Path,
    source: &str,
) -> Vec<(StaticCompileOperation, PathBuf)> {
    let re = Regex::new(r#"(?is)<Compile\b[^>]*?\b(Include|Remove)\s*=\s*["']([^"']+)["'][^>]*>"#)
        .expect("valid static Compile operation regex");
    let condition_re =
        Regex::new(r#"(?is)\bCondition\s*="#).expect("valid MSBuild condition regex");
    re.captures_iter(source)
        .flat_map(|captures| {
            let operation = if captures[1].eq_ignore_ascii_case("Include") {
                StaticCompileOperation::Include
            } else {
                StaticCompileOperation::Remove
            };
            if matches!(operation, StaticCompileOperation::Remove)
                && condition_re.is_match(&captures[0])
            {
                return Vec::new();
            }
            captures[2]
                .split(';')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| {
                    (
                        operation,
                        normalize_path(&project_dir.join(msbuild_path(value))),
                    )
                })
                .collect()
        })
        .collect()
}

fn default_compile_items_enabled(source: &str) -> bool {
    if !is_sdk_project(source) {
        return false;
    }
    ["EnableDefaultItems", "EnableDefaultCompileItems"]
        .into_iter()
        .all(|tag| {
            xml_tag_last(source, tag).is_none_or(|value| !value.eq_ignore_ascii_case("false"))
        })
}

fn is_sdk_project(source: &str) -> bool {
    static_item_regex("Project", "Sdk").is_match(source)
        || static_item_regex("Sdk", "Name").is_match(source)
        || static_item_regex("Import", "Sdk").is_match(source)
}

fn strip_xml_comments(source: &str) -> Cow<'_, str> {
    Regex::new(r"(?s)<!--.*?-->")
        .expect("valid XML comment regex")
        .replace_all(source, "")
}

fn xml_tag_last(source: &str, tag: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?is)<{tag}>\s*([^<]+?)\s*</{tag}>")).ok()?;
    re.captures_iter(source)
        .last()
        .and_then(|cap| cap.get(1))
        .map(|value| value.as_str().trim().to_string())
}

fn static_project_references(project_dir: &Path, source: &str) -> BTreeSet<PathBuf> {
    static_path_includes(project_dir, source, "ProjectReference")
}

fn static_path_includes(project_dir: &Path, source: &str, element: &str) -> BTreeSet<PathBuf> {
    static_path_items(project_dir, source, element, "Include")
}

fn static_path_items(
    project_dir: &Path,
    source: &str,
    element: &str,
    attribute: &str,
) -> BTreeSet<PathBuf> {
    static_item_values(source, element, attribute)
        .into_iter()
        .map(|value| normalize_path(&project_dir.join(msbuild_path(&value))))
        .collect()
}

fn static_package_references(source: &str) -> BTreeSet<String> {
    static_item_values(source, "PackageReference", "Include")
}

fn static_item_values(source: &str, element: &str, attribute: &str) -> BTreeSet<String> {
    let re = static_item_regex(element, attribute);
    re.captures_iter(source)
        .flat_map(|cap| {
            cap.get(1)
                .map(|value| {
                    value
                        .as_str()
                        .split(';')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect()
}

fn static_item_regex(element: &str, attribute: &str) -> Regex {
    Regex::new(&format!(
        r#"(?is)<{element}\b[^>]*\b{attribute}\s*=\s*["']([^"']+)["']"#
    ))
    .expect("valid static MSBuild item regex")
}
