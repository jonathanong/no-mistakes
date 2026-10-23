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
        diagnostics.findings.extend(next.findings);
        diagnostics.advisories.extend(next.advisories);
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
    } = *context;
    let Some(config_path) = resolve_config_path(target_root, files, opts.config_path.as_deref())
    else {
        return AnalysisDiagnostics::default();
    };
    let Some(source) = super::super::read_source(sources, &config_path) else {
        return AnalysisDiagnostics::default();
    };
    if has_disable_file_comment(&source, RULE_ID) {
        return AnalysisDiagnostics::default();
    }
    let app_root = resolved_app_root(target_root, opts.app_root.as_deref());
    let route_set = super::routes::build_route_set(route_files, &app_root);
    let app_root_label = display_app_root(opts.app_root.as_deref());
    let rel = relative_slash_path(root, &config_path);
    let extracted =
        super::delegate::analyze_config(&config_path, &source, analysis.effort == Effort::Low)
            .unwrap_or_default();
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
        },
    ));
    diagnostics
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
