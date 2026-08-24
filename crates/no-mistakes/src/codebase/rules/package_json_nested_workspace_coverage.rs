use super::RuleFinding;
use crate::codebase::package_deps;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "package-json-nested-workspace-coverage";

const DEFAULT_DEPENDENCY_FIELDS: &[&str] =
    &["dependencies", "devDependencies", "optionalDependencies"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) roots: Vec<String>,
    pub(crate) dependency_name_prefixes: Vec<String>,
    pub(crate) dependency_fields: Vec<String>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts: Options = rule.rule_options()?;
            scan(root, &opts, all_files, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[derive(Clone)]
struct Manifest {
    path: PathBuf,
    dir: PathBuf,
    name: Option<String>,
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    if opts.roots.is_empty() || opts.dependency_name_prefixes.is_empty() {
        return Ok(Vec::new());
    }
    let manifests = manifests(root, files, sources);
    let root_globs = root_globs(&opts.roots)?;
    let resolved_roots: Vec<_> = manifests
        .iter()
        .filter(|manifest| root_globs.is_match(relative_slash_path(root, &manifest.dir)))
        .cloned()
        .collect();
    if resolved_roots.is_empty() {
        return Ok(Vec::new());
    }

    let by_name = manifests
        .iter()
        .filter_map(|manifest| manifest.name.as_ref().map(|name| (name.clone(), manifest)))
        .fold(
            BTreeMap::<String, Vec<&Manifest>>::new(),
            |mut map, (name, manifest)| {
                map.entry(name).or_default().push(manifest);
                map
            },
        );
    let fields = dependency_fields(opts);
    let mut findings = Vec::new();
    for manifest in resolved_roots {
        let used = manifests
            .iter()
            .filter(|candidate| candidate.dir.starts_with(&manifest.dir))
            .flat_map(|candidate| {
                matching_dependencies(candidate, &opts.dependency_name_prefixes, &fields, sources)
            })
            .collect::<BTreeSet<_>>();
        let target_dirs = used
            .iter()
            .map(
                |name| match by_name.get(name).filter(|items| items.len() == 1) {
                    Some(items) => Ok((name.clone(), items[0].dir.clone())),
                    _ => Err(name.clone()),
                },
            )
            .collect::<Result<BTreeMap<_, _>, _>>();
        let file = relative_slash_path(root, &manifest.path);
        let line = workspace_line(&manifest.path, sources);
        let target_dirs = match target_dirs {
            Ok(target_dirs) => target_dirs,
            Err(name) => {
                findings.push(finding(&file, line, format!("{file}: dependency `{name}` matches a configured prefix but has no unique visible package.json target")));
                continue;
            }
        };
        let workspaces = workspace_entries(&manifest.path, sources);
        let expected: BTreeSet<_> = target_dirs
            .values()
            .map(|dir| relative_from(&manifest.dir, dir))
            .collect();
        let declared_target_paths: BTreeSet<_> = manifests
            .iter()
            .filter(|candidate| {
                candidate.name.as_ref().is_some_and(|name| {
                    opts.dependency_name_prefixes
                        .iter()
                        .any(|prefix| name.starts_with(prefix))
                })
            })
            .map(|candidate| relative_from(&manifest.dir, &candidate.dir))
            .collect();
        let actual: BTreeSet<_> = workspaces
            .iter()
            .filter(|entry| declared_target_paths.contains(entry.as_str()))
            .cloned()
            .collect();

        let mut wildcard_finding = false;
        for entry in &workspaces {
            if contains_wildcard(entry)
                && wildcard_targets_dependency(
                    &manifest.dir,
                    entry,
                    manifests.iter().filter_map(|candidate| {
                        candidate
                            .name
                            .as_ref()
                            .filter(|name| {
                                opts.dependency_name_prefixes
                                    .iter()
                                    .any(|prefix| name.starts_with(prefix))
                            })
                            .map(|_| &candidate.dir)
                    }),
                )?
            {
                wildcard_finding = true;
                findings.push(finding(&file, line, format!("{file}: workspace entry `{entry}` uses a wildcard for a configured dependency package; use its explicit relative path")));
            }
        }
        if wildcard_finding {
            continue;
        }
        let missing: Vec<_> = expected.difference(&actual).cloned().collect();
        if !missing.is_empty() {
            findings.push(finding(
                &file,
                line,
                format!(
                    "{file}: missing nested workspace entries: {}",
                    missing.join(", ")
                ),
            ));
        }
        let extra: Vec<_> = actual.difference(&expected).cloned().collect();
        if !extra.is_empty() {
            findings.push(finding(
                &file,
                line,
                format!(
                    "{file}: unused nested workspace entries: {}",
                    extra.join(", ")
                ),
            ));
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn manifests(
    root: &Path,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<Manifest> {
    let mut manifests = files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .filter_map(|path| {
            let value = sources.parse_json_path(path).ok()?;
            let name = value
                .get("name")
                .and_then(|value| value.as_str())
                .map(str::to_string);
            Some(Manifest {
                path: path.clone(),
                dir: path.parent()?.to_path_buf(),
                name,
            })
        })
        .collect::<Vec<_>>();
    manifests.sort_by_key(|manifest| relative_slash_path(root, &manifest.path));
    manifests
}

fn root_globs(roots: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for root in roots {
        builder.add(
            GlobBuilder::new(&crate::codebase::glob_normalize::normalize(root))
                .literal_separator(true)
                .build()?,
        );
    }
    Ok(builder.build()?)
}

fn dependency_fields(opts: &Options) -> Vec<&str> {
    if opts.dependency_fields.is_empty() {
        DEFAULT_DEPENDENCY_FIELDS.to_vec()
    } else {
        opts.dependency_fields.iter().map(String::as_str).collect()
    }
}

fn matching_dependencies(
    manifest: &Manifest,
    prefixes: &[String],
    fields: &[&str],
    sources: &crate::codebase::ts_source::SourceStore,
) -> BTreeSet<String> {
    package_deps::dependency_entries_from_source_store(&manifest.path, fields, sources)
        .into_iter()
        .filter(|dep| prefixes.iter().any(|prefix| dep.name.starts_with(prefix)))
        .map(|dep| dep.name)
        .collect()
}

fn workspace_entries(
    path: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<String> {
    sources
        .parse_json_path(path)
        .ok()
        .and_then(|value| {
            value
                .get("workspaces")
                .and_then(|value| value.as_array())
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|entry| entry.as_str())
                        .map(normalize_workspace_entry)
                        .collect()
                })
        })
        .unwrap_or_default()
}

fn normalize_workspace_entry(entry: &str) -> String {
    let normalized = entry.replace('\\', "/");
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                let can_pop = parts.last().is_some_and(|parent: &&str| {
                    *parent != ".."
                        && !parent
                            .chars()
                            .any(|ch| matches!(ch, '*' | '?' | '[' | ']' | '{' | '}'))
                });
                if can_pop {
                    parts.pop();
                } else {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

fn relative_from(from: &Path, to: &Path) -> String {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    let shared = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut parts = vec![".."; from.len().saturating_sub(shared)];
    parts.extend(
        to[shared..]
            .iter()
            .filter_map(|component| component.as_os_str().to_str()),
    );
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

fn contains_wildcard(entry: &str) -> bool {
    entry.contains('*') || entry.contains('?') || entry.contains('[') || entry.contains('{')
}

fn wildcard_targets_dependency<'a>(
    root_dir: &Path,
    entry: &str,
    mut targets: impl Iterator<Item = &'a PathBuf>,
) -> Result<bool> {
    let glob = Glob::new(entry)?.compile_matcher();
    Ok(targets.any(|target| glob.is_match(relative_from(root_dir, target))))
}

fn workspace_line(path: &Path, sources: &crate::codebase::ts_source::SourceStore) -> usize {
    super::read_source(sources, path)
        .and_then(|source| {
            source
                .lines()
                .position(|line| line.contains("\"workspaces\""))
                .map(|index| index + 1)
        })
        .unwrap_or(1)
}

fn finding(file: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests;
