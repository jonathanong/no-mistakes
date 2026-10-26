use super::policy::CooldownEntry;
use super::*;
use crate::codebase::ts_source::SourceStore;
use serde_json::Value as Json;
use serde_yaml::Value as Yaml;
use std::collections::HashSet;

const DEP_FIELDS: [&str; 4] = [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
];

pub(super) fn cooldown(
    sources: &SourceStore,
    path: &Path,
) -> Result<Option<Vec<CooldownEntry>>, String> {
    let source = sources.read_path(path).map_err(|error| error.to_string())?;
    let yaml: Yaml =
        serde_yaml::from_str(&source).map_err(|error| format!("failed to parse YAML: {error}"))?;
    let Some(updates) = yaml.get("updates").and_then(Yaml::as_sequence) else {
        return Ok(None);
    };
    let Some(npm) = updates.iter().find(|entry| {
        entry.get("package-ecosystem").and_then(Yaml::as_str) == Some("npm")
            && entry.get("directory").and_then(Yaml::as_str) == Some("/")
    }) else {
        return Ok(None);
    };
    Ok(Some(
        match npm.get("cooldown").and_then(|c| c.get("exclude")) {
            Some(Yaml::Sequence(seq)) => seq
                .iter()
                .map(|entry| match entry.as_str() {
                    Some(pattern) => CooldownEntry::Pattern(pattern.to_string()),
                    None => CooldownEntry::Other,
                })
                .collect(),
            _ => Vec::new(),
        },
    ))
}

pub(super) fn active_names(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    findings: &mut Vec<ClassifiedFinding>,
) -> HashSet<String> {
    let mut names = HashSet::new();
    for path in files {
        if path.file_name().and_then(|name| name.to_str()) != Some("package.json") {
            continue;
        }
        let json = match sources.parse_json_path(path) {
            Ok(json) => json,
            Err(error) => {
                findings.push(super::super::manifest_analysis::unavailable(
                    root, path, RULE_ID, error,
                ));
                continue;
            }
        };
        for field in DEP_FIELDS {
            let Some(deps) = json.get(field).and_then(Json::as_object) else {
                continue;
            };
            names.extend(deps.keys().cloned());
        }
    }
    names
}

pub(super) fn lockfile_keys(
    sources: &SourceStore,
    path: &Path,
) -> Result<Option<Vec<String>>, String> {
    let source = match sources.read_path(path) {
        Ok(source) => source,
        Err(error) => return Err(format!("pnpm lockfile could not be read: {error}")),
    };
    let docs = crate::codebase::lockfile::pnpm::load_documents(&source)
        .map_err(|error| format!("pnpm lockfile could not be parsed: {error:?}"))?;
    crate::codebase::lockfile::pnpm::validate_env_prefix(&docs).map_err(|_| {
        "pnpm lockfile has a non-env document before the project lockfile".to_string()
    })?;
    Ok(crate::codebase::lockfile::pnpm::package_key_strings(&docs))
}
