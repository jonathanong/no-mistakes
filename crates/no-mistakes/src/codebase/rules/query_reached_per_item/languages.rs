use super::config::{Allowance, Options};
use crate::codebase::rules::allow_audit::{AllowAudit, AllowUsage};
use crate::codebase::rules::path_filter::RulePathFilter;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::Unanalyzable;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::{bail, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

mod regions;
mod resolve;
#[cfg(test)]
mod tests;

use resolve::project;

pub(super) struct Sink {
    kind: String,
    module: String,
    export: String,
}

pub(super) struct Ctx<'a> {
    sinks: &'a [Sink],
    message: &'a str,
    unanalyzable: Unanalyzable,
    allow: &'a [Allowance],
    usage: Option<&'a AllowUsage>,
}

const PROVEN_MESSAGE: &str = "per-item call reaches a configured effect sink; batch the helper, merge the lookup, or add a justified suppression";

const LANGUAGES: &[(&str, &[&str])] = &[
    ("python", &["py"]),
    ("go", &["go"]),
    ("rust", &["rs"]),
    ("rails", &["rb"]),
    ("php", &["php"]),
    ("java", &["java"]),
    ("kotlin", &["kt"]),
    ("elixir", &["ex", "exs"]),
    ("dart", &["dart"]),
    ("swift", &["swift"]),
    ("dotnet", &["cs"]),
];

pub(super) fn known(language: &str) -> bool {
    LANGUAGES.iter().any(|(id, _)| *id == language)
}

pub(super) fn root_entries(config: &NoMistakesConfig, language: &str) -> Vec<String> {
    match language {
        "python" => config.tests.python.packages.clone(),
        "go" => config.tests.go.modules.clone(),
        "rust" => config.tests.rust.packages.clone(),
        "rails" => config.tests.rails.apps.clone(),
        "php" => config.tests.php.apps.clone(),
        "java" => config.tests.java.packages.clone(),
        "kotlin" => config.tests.kotlin.packages.clone(),
        "elixir" => config.tests.elixir.apps.clone(),
        "dart" => config.tests.dart.packages.clone(),
        "swift" => config.tests.swift.packages.clone(),
        "dotnet" => config
            .tests
            .dotnet
            .projects
            .values()
            .filter(|project| !project.project.trim().is_empty())
            .map(|project| project.project.clone())
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn collect(
    root: &Path,
    config: &NoMistakesConfig,
    application: &RuleDef,
    options: &Options,
    audit: &AllowAudit,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    let languages = deduped(&options.languages);
    if languages.is_empty() {
        return Ok(Vec::new());
    }
    let Some(session) = session else {
        bail!("query-reached-per-item: options.languages requires a prepared analysis session");
    };
    let policy = config.effective_rule_analysis(application)?;
    let sinks = sink_list(config, options);
    let ctx = Ctx {
        sinks: &sinks,
        message: application.message.as_deref().unwrap_or(PROVEN_MESSAGE),
        unanalyzable: policy.unanalyzable,
        allow: &options.allow,
        usage: Some(&audit.usage),
    };
    let dataset = session.dataset(root);
    let visible = dataset.paths_for(root);
    let sources = dataset.sources_for(root);
    let filter = RulePathFilter::new(root, config, application)?;
    let files = selected_files(root, config, &languages, &visible, &filter);
    let scanned: Vec<Vec<RuleFinding>> = files
        .par_iter()
        .map(|(path, extension)| {
            let file = crate::codebase::ts_source::relative_slash_path(root, path);
            let text = sources.read_path(path).ok();
            project(&file, text.as_deref(), extension, &ctx)
        })
        .collect();
    let mut findings: Vec<_> = scanned.into_iter().flatten().collect();
    findings.sort();
    Ok(findings)
}

fn deduped(languages: &[String]) -> Vec<String> {
    let mut selected = Vec::new();
    for language in languages {
        if !selected.contains(language) {
            selected.push(language.clone());
        }
    }
    selected
}

fn sink_list(config: &NoMistakesConfig, options: &Options) -> Vec<Sink> {
    let mut sinks = Vec::new();
    for kind in &options.effects {
        let Some(effect) = config.effects.get(kind) else {
            continue;
        };
        for target in &effect.targets {
            sinks.push(Sink {
                kind: kind.clone(),
                module: target.module.clone(),
                export: target.export.clone(),
            });
        }
    }
    sinks
}

fn selected_files(
    root: &Path,
    config: &NoMistakesConfig,
    languages: &[String],
    visible: &[PathBuf],
    filter: &RulePathFilter,
) -> Vec<(PathBuf, &'static str)> {
    let mut files = Vec::new();
    for language in languages {
        let Some((_, extensions)) = LANGUAGES.iter().find(|(id, _)| *id == language) else {
            continue;
        };
        let roots = scan_roots(root, &root_entries(config, language), language);
        for path in visible {
            let Some(found) = path.extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            let Some(extension) = extensions.iter().copied().find(|candidate| *candidate == found)
            else {
                continue;
            };
            if roots.iter().any(|dir| path.starts_with(dir)) && filter.is_match(path) {
                files.push((path.clone(), extension));
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

fn scan_roots(root: &Path, entries: &[String], language: &str) -> Vec<PathBuf> {
    if language != "dotnet" {
        return crate::codebase::lang_frontends::configured_roots(root, entries);
    }
    entries
        .iter()
        .map(|entry| {
            let mut path = crate::codebase::ts_resolver::normalize_path(
                &root.join(entry.trim_end_matches('/')),
            );
            if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("csproj" | "fsproj" | "vbproj")
            ) {
                path.pop();
            }
            path
        })
        .collect()
}
