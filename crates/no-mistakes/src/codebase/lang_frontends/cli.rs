use super::{LangFactMap, LangFrontendConfig};
use crate::config::v2::NoMistakesConfig;
use globset::Glob;
use std::collections::HashMap;
use std::path::Path;

pub(crate) fn lang_config_from_v2(config: &NoMistakesConfig) -> LangFrontendConfig {
    LangFrontendConfig {
        python_packages: config.tests.python.packages.clone(),
        go_modules: config.tests.go.modules.clone(),
        rust_packages: config.tests.rust.packages.clone(),
        rails_apps: config.tests.rails.apps.clone(),
        php_apps: config.tests.php.apps.clone(),
        php_framework: config.tests.php.framework.clone(),
    }
}

pub(crate) fn lang_config_is_empty(config: &LangFrontendConfig) -> bool {
    config.python_packages.is_empty()
        && config.go_modules.is_empty()
        && config.rust_packages.is_empty()
        && config.rails_apps.is_empty()
        && config.php_apps.is_empty()
}

pub(crate) struct QueueGlobMatchers {
    pub enqueues: Vec<(globset::GlobMatcher, String)>,
    pub workers: Vec<(globset::GlobMatcher, String)>,
    pub clusters: HashMap<String, Option<String>>,
    pub default_cluster: Option<String>,
}

pub(crate) fn queue_globs_from_v2(config: &NoMistakesConfig) -> QueueGlobMatchers {
    let mut enqueues = Vec::new();
    let mut workers = Vec::new();
    let mut clusters = HashMap::new();
    let default_cluster = config
        .projects
        .values()
        .find_map(|project| project.queues.cluster.clone());
    for project in config.projects.values() {
        let cluster = project.queues.cluster.clone();
        for glob in prefixed_globs(project.root.as_deref(), &project.queues.enqueues) {
            clusters
                .entry(glob.clone())
                .or_insert_with(|| cluster.clone());
            if let Ok(compiled) = Glob::new(&glob) {
                enqueues.push((compiled.compile_matcher(), glob));
            }
        }
        for glob in prefixed_globs(project.root.as_deref(), &project.queues.workers) {
            clusters
                .entry(glob.clone())
                .or_insert_with(|| cluster.clone());
            if let Ok(compiled) = Glob::new(&glob) {
                workers.push((compiled.compile_matcher(), glob));
            }
        }
    }
    QueueGlobMatchers {
        enqueues,
        workers,
        clusters,
        default_cluster,
    }
}

pub(crate) fn matching_cluster(
    root: &Path,
    path: &Path,
    matchers: &[(globset::GlobMatcher, String)],
    globs: &QueueGlobMatchers,
) -> Option<Option<String>> {
    if matchers.is_empty() {
        return None;
    }
    let rel = path.strip_prefix(root).unwrap_or(path);
    matchers.iter().find_map(|(matcher, glob)| {
        matcher
            .is_match(rel)
            .then(|| match globs.clusters.get(glob) {
                Some(cluster) => cluster.clone(),
                None => globs.default_cluster.clone(),
            })
    })
}

pub(crate) fn each_lang_map(facts: &super::CollectedLangFacts) -> [&LangFactMap; 5] {
    [
        &facts.python,
        &facts.go,
        &facts.rust,
        &facts.ruby,
        &facts.php,
    ]
}

fn prefixed_globs(root: Option<&str>, globs: &[String]) -> Vec<String> {
    let prefix = root
        .map(str::trim)
        .filter(|root| !root.is_empty() && *root != ".");
    globs
        .iter()
        .map(|glob| match prefix {
            Some(root) if glob == root || glob.starts_with(&format!("{root}/")) => glob.clone(),
            Some(root) => format!("{}/{glob}", root.trim_end_matches('/')),
            None => glob.clone(),
        })
        .collect()
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
