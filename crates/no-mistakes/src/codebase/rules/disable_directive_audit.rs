//! Aggregate-only suppression hygiene. Findings are appended after filtering.
use super::suppression::{SuppressionDirective, SuppressionDirectiveKind};
use super::{RuleFinding, SuppressedFinding};
use crate::codebase::ts_source::{
    disable_directives, relative_slash_path, DisableDirective, SourceStore,
};
use crate::config::v2::{DisableDirectivePolicy, NoMistakesConfig};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Reuses the request inventory, source store, application scopes, and consumed
/// directive provenance. Never rediscovers files or parses a TS/JS program.
#[doc(hidden)]
pub fn audit(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    suppressed: &[SuppressedFinding],
) -> Result<Vec<RuleFinding>> {
    if config.report_unused_disable_directives == DisableDirectivePolicy::Ignore {
        return Ok(Vec::new());
    }
    let mut scopes: BTreeMap<PathBuf, BTreeSet<&str>> = BTreeMap::new();
    for rule in &config.rules {
        // Audit the declared application scope; domain-specific options can
        // select inputs rather than filter source files and must not be reinterpreted.
        let mut selected_rule = rule.clone();
        selected_rule.enabled = true;
        let selected = super::selected_rule_files(root, config, &selected_rule, files)?;
        for file in selected {
            let enabled = scopes.entry(file).or_default();
            if rule.enabled && known_rule(&rule.rule) && config.rule_configured(&rule.rule) {
                enabled.insert(&rule.rule);
            }
        }
    }
    let used: BTreeSet<_> = suppressed
        .iter()
        .map(|item| {
            (
                relative_slash_path(root, Path::new(&item.source_file)),
                item.rule.as_str(),
                item.directive.clone(),
            )
        })
        .collect();
    let mut findings: Vec<_> = scopes.par_iter().flat_map_iter(|(path, enabled)| {
        let file = relative_slash_path(root, path);
        let Ok(source) = sources.read_path(path) else { return Vec::new() };
        let markdown = path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"));
        disable_directives(&source, !markdown).into_iter().filter_map(|named| {
            let directive = project(named.directive);
            if used.contains(&(file.clone(), named.rule.as_str(), directive.clone())) { return None }
            let (rule, detail) = if enabled.contains(named.rule.as_str()) {
                ("unused-disable-directive", format!("directive for `{}` suppressed no findings in this invocation", named.rule))
            } else if known_rule(&named.rule) {
                ("invalid-disable-directive", format!("directive names disabled or out-of-scope rule `{}`", named.rule))
            } else {
                ("invalid-disable-directive", format!("directive names unknown rule `{}`", named.rule))
            };
            Some(RuleFinding {
                rule: rule.to_owned(), file: file.clone(), line: directive.line,
                message: format!("{detail}; remove or correct the comment so suppressions cannot hide stale rule intent"),
                import: None, target: Some(named.rule),
            })
        }).collect::<Vec<_>>()
    }).collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn project(directive: DisableDirective) -> SuppressionDirective {
    let (kind, line) = match directive {
        DisableDirective::File { line } => (SuppressionDirectiveKind::File, line),
        DisableDirective::Line { line } => (SuppressionDirectiveKind::Line, line),
        DisableDirective::NextLine { line } => (SuppressionDirectiveKind::NextLine, line),
    };
    SuppressionDirective {
        kind,
        line: line as usize,
    }
}

fn known_rule(rule: &str) -> bool {
    use super::*;
    super::filesystem_dispatch::FILESYSTEM_RULE_IDS.contains(&rule)
        || [
            coverage_producer_assertion_owner::RULE_ID,
            FORBIDDEN_CALLS,
            FORBIDDEN_DEPENDENCIES,
            NEXTJS_NO_API_ROUTES,
            NEXTJS_NO_CACHING,
            POSTGRES_NO_MUTABLE_SQL_REGISTRY,
            POSTGRES_NO_UNBOUNDED_SQL_IDENTIFIER,
            QUERY_REACHED_PER_ITEM,
            REQUIRE_STORYBOOK_STORIES,
            REQUIRED_ENTRYPOINT_REACHABILITY,
            SERVER_ROUTE_CLIENT_BOUNDARY,
            TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
            UNCONSTRUCTED_ERROR_CLASS,
            "assert-no-fetch",
            "queues-check",
            "unique-exports",
            "playwright-coverage",
            "playwright-unique-test-ids",
            "playwright-unique-html-ids",
            "playwright-prefer-test-id-locators",
            "unused-disable-directive",
            "invalid-disable-directive",
        ]
        .contains(&rule)
}

#[cfg(test)]
mod tests;
