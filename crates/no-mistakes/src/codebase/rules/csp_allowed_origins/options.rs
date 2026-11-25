use super::{ParsedOrigin, RULE_ID};
use crate::codebase::rules::path_filter::RulePathFilter;
use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default)]
    csp_files: Vec<String>,
    #[serde(default)]
    self_origins: Vec<String>,
}

pub(super) struct Application<'a> {
    pub(super) csp_files: Vec<PathBuf>,
    pub(super) self_origins: Vec<ParsedOrigin>,
    pub(super) target_files: Vec<PathBuf>,
    pub(super) message: Option<&'a str>,
    pub(super) unanalyzable: Unanalyzable,
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
        let unanalyzable = config.effective_rule_analysis(rule)?.unanalyzable;
        anyhow::ensure!(
            !options.csp_files.is_empty(),
            "{RULE_ID} requires non-empty options.cspFiles naming the file(s) that declare your Content-Security-Policy"
        );
        anyhow::ensure!(
            options.csp_files.iter().all(|path| !path.trim().is_empty()),
            "{RULE_ID} options.cspFiles entries must be nonempty paths"
        );
        let mut csp_files = Vec::with_capacity(options.csp_files.len());
        let mut csp_set = BTreeSet::new();
        for raw in options.csp_files {
            let path = normalize_path(&root.join(raw));
            if csp_set.insert(path.clone()) {
                csp_files.push(path);
            }
        }
        let mut self_origins = Vec::with_capacity(options.self_origins.len());
        for raw in options.self_origins {
            let trimmed = raw.trim();
            let origin = trimmed
                .contains("://")
                .then(|| ParsedOrigin::parse(trimmed))
                .flatten()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "{RULE_ID} options.selfOrigins entry '{raw}' must be a valid origin (for example https://app.example.com)"
                    )
                })?;
            self_origins.push(origin);
        }
        let filter = RulePathFilter::new(root, config, rule)?;
        let target_files = normalized_files
            .iter()
            .filter(|path| {
                crate::codebase::dependencies::extract::is_indexable(path)
                    && !csp_set.contains(*path)
                    && filter.is_match(path)
            })
            .cloned()
            .collect();
        applications.push(Application {
            csp_files,
            self_origins,
            target_files,
            message: rule.message.as_deref(),
            unanalyzable,
        });
    }
    Ok(applications)
}
