use super::RULE_ID;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::rules::path_filter::{GlobMatcher, RulePathFilter};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::RuleDef;
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
    session: Option<&AnalysisSession>,
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
        let result = select_application(root, config, rule, &normalized_files, &mut applications);
        if let Some(session) = session {
            session.capture_check_result(RULE_ID, result)?;
        } else {
            result?;
        }
    }
    Ok(applications)
}

fn select_application<'a>(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &'a RuleDef,
    files: &[PathBuf],
    applications: &mut Vec<Application<'a>>,
) -> Result<()> {
    let options: Options = rule.try_rule_options()?;
    let file_matcher = options.validate()?;
    if options.required_globs.is_empty() {
        return Ok(());
    }
    let filter = RulePathFilter::new(root, config, rule)?;
    let config_file = options
        .config
        .map(|config_path| (normalize_path(&root.join(&config_path)), config_path))
        .filter(|(path, _)| filter.is_match(path));
    if let Some((path, configured)) = &config_file {
        anyhow::ensure!(
            crate::codebase::dependencies::extract::is_indexable(path),
            invalid("config", configured, "a TypeScript or JavaScript file path")
        );
    }
    let mut seen = FxHashSet::default();
    let required_globs: Vec<String> = options
        .required_globs
        .into_iter()
        .filter(|glob| seen.insert(glob.clone()))
        .collect();
    let mut matched_files = BTreeSet::new();
    if !file_matcher.is_empty() {
        for path in files.iter().filter(|path| {
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
    if let Some((path, _)) = config_file {
        if !matched_files.contains(&path) {
            applications.push(Application {
                path,
                mode: CheckMode::DefaultExport,
                required_globs,
                message: rule.message.as_deref(),
            });
        }
    }
    Ok(())
}

impl Options {
    fn validate(&self) -> Result<GlobMatcher> {
        if let Some(config_path) = &self.config {
            anyhow::ensure!(
                !config_path.trim().is_empty(),
                invalid(
                    "config",
                    config_path,
                    "an explicit non-empty Vitest config file path"
                )
            );
        }
        anyhow::ensure!(
            self.config.is_some() || !self.files.is_empty(),
            invalid(
                "files",
                &self.files,
                "a non-empty list of globs when options.config is absent"
            )
        );
        for glob in &self.files {
            anyhow::ensure!(
                !glob.trim().is_empty(),
                invalid("files", glob, "a valid non-empty glob")
            );
        }
        let matcher = GlobMatcher::new(&self.files, RULE_ID).map_err(|error| {
            error.context(invalid(
                "files",
                &self.files,
                "a list of valid non-empty globs",
            ))
        })?;
        for glob in &self.required_globs {
            anyhow::ensure!(
                !glob.trim().is_empty(),
                invalid("requiredGlobs", glob, "a non-empty glob string")
            );
        }
        Ok(matcher)
    }
}

fn invalid(option: &str, value: impl std::fmt::Debug, allowed: &str) -> CheckInputFailure {
    CheckInputFailure::invalid_rule_option(RULE_ID, option, value, allowed)
}
