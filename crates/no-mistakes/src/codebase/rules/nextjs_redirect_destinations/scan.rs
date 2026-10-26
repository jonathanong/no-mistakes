use super::super::RuleFinding;
use super::config::{display_app_root, resolve_config_path, resolved_app_root};
use super::options::Options;
pub(super) use super::words::{contains_word, word_line};
use super::RULE_ID;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use crate::config::v2::{EffectiveRuleAnalysis, Effort};
use std::path::{Path, PathBuf};

mod named;
pub(super) use named::{check_named_section, NamedCheck};

pub(super) struct ScanContext<'a> {
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
    pub(super) resolver: &'a super::PreparedConfigResolver<'a>,
    pub(super) analysis: EffectiveRuleAnalysis,
    pub(super) defer_suppression: bool,
}

pub(super) fn scan_diagnostics(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    route_files: &[PathBuf],
    target_roots: &[PathBuf],
    context: &ScanContext<'_>,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    for target_root in target_roots {
        let next = scan_target(root, opts, files, route_files, target_root, context);
        diagnostics.append(next);
    }
    diagnostics
}

fn scan_target(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    route_files: &[PathBuf],
    target_root: &Path,
    context: &ScanContext<'_>,
) -> AnalysisDiagnostics {
    let ScanContext {
        sources,
        resolver,
        analysis,
        defer_suppression,
    } = *context;
    let Some(config_path) = resolve_config_path(target_root, files, opts.config_path.as_deref())
    else {
        return AnalysisDiagnostics::default();
    };
    let rel = relative_slash_path(root, &config_path);
    let source = match sources.read_path(&config_path) {
        Ok(source) => source,
        Err(error) => {
            return entry_failure(analysis, &rel, 1, format!("could not read config: {error}"));
        }
    };
    if !defer_suppression && has_disable_file_comment(&source, RULE_ID) {
        return AnalysisDiagnostics::default();
    }
    let app_root = resolved_app_root(target_root, opts.app_root.as_deref());
    let route_set = super::routes::build_route_set(route_files, &app_root);
    let app_root_label = display_app_root(opts.app_root.as_deref());
    let extracted = match super::delegate::analyze_config(
        &config_path,
        &source,
        analysis.effort == Effort::Low,
    ) {
        Ok(extracted) => extracted,
        Err(error) => {
            return entry_failure(
                analysis,
                &rel,
                first_code_line(&source),
                format!("could not parse config: {error}"),
            );
        }
    };
    let mut diagnostics = check_named_section(NamedCheck {
        rel: &rel,
        source: &source,
        extracted: &extracted.redirects,
        name: "redirects",
        kind: "redirect",
        route_set: &route_set,
        app_root_label: &app_root_label,
        analysis,
    });
    if opts.include_rewrites {
        diagnostics.append(check_named_section(NamedCheck {
            rel: &rel,
            source: &source,
            extracted: &extracted.rewrites,
            name: "rewrites",
            kind: "rewrite",
            route_set: &route_set,
            app_root_label: &app_root_label,
            analysis,
        }));
    }
    diagnostics.append(super::delegate::append_delegated(
        &super::delegate::DelegateRequest {
            root,
            config_path: &config_path,
            source: &source,
            files,
            sources,
            resolver,
            route_set: &route_set,
            app_root_label: &app_root_label,
            include_rewrites: opts.include_rewrites,
            specifier: extracted
                .specifier
                .as_ref()
                .map(|(specifier, line)| (specifier.as_str(), *line)),
            analysis,
            defer_suppression,
        },
    ));
    diagnostics
}

fn entry_failure(
    analysis: EffectiveRuleAnalysis,
    rel: &str,
    line: usize,
    error: String,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    diagnostics.unanalyzable(
        analysis.unanalyzable,
        finding(
            rel,
            line,
            format!(
                "{rel}: nextjs-redirect-destinations could not analyze the entry config: {error}; fix the file so redirect and rewrite destinations can be checked."
            ),
        ),
    );
    diagnostics
}

// The parser exposes error text but no stable source location to this rule.
// Anchor a failed config at its first code line for line/next-line directives.
fn first_code_line(source: &str) -> usize {
    let mut in_block = false;
    for (index, line) in source.lines().enumerate() {
        let mut rest = line.trim_start();
        loop {
            if in_block {
                if let Some(end) = rest.find("*/") {
                    rest = rest[end + 2..].trim_start();
                    in_block = false;
                } else {
                    break;
                }
            } else if rest.starts_with("/*") {
                in_block = true;
                rest = &rest[2..];
            } else {
                if !rest.is_empty() && !rest.starts_with("//") {
                    return index + 1;
                }
                break;
            }
        }
    }
    1
}

pub(super) fn finding(file: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: None,
    }
}
