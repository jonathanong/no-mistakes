use super::RULE_ID;
use crate::codebase::rules::path_filter::{GlobMatcher, RulePathFilter};
use crate::codebase::rules::validate_options::finish;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod validate;
use validate::{collect_option_errors, invalid};

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

pub(crate) fn collect_config_errors(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    errors: &mut Vec<String>,
) {
    let root = normalize_path(root);
    for (index, rule) in
        crate::codebase::rules::validate_options::indexed_rule_applications(config, RULE_ID)
    {
        let app_errors = collect_application_errors(&root, config, rule)
            .unwrap_or_else(|error| BTreeSet::from([format!("{error:#}")]));
        crate::codebase::rules::validate_options::push_application_errors(
            &root,
            config,
            index,
            config_path,
            app_errors,
            errors,
        );
    }
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
    let root = normalize_path(root);
    let mut errors = Vec::new();
    collect_config_errors(&root, None, config, &mut errors);
    finish(errors)?;
    let normalized_files: Vec<PathBuf> = files
        .iter()
        .map(|path| normalize_path(path))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut apps = Vec::new();
    for rule in rules {
        select_application(&root, config, rule, &normalized_files, &mut apps)?;
    }
    Ok(apps)
}

pub(crate) fn collect_application_errors(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
) -> Result<BTreeSet<String>> {
    let options: Options = rule.strict_rule_options()?;
    let mut errors = BTreeSet::new();
    collect_option_errors(&options, &mut errors);
    let filter = match RulePathFilter::new(root, config, rule) {
        Ok(filter) => filter,
        Err(error) => {
            errors.insert(format!("{error:#}"));
            return Ok(errors);
        }
    };
    let configured = options
        .config
        .as_deref()
        .filter(|path| !options.required_globs.is_empty() && !path.trim().is_empty());
    if let Some(configured) = configured {
        let path = normalize_path(&root.join(configured));
        if filter.is_match(&path) && !crate::codebase::dependencies::extract::is_indexable(&path) {
            errors.insert(invalid(
                "config",
                configured,
                "a TypeScript or JavaScript file path",
            ));
        }
    }
    Ok(errors)
}

fn select_application<'a>(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &'a RuleDef,
    files: &[PathBuf],
    apps: &mut Vec<Application<'a>>,
) -> Result<()> {
    let options: Options = rule.try_rule_options()?;
    if options.required_globs.is_empty() {
        return Ok(());
    }
    let file_matcher = GlobMatcher::new(&options.files, RULE_ID)?;
    let filter = RulePathFilter::new(root, config, rule)?;
    let config_file = options
        .config
        .map(|config_path| normalize_path(&root.join(&config_path)))
        .filter(|path| filter.is_match(path));
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
            apps.push(Application {
                path: path.clone(),
                mode: CheckMode::TestLiterals,
                required_globs: required_globs.clone(),
                message: rule.message.as_deref(),
            });
        }
    }
    if let Some(path) = config_file.filter(|path| !matched_files.contains(path)) {
        apps.push(Application {
            path,
            mode: CheckMode::DefaultExport,
            required_globs,
            message: rule.message.as_deref(),
        });
    }
    Ok(())
}
