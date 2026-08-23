use super::lockfile::package_name_from_lock_key;
use super::policy::{check, CooldownEntry, ExcludeEntry, FileKind, Issue, Snapshot};
use super::{rel, Options, RuleFinding, RULE_ID};
use crate::codebase::ts_source::SourceStore;
use serde_json::Value as Json;
use serde_yaml::Value as Yaml;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const DEP_FIELDS: [&str; 4] = [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
];

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let by_rel: HashMap<String, &PathBuf> =
        files.iter().map(|path| (rel(root, path), path)).collect();
    let Some(workspace) = by_rel.get(opts.workspace_yaml()) else {
        return Vec::new();
    };
    let Ok(source) = sources.read_path(workspace) else {
        return Vec::new();
    };
    let yaml = match serde_yaml::from_str::<Yaml>(&source) {
        Ok(Yaml::Mapping(map)) => Yaml::Mapping(map),
        Ok(_) => return Vec::new(),
        Err(error) => {
            return vec![finding(
                rel(root, workspace),
                format!("failed to parse YAML: {error}"),
                opts.workspace_yaml(),
            )];
        }
    };
    let lockfile_keys = by_rel
        .get(opts.lockfile_path())
        .and_then(|path| lockfile_keys(sources, path));
    let mut active_names = active_names(files, sources);
    if let Some(keys) = &lockfile_keys {
        for key in keys {
            if let Some(name) = package_name_from_lock_key(key) {
                active_names.insert(name);
            }
        }
    }
    let snapshot = Snapshot {
        exclude: exclude_entries(&yaml),
        cooldown: by_rel
            .get(opts.dependabot_path())
            .and_then(|path| cooldown(sources, path)),
        active_names,
        lockfile_keys,
    };
    check(opts, &snapshot)
        .into_iter()
        .map(|issue| issue_finding(root, opts, &by_rel, issue))
        .collect()
}

fn exclude_entries(yaml: &Yaml) -> Vec<ExcludeEntry> {
    match yaml.get("minimumReleaseAgeExclude") {
        Some(Yaml::Sequence(seq)) => seq
            .iter()
            .map(|entry| match entry.as_str() {
                Some(name) => ExcludeEntry::Name(name.to_string()),
                None => ExcludeEntry::Other,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn cooldown(sources: &SourceStore, path: &Path) -> Option<Vec<CooldownEntry>> {
    let source = sources.read_path(path).ok()?;
    let yaml: Yaml = serde_yaml::from_str(&source).ok()?;
    let updates = yaml.get("updates")?.as_sequence()?;
    let npm = updates.iter().find(|entry| {
        entry.get("package-ecosystem").and_then(Yaml::as_str) == Some("npm")
            && entry.get("directory").and_then(Yaml::as_str) == Some("/")
    })?;
    Some(match npm.get("cooldown").and_then(|c| c.get("exclude")) {
        Some(Yaml::Sequence(seq)) => seq
            .iter()
            .map(|entry| match entry.as_str() {
                Some(pattern) => CooldownEntry::Pattern(pattern.to_string()),
                None => CooldownEntry::Other,
            })
            .collect(),
        _ => Vec::new(),
    })
}

fn active_names(files: &[PathBuf], sources: &SourceStore) -> HashSet<String> {
    let mut names = HashSet::new();
    for path in files {
        if path.file_name().and_then(|name| name.to_str()) != Some("package.json") {
            continue;
        }
        let Ok(json) = sources.parse_json_path(path) else {
            continue;
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

fn lockfile_keys(sources: &SourceStore, path: &Path) -> Option<Vec<String>> {
    let source = sources.read_path(path).ok()?;
    let yaml: Yaml = serde_yaml::from_str(&source).ok()?;
    let packages = yaml.get("packages")?.as_mapping()?;
    let mut keys = Vec::new();
    for key in packages.keys() {
        if let Some(key) = key.as_str() {
            keys.push(key.to_string());
        }
    }
    Some(keys)
}

fn issue_finding(
    root: &Path,
    opts: &Options,
    by_rel: &HashMap<String, &PathBuf>,
    issue: Issue,
) -> RuleFinding {
    let rel_path = match issue.file {
        FileKind::Workspace => opts.workspace_yaml().to_string(),
        FileKind::Dependabot => opts.dependabot_path().to_string(),
        FileKind::Lockfile => opts.lockfile_path().to_string(),
    };
    let file = by_rel
        .get(&rel_path)
        .map(|path| rel(root, path))
        .unwrap_or(rel_path);
    finding(
        file.clone(),
        format!("{file}: {}", issue.message),
        &issue.target,
    )
}

fn finding(file: String, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file,
        line: 1,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
