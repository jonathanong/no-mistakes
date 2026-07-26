//! Enforces the deliberately small documentation discovery graph used by agents.
use super::RuleFinding;
use crate::codebase::md_links;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use pulldown_cmark::{Event, Options as MarkdownOptions, Parser, Tag};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Component, Path, PathBuf};

pub const RULE_ID: &str = "markdown-reachability";
const DEFAULT_ROOT_FILENAMES: &[&str] = &["CLAUDE.md"];
const DEFAULT_INDEX_FILENAMES: &[&str] = &["README.md"];
const DEFAULT_MAX_DEPTH: usize = 2;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    root_filenames: Option<Vec<String>>,
    index_filenames: Option<Vec<String>>,
    max_depth: Option<usize>,
    baseline_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct BaselineEntry {
    state: String,
    #[serde(default)]
    depth: Option<usize>,
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let markdown = markdown_files(root, all_files);
    let graph = link_graph(root, &markdown, sources);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.rule_options();
        let roots = filenames(&options.root_filenames, DEFAULT_ROOT_FILENAMES);
        let indexes = filenames(&options.index_filenames, DEFAULT_INDEX_FILENAMES);
        let max_depth = validate_max_depth(options.max_depth)?;
        let target_paths = super::path_filter::filter_rule_files(root, config, rule, &markdown)?;
        let target_names = target_paths
            .iter()
            .filter(|path| !is_named(path, &roots))
            .map(|path| relative_slash_path(root, path))
            .collect::<BTreeSet<_>>();
        let states = target_paths
            .iter()
            .filter(|path| !is_named(path, &roots))
            .map(|path| {
                let depth = shortest_depth(path, &roots, &graph);
                let allowed = direct_or_readme_hop(path, &roots, &indexes, &graph, max_depth);
                (relative_slash_path(root, path), (depth, allowed))
            })
            .collect::<BTreeMap<_, _>>();
        let baseline = read_baseline(root, options.baseline_file.as_deref(), all_files)?;
        for (file, (depth, allowed)) in states {
            let expected = match (depth, allowed) {
                (_, true) => None,
                (Some(depth), false) => Some(BaselineEntry {
                    state: "depth".to_string(),
                    depth: Some(depth),
                }),
                (None, false) => Some(BaselineEntry {
                    state: "unreachable".to_string(),
                    depth: None,
                }),
            };
            match (expected, baseline.get(&file)) {
                (None, Some(_)) => {
                    findings.push(stale(&file, "is reachable; remove its baseline entry"))
                }
                (None, None) => {}
                (Some(expected), Some(actual)) if *actual == expected => {}
                (Some(expected), Some(_)) => findings.push(stale(
                    &file,
                    &format!("baseline does not match current {}", expected.state),
                )),
                (Some(expected), None) => findings.push(finding(&file, &expected, max_depth)),
            }
        }
        for file in baseline.keys() {
            if !target_names.contains(file) {
                findings.push(stale(
                    file,
                    "references a deleted or excluded Markdown file",
                ));
            }
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn validate_max_depth(configured: Option<usize>) -> Result<usize> {
    let depth = configured.unwrap_or(DEFAULT_MAX_DEPTH);
    if !(1..=2).contains(&depth) {
        anyhow::bail!("{RULE_ID} options.maxDepth must be 1 or 2; README-only discovery supports no deeper graph")
    }
    Ok(depth)
}

fn markdown_files(root: &Path, files: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = files
        .iter()
        .filter(|path| path.starts_with(root) && path.extension().is_some_and(|ext| ext == "md"))
        .cloned()
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}

fn filenames(configured: &Option<Vec<String>>, defaults: &[&str]) -> BTreeSet<String> {
    configured
        .as_ref()
        .map(|items| items.iter().cloned().collect())
        .unwrap_or_else(|| defaults.iter().map(|item| (*item).to_string()).collect())
}

fn is_named(path: &Path, names: &BTreeSet<String>) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| names.contains(name))
}

fn link_graph(
    root: &Path,
    markdown: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let known = markdown.iter().cloned().collect::<BTreeSet<_>>();
    markdown
        .iter()
        .map(|path| {
            let links = super::read_source(sources, path)
                .map(|source| extract_local_links(root, path, &source, &known))
                .unwrap_or_default();
            (path.clone(), links)
        })
        .collect()
}

