use super::RULE_ID;
use crate::codebase::rules::path_filter::{GlobMatcher, RulePathFilter};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const DEFAULT_GLOBS: [&str; 2] = ["**/node_modules/**", "**/.git/**"];

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default)]
    config: Option<String>,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default = "default_globs")]
    required_globs: Vec<String>,
}

fn default_globs() -> Vec<String> {
    DEFAULT_GLOBS.into_iter().map(str::to_owned).collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CheckMode {
    DefaultExport,
    TestLiterals,
}

pub(super) struct Application<'a> {
    pub(super) path: PathBuf,
    pub(super) mode: CheckMode,
    pub(super) required_globs: Vec<String>,
    pub(super) message: Option<&'a str>,
}

pub(super) fn applications<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<Application<'a>>> {
    let rules = config.rule_applications(RULE_ID);
    if rules.is_empty() {
        return Ok(Vec::new());
    }
    let normalized_root = normalize_path(root);
    let root = normalized_root.as_path();
    let normalized_files: Vec<PathBuf> = files
        .iter()
        .map(|path| normalize_path(path))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut applications = Vec::new();
    for rule in rules {
        let options: Options = rule.try_rule_options()?;
        if let Some(config_path) = &options.config {
            anyhow::ensure!(
                !config_path.trim().is_empty(),
                "{RULE_ID} requires options.config naming an explicit Vitest config file"
            );
        }
        anyhow::ensure!(
            options.config.is_some() || !options.files.is_empty(),
            "{RULE_ID} requires options.config or non-empty options.files"
        );
        anyhow::ensure!(
            options.files.iter().all(|glob| !glob.trim().is_empty()),
            "{RULE_ID} files entries must be nonempty globs"
        );
        let file_matcher = GlobMatcher::new(&options.files, RULE_ID)?;
        anyhow::ensure!(
            options
                .required_globs
                .iter()
                .all(|glob| !glob.trim().is_empty()),
            "{RULE_ID} requiredGlobs entries must be nonempty strings"
        );
        if options.required_globs.is_empty() {
            continue;
        }
        let filter = RulePathFilter::new(root, config, rule)?;
        let mut seen = FxHashSet::default();
        let required_globs: Vec<String> = options
            .required_globs
            .into_iter()
            .filter(|glob| seen.insert(glob.clone()))
            .collect();
        let mut matched_files = BTreeSet::new();
        if !file_matcher.is_empty() {
            for path in normalized_files.iter().filter(|path| {
                crate::codebase::dependencies::extract::is_indexable(path)
                    && filter.is_match(path)
                    && file_matcher.is_match(&relative_slash_path(root, path))
            }) {
                matched_files.insert(path.clone());
                applications.push(Application {
                    path: path.clone(),
                    mode: CheckMode::TestLiterals,
                    required_globs: required_globs.clone(),
                    message: rule.message.as_deref(),
                });
            }
        }
        if let Some(config_path) = options.config {
            let path = normalize_path(&root.join(config_path));
            if filter.is_match(&path) {
                anyhow::ensure!(
                    crate::codebase::dependencies::extract::is_indexable(&path),
                    "{RULE_ID} options.config must name a TypeScript or JavaScript file: {}",
                    path.display()
                );
                if !matched_files.contains(&path) {
                    applications.push(Application {
                        path,
                        mode: CheckMode::DefaultExport,
                        required_globs,
                        message: rule.message.as_deref(),
                    });
                }
            }
        }
    }
    Ok(applications)
}
