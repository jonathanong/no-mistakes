use super::super::{self as rules, analysis_policy::AnalysisDiagnostics};
use super::{collect_config_errors, hit, judge, local_action, options, scan, text, RULE_ID};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut errors = Vec::new();
    collect_config_errors(root, None, config, &mut errors);
    rules::validate_options::finish(errors)?;
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let application = Application {
            options: rule.try_rule_options()?,
            message: rule.message.as_deref().map(str::trim).filter(|msg| !msg.is_empty()),
            unanalyzable: config.effective_rule_analysis(rule)?.unanalyzable,
        };
        options::validate(&application.options)?;
        let selected = rules::selected_rule_files(root, config, rule, files)?;
        let batches: Vec<_> = selected
            .par_iter()
            .map(|path| check_path(root, path, sources, &application, defer_suppression))
            .collect();
        for batch in batches {
            output.append(batch);
        }
    }
    if !defer_suppression {
        rules::suppress_rule_findings_with_sources(root, &mut output.findings, sources);
        rules::suppress_rule_findings_with_sources(root, &mut output.advisories, sources);
    }
    Ok(rules::file_diagnostics::sorted(output))
}

struct Application<'a> {
    options: options::Options,
    message: Option<&'a str>,
    unanalyzable: Unanalyzable,
}

fn check_path(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    application: &Application<'_>,
    defer_suppression: bool,
) -> AnalysisDiagnostics {
    let rel = relative_slash_path(root, path);
    let Some(action) = rules::github_actions_workflow_files::gha_kind(&rel) else {
        return AnalysisDiagnostics::default();
    };
    match sources.read_path(path) {
        Ok(source) => evaluate(&rel, &source, action, application, root, sources),
        Err(error) => AnalysisDiagnostics {
            findings: rules::github_actions_workflow_files::unreadable_finding(
                &error,
                defer_suppression,
                |line| {
                    hit(
                        &rel,
                        line,
                        format!(
                            "Cannot read GitHub Actions YAML as UTF-8 ({error}); restore a readable file so cache keys can be checked."
                        ),
                    )
                },
            ).into_iter().collect(),
            ..Default::default()
        },
    }
}

fn evaluate(
    rel: &str,
    source: &str,
    action: bool,
    application: &Application<'_>,
    root: &Path,
    sources: &SourceStore,
) -> AnalysisDiagnostics {
    let options = &application.options;
    let configured: Vec<String> = options.caches.iter()
        .map(|cache| text::norm_path(&cache.path))
        .chain(options.forbidden_paths.iter().map(|path| text::norm_path(path)))
        .collect();
    let finding = match scan::analyze(source, action, &configured) {
        scan::Document::Ignored => return AnalysisDiagnostics::default(),
        scan::Document::Invalid { line, detail } => hit(
            rel,
            line,
            format!("Invalid YAML ({detail}); fix the YAML so cache keys can be checked."),
        ),
        scan::Document::Unaligned => hit(
            rel,
            1,
            "Cache steps could not be aligned to block-style source locations. Rewrite anchors, aliases, and unusual flow as ordinary block-style steps so cache keys can be checked.".into(),
        ),
        scan::Document::Jobs(jobs) => {
            return judge::judge(
                rel,
                &jobs,
                options,
                application.message,
                &|uses| read_local_action(root, sources, uses),
                application.unanalyzable,
            );
        }
    };
    AnalysisDiagnostics { findings: vec![finding], ..Default::default() }
}

fn read_local_action(root: &Path, sources: &SourceStore, uses: &str) -> Option<String> {
    let rel = local_action::local_action_dir(uses)?;
    for name in ["action.yml", "action.yaml"] {
        let path = root.join(rel).join(name);
        if let Ok(text) = sources.read_path(&path) {
            return Some(text.to_string());
        }
    }
    None
}