fn extract_local_links(
    root: &Path,
    source: &Path,
    content: &str,
    known: &BTreeSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    for event in Parser::new_ext(content, MarkdownOptions::all()) {
        let Event::Start(Tag::Link { dest_url, .. }) = event else {
            continue;
        };
        let destination = dest_url
            .as_ref()
            .split(['#', '?'])
            .next()
            .unwrap_or_default();
        if destination.is_empty()
            || md_links::is_external(destination)
            || !destination.ends_with(".md")
        {
            continue;
        }
        let Some(destination) = md_links::decode_local_path(destination) else {
            continue;
        };
        let base = if destination.starts_with('/') {
            root.to_path_buf()
        } else {
            source.parent().unwrap_or(root).to_path_buf()
        };
        if let Some(path) = normalize_inside(root, &base.join(destination.trim_start_matches('/')))
        {
            if known.contains(&path) {
                paths.insert(path);
            }
        }
    }
    paths.into_iter().collect()
}

fn normalize_inside(root: &Path, path: &Path) -> Option<PathBuf> {
    let mut relative = PathBuf::new();
    for component in path.strip_prefix(root).ok()?.components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !relative.pop() {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(root.join(relative))
}

fn direct_or_readme_hop(
    target: &Path,
    roots: &BTreeSet<String>,
    indexes: &BTreeSet<String>,
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    max_depth: usize,
) -> bool {
    for root in graph.keys().filter(|path| is_named(path, roots)) {
        if graph
            .get(root)
            .is_some_and(|links| links.contains(&target.to_path_buf()))
        {
            return true;
        }
        if max_depth >= 2 {
            for index in graph
                .get(root)
                .into_iter()
                .flatten()
                .filter(|path| is_named(path, indexes))
            {
                if graph
                    .get(index)
                    .is_some_and(|links| links.contains(&target.to_path_buf()))
                {
                    return true;
                }
            }
        }
    }
    false
}

fn shortest_depth(
    target: &Path,
    roots: &BTreeSet<String>,
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
) -> Option<usize> {
    let mut queue = graph
        .keys()
        .filter(|path| is_named(path, roots))
        .cloned()
        .map(|path| (path, 0usize))
        .collect::<VecDeque<_>>();
    let mut seen = BTreeSet::new();
    while let Some((current, depth)) = queue.pop_front() {
        if !seen.insert(current.clone()) {
            continue;
        }
        if current == target {
            return Some(depth);
        }
        for next in graph.get(&current).into_iter().flatten() {
            queue.push_back((next.clone(), depth + 1));
        }
    }
    None
}

fn read_baseline(
    root: &Path,
    path: Option<&Path>,
    tracked_files: &[PathBuf],
) -> Result<BTreeMap<String, BaselineEntry>> {
    let Some(path) = path else {
        return Ok(BTreeMap::new());
    };
    let baseline_path = crate::codebase::ts_resolver::normalize_path(&root.join(path));
    let mut baseline_is_tracked = false;
    for file in tracked_files {
        if crate::codebase::ts_resolver::normalize_path(file) == baseline_path {
            baseline_is_tracked = true;
            break;
        }
    }
    if !baseline_is_tracked {
        anyhow::bail!(
            "{RULE_ID} options.baselineFile must reference a tracked repository file: {}",
            path.display()
        )
    }
    let content = std::fs::read_to_string(&baseline_path)
        .context(format!("read {RULE_ID} baseline {}", path.display()))?;
    serde_json::from_str(&content).context("parse markdown-reachability baseline JSON")
}

fn finding(file: &str, state: &BaselineEntry, max_depth: usize) -> RuleFinding {
    let message = if state.state == "unreachable" {
        format!("not reachable from a configured root Markdown file within {max_depth} hops")
    } else {
        format!(
            "reachable only at depth {}; maximum is {max_depth}",
            state.depth.unwrap_or_default()
        )
    };
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: None,
    }
}

