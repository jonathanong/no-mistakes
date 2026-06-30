use regex::Regex;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{msbuild_path, normalize_path, DotnetProjectFacts};

pub(in crate::codebase::dotnet) fn parse_project_static(
    project_path: &Path,
    source: &str,
) -> DotnetProjectFacts {
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
        compile_files: static_compile_includes(project_dir, source),
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

fn static_compile_includes(project_dir: &Path, source: &str) -> BTreeSet<PathBuf> {
    let re = Regex::new(r#"(?is)<Compile[^>]+Include\s*=\s*"([^"]+)""#).expect("valid regex");
    static_path_includes(project_dir, source, &re)
}

fn static_project_references(project_dir: &Path, source: &str) -> BTreeSet<PathBuf> {
    let re =
        Regex::new(r#"(?is)<ProjectReference[^>]+Include\s*=\s*"([^"]+)""#).expect("valid regex");
    static_path_includes(project_dir, source, &re)
}

fn static_path_includes(project_dir: &Path, source: &str, re: &Regex) -> BTreeSet<PathBuf> {
    re.captures_iter(source)
        .filter_map(|cap| {
            cap.get(1)
                .map(|m| normalize_path(&project_dir.join(msbuild_path(m.as_str()))))
        })
        .collect()
}

fn static_package_references(source: &str) -> BTreeSet<String> {
    let re =
        Regex::new(r#"(?is)<PackageReference[^>]+Include\s*=\s*"([^"]+)""#).expect("valid regex");
    re.captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}
