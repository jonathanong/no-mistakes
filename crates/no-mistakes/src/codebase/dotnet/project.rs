use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{
    msbuild_path, normalize_path, project_static::parse_project_static, DotnetConfigProject,
    DotnetProjectFacts,
};

pub(in crate::codebase::dotnet) fn parse_project(
    root: &Path,
    all_files: &[PathBuf],
    config: &DotnetConfigProject,
) -> (Option<DotnetProjectFacts>, Vec<String>) {
    let root = normalize_path(root);
    let project_path = normalize_path(&root.join(&config.project));
    let project_dir = project_path.parent().unwrap_or(&root).to_path_buf();
    let mut warnings = Vec::new();
    let source = match std::fs::read_to_string(&project_path) {
        Ok(source) => source,
        Err(error) => {
            warnings.push(format!(
                "configured dotnet project `{}` at `{}` could not be read: {error}",
                config.name,
                project_path.display()
            ));
            return (None, warnings);
        }
    };
    let evaluated = evaluate_project_with_msbuild(&project_path, &config.name)
        .map_err(|warning| warnings.push(warning))
        .ok();
    let mut facts = evaluated.unwrap_or_else(|| parse_project_static(&project_path, &source));
    finalize_project_facts(
        &mut facts,
        &root,
        all_files,
        config,
        &project_path,
        &project_dir,
    );
    (Some(facts), warnings)
}

pub(in crate::codebase::dotnet) fn finalize_project_facts(
    facts: &mut DotnetProjectFacts,
    root: &Path,
    all_files: &[PathBuf],
    config: &DotnetConfigProject,
    project_path: &Path,
    project_dir: &Path,
) {
    facts.name = config.name.clone();
    facts.project_path = project_path.to_path_buf();
    facts.project_dir = project_dir.to_path_buf();
    if config.test {
        facts.is_test = true;
    }
    if facts.assembly_name.is_empty() {
        facts.assembly_name = project_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(&config.name)
            .to_string();
    }
    if facts.root_namespace.is_empty() {
        facts.root_namespace = facts.assembly_name.clone();
    }
    if facts.compile_files.is_empty() {
        facts.compile_files = default_compile_files(all_files, project_dir);
    }
    let visible_files: BTreeSet<PathBuf> =
        all_files.iter().map(|path| normalize_path(path)).collect();
    facts
        .compile_files
        .retain(|path| path.starts_with(root) && visible_files.contains(&normalize_path(path)));
}

fn evaluate_project_with_msbuild(
    project_path: &Path,
    name: &str,
) -> Result<DotnetProjectFacts, String> {
    evaluate_project_with_program(project_path, name, "dotnet")
}

pub(in crate::codebase::dotnet) fn evaluate_project_with_program(
    project_path: &Path,
    name: &str,
    program: &str,
) -> Result<DotnetProjectFacts, String> {
    let output = std::process::Command::new(program)
        .arg("msbuild")
        .arg(project_path)
        .arg("-getProperty:AssemblyName,RootNamespace,IsTestProject,TargetFramework,TargetFrameworks")
        .arg("-getItem:Compile,ProjectReference,PackageReference")
        .output()
        .map_err(|error| format!("dotnet msbuild failed to start for `{name}`: {error}"))?;
    parse_msbuild_output(
        project_path,
        name,
        output.status.success(),
        &output.stdout,
        &output.stderr,
    )
}

pub(in crate::codebase::dotnet) fn parse_msbuild_output(
    project_path: &Path,
    name: &str,
    success: bool,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<DotnetProjectFacts, String> {
    if !success {
        return Err(format!(
            "dotnet msbuild failed for `{name}`: {}",
            String::from_utf8_lossy(stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(stdout);
    parse_msbuild_json(project_path, &stdout)
        .ok_or_else(|| format!("dotnet msbuild output was not parseable for `{name}`"))
}

pub(in crate::codebase::dotnet) fn parse_msbuild_json(
    project_path: &Path,
    output: &str,
) -> Option<DotnetProjectFacts> {
    let trimmed = output.trim();
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    let value: serde_json::Value = serde_json::from_str(&trimmed[start..=end]).ok()?;
    let properties = value.get("Properties").or_else(|| value.get("properties"));
    let items = value.get("Items").or_else(|| value.get("items"));
    let project_dir = project_path.parent().unwrap_or(Path::new("."));
    let mut facts = DotnetProjectFacts {
        project_path: project_path.to_path_buf(),
        project_dir: project_dir.to_path_buf(),
        assembly_name: property(properties, "AssemblyName").unwrap_or_default(),
        root_namespace: property(properties, "RootNamespace").unwrap_or_default(),
        is_test: property(properties, "IsTestProject")
            .is_some_and(|value| value.eq_ignore_ascii_case("true")),
        ..Default::default()
    };
    facts.compile_files = item_paths(items, "Compile", project_dir);
    facts.project_references = item_paths(items, "ProjectReference", project_dir);
    facts.package_references = item_names(items, "PackageReference");
    Some(facts)
}

fn property(properties: Option<&serde_json::Value>, name: &str) -> Option<String> {
    properties?
        .get(name)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn item_paths(
    items: Option<&serde_json::Value>,
    item_name: &str,
    project_dir: &Path,
) -> BTreeSet<PathBuf> {
    item_values(items, item_name)
        .into_iter()
        .filter_map(|value| {
            let raw = value
                .get("Identity")
                .or_else(|| value.get("identity"))
                .and_then(|value| value.as_str())?;
            Some(normalize_path(&project_dir.join(msbuild_path(raw))))
        })
        .collect()
}

fn item_names(items: Option<&serde_json::Value>, item_name: &str) -> BTreeSet<String> {
    item_values(items, item_name)
        .into_iter()
        .filter_map(|value| {
            value
                .get("Identity")
                .or_else(|| value.get("identity"))
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .collect()
}

fn item_values<'a>(
    items: Option<&'a serde_json::Value>,
    item_name: &str,
) -> Vec<&'a serde_json::Value> {
    let Some(items) = items else {
        return Vec::new();
    };
    items
        .get(item_name)
        .and_then(|value| value.as_array())
        .map(|values| values.iter().collect())
        .unwrap_or_default()
}

fn default_compile_files(all_files: &[PathBuf], project_dir: &Path) -> BTreeSet<PathBuf> {
    all_files
        .iter()
        .filter(|path| path.starts_with(project_dir))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("cs"))
        .cloned()
        .collect()
}
