use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

pub(super) fn selected_match(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    path: &Path,
) -> bool {
    let rel = relative_slash_path(root, path);
    rule.tests.vitest.iter().any(|name| {
        config
            .tests
            .vitest
            .projects
            .get(name)
            .is_some_and(|policy| policy_matches(policy, &rel))
    }) || rule.tests.playwright.iter().any(|name| {
        config
            .tests
            .playwright
            .projects
            .get(name)
            .is_some_and(|policy| policy_matches(policy, &rel))
    })
}

fn policy_matches(policy: &crate::config::v2::schema::TestProjectPolicy, rel: &str) -> bool {
    if policy.include.is_empty() {
        return false;
    }
    globset_matches(&policy.include, rel) && !globset_matches(&policy.exclude, rel)
}

fn globset_matches(patterns: &[String], rel: &str) -> bool {
    compile_globset(patterns).is_some_and(|globset| globset.is_match(rel))
}

fn compile_globset(patterns: &[String]) -> Option<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    let mut has_valid = false;
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
            has_valid = true;
        }
    }
    has_valid.then(|| builder.build().ok()).flatten()
}