fn stale(file: &str, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: format!("stale baseline entry: {message}"),
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::v2::{
        schema::{RuleDef, RuleScope},
        NoMistakesConfig,
    };

    fn config(options: &str, include: &[&str], exclude: &[&str]) -> NoMistakesConfig {
        NoMistakesConfig {
            rules: vec![RuleDef {
                rule: RULE_ID.to_string(),
                scope: Some(RuleScope::Repository),
                include: include.iter().map(|item| (*item).to_string()).collect(),
                exclude: exclude.iter().map(|item| (*item).to_string()).collect(),
                options: serde_yaml::from_str(options).unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn run(
        root: &Path,
        config: &NoMistakesConfig,
        relative_files: &[&str],
    ) -> Result<Vec<RuleFinding>> {
        let files = relative_files
            .iter()
            .map(|file| root.join(file))
            .collect::<Vec<_>>();
        let sources = super::super::source_store_for_files(&files);
        check_with_files_and_sources(root, config, &files, &sources)
    }

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-reachability")
            .join(name)
    }
    #[test]
    fn resolves_only_links_inside_root() {
        let root = Path::new("/repo");
        assert_eq!(
            normalize_inside(root, &root.join("a/../b.md")),
            Some(root.join("b.md"))
        );
        assert_eq!(normalize_inside(root, &root.join("../../b.md")), None);
    }

    #[test]
    fn accepts_only_supported_depths() {
        assert_eq!(validate_max_depth(None).unwrap(), 2);
        assert_eq!(validate_max_depth(Some(1)).unwrap(), 1);
        assert!(validate_max_depth(Some(0)).is_err());
        assert!(validate_max_depth(Some(3)).is_err());
    }

    #[test]
    fn full_check_accepts_direct_and_readme_paths_and_rejects_other_paths() {
        let root = fixture("paths");
        let files = [
            "CLAUDE.md",
            "README.md",
            "other.md",
            "direct.md",
            "indexed.md",
            "arbitrary.md",
            "lost.md",
        ];
        let findings = run(&root, &config("", &["**/*.md"], &[]), &files).unwrap();
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.file.as_str())
                .collect::<Vec<_>>(),
            ["arbitrary.md", "lost.md"]
        );
        assert!(findings[0].message.contains("depth 2"));
        assert!(findings[1].message.contains("not reachable"));
    }

    #[test]
    fn recognizes_reference_links_and_ignores_non_local_or_escaping_links() {
        let root = fixture("links");
        let findings = run(
            &root,
            &config("", &["**/*.md"], &[]),
            &[
                "CLAUDE.md",
                "docs/doc.md",
                "docs/My Guide.md",
                "docs/unlinked.md",
                "docs/blocked.md",
            ],
        )
        .unwrap();
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.file.as_str())
                .collect::<Vec<_>>(),
            ["docs/blocked.md", "docs/unlinked.md"]
        );
    }

    #[test]
    fn baseline_requires_exact_state_and_filtered_targets_make_entries_stale() {
        let root = fixture("baseline-match");
        let options = "baselineFile: baseline.json";
        let files = ["CLAUDE.md", "other.md", "deep.md", "baseline.json"];
        let findings = run(&root, &config(options, &["**/*.md"], &[]), &files).unwrap();
        assert!(findings.is_empty(), "{findings:#?}");
        let filtered = run(&root, &config(options, &["**/*.md"], &["deep.md"]), &files).unwrap();
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].message.contains("deleted or excluded"));
    }

    #[test]
    fn baseline_entry_for_a_configured_root_is_stale() {
        let root = fixture("baseline-root");
        let findings = run(
            &root,
            &config("baselineFile: baseline.json", &["**/*.md"], &[]),
            &["CLAUDE.md", "baseline.json"],
        )
        .unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("deleted or excluded"));
    }

    #[test]
    fn mismatched_baseline_state_is_stale() {
        let root = fixture("baseline-mismatched");
        let findings = run(
            &root,
            &config("baselineFile: baseline.json", &["**/*.md"], &[]),
            &["CLAUDE.md", "other.md", "deep.md", "baseline.json"],
        )
        .unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("does not match"));
    }

    #[test]
    fn baseline_must_be_tracked_and_valid_json() {
        let root = fixture("baseline-invalid");
        let config = config("baselineFile: baseline.json", &["**/*.md"], &[]);
        assert!(run(&root, &config, &["CLAUDE.md", "baseline.json"]).is_err());
        assert!(run(&root, &config, &["CLAUDE.md"])
            .unwrap_err()
            .to_string()
            .contains("tracked"));
    }

    #[test]
    fn accepts_a_tracked_baseline_outside_the_rule_project_root() {
        let root = fixture("scoped");
        let mut config = config("baselineFile: baselines/reachability.json", &[], &[]);
        config.projects.insert(
            "docs".to_string(),
            crate::config::v2::schema::Project {
                root: Some("docs".to_string()),
                ..Default::default()
            },
        );
        config.rules[0].scope = None;
        config.rules[0].projects = vec!["docs".to_string()];
        let findings = run(
            &root,
            &config,
            &[
                "docs/CLAUDE.md",
                "docs/other.md",
                "docs/deep.md",
                "baselines/reachability.json",
            ],
        )
        .unwrap();
        assert!(findings.is_empty(), "{findings:#?}");
    }

    #[test]
    fn dispatcher_applies_standard_file_suppression() {
        let root = fixture("suppression");
        let config_path = root.join(".no-mistakes.yml");
        let findings = crate::codebase::rules::run_filesystem_rules_with_files(
            &root,
            Some(&config_path),
            &[root.join("CLAUDE.md"), root.join("lost.md")],
        )
        .unwrap();
        assert!(findings.is_empty(), "{findings:#?}");
    }
}
