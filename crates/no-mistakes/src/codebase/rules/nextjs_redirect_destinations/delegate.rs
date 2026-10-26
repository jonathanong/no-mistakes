use super::extract::{extract_named_destinations_from_program, ExtractedDestinations};
use super::scan::{check_named_section, finding, NamedCheck};
use super::words::contains_word;
use super::RULE_ID;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use crate::config::v2::{EffectiveRuleAnalysis, Effort};
mod resolve;
use resolve::{default_module_specifier, resolve_relative};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct ConfigAnalysis {
    pub(super) redirects: ExtractedDestinations,
    pub(super) rewrites: ExtractedDestinations,
    pub(super) specifier: Option<(String, usize)>,
}

pub(super) fn analyze_config(
    path: &Path,
    source: &str,
    low: bool,
) -> anyhow::Result<ConfigAnalysis> {
    crate::ast::with_program(path, source, |program, source| ConfigAnalysis {
        redirects: extract_named_destinations_from_program(program, source, "redirects", low),
        rewrites: extract_named_destinations_from_program(program, source, "rewrites", low),
        specifier: default_module_specifier(program).map(|(specifier, offset)| {
            (
                specifier.to_string(),
                crate::codebase::ts_source::byte_offset_to_line(source, offset as usize) as usize,
            )
        }),
    })
}

pub(super) struct DelegateRequest<'a> {
    pub(super) root: &'a Path,
    pub(super) config_path: &'a Path,
    pub(super) source: &'a str,
    pub(super) files: &'a [PathBuf],
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
    pub(super) resolver: &'a super::PreparedConfigResolver<'a>,
    pub(super) route_set: &'a BTreeSet<String>,
    pub(super) app_root_label: &'a str,
    pub(super) include_rewrites: bool,
    pub(super) specifier: Option<(&'a str, usize)>,
    pub(super) analysis: EffectiveRuleAnalysis,
    pub(super) defer_suppression: bool,
}

pub(super) fn append_delegated(request: &DelegateRequest<'_>) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    let follow_redirects = !contains_word(request.source, "redirects");
    let follow_rewrites = request.include_rewrites && !contains_word(request.source, "rewrites");
    if !follow_redirects && !follow_rewrites {
        return diagnostics;
    }
    let Some((specifier, line)) = request.specifier else {
        return diagnostics;
    };
    let target = if specifier.starts_with("./") || specifier.starts_with("../") {
        Some(resolve_relative(
            request.config_path,
            specifier,
            request.files,
        ))
    } else {
        request
            .resolver
            .resolve_alias(specifier, request.config_path)
    };
    // Unmatched package/baseUrl-only imports are outside the declared one-hop scope.
    let Some(target) = target else {
        return diagnostics;
    };
    let Some(delegated) = target.filter(|path| request.files.contains(path)) else {
        let rel = relative_slash_path(request.root, request.config_path);
        let finding = finding(
            &rel,
            line,
            format!(
                "{rel}: nextjs-redirect-destinations extraction is incomplete: delegated config specifier '{specifier}' could not be resolved; restore the local module or correct its relative path or TypeScript paths mapping so redirect and rewrite destinations can be checked."
            ),
        );
        if request.analysis.effort == Effort::Low {
            diagnostics.unanalyzable(request.analysis.unanalyzable, finding);
        } else {
            diagnostics.findings.push(finding);
        }
        return diagnostics;
    };
    let delegated_source = match request.sources.read_path(&delegated) {
        Ok(source) => source,
        Err(error) => {
            return delegation_error(request, line, &delegated, "read", &error.to_string())
        }
    };
    if !request.defer_suppression && has_disable_file_comment(&delegated_source, RULE_ID) {
        return diagnostics;
    }
    let rel = relative_slash_path(request.root, &delegated);
    let analysis = match analyze_config(
        &delegated,
        &delegated_source,
        request.analysis.effort == Effort::Low,
    ) {
        Ok(analysis) => analysis,
        Err(error) => {
            return delegation_error(request, line, &delegated, "parsed", &error.to_string())
        }
    };
    for (follow, name, kind, extracted) in [
        (
            follow_redirects,
            "redirects",
            "redirect",
            &analysis.redirects,
        ),
        (follow_rewrites, "rewrites", "rewrite", &analysis.rewrites),
    ] {
        if follow {
            diagnostics.append(check_named_section(NamedCheck {
                rel: &rel,
                source: &delegated_source,
                name,
                kind,
                extracted,
                route_set: request.route_set,
                app_root_label: request.app_root_label,
                analysis: request.analysis,
            }));
        }
    }
    diagnostics
}

fn delegation_error(
    request: &DelegateRequest<'_>,
    line: usize,
    delegated: &Path,
    operation: &str,
    error: &str,
) -> AnalysisDiagnostics {
    let config = relative_slash_path(request.root, request.config_path);
    let target = relative_slash_path(request.root, delegated);
    let finding = finding(
        &config,
        line,
        format!(
            "{config}: nextjs-redirect-destinations extraction is incomplete: delegated config '{target}' could not be {operation}: {error}; restore a readable, valid module so redirect and rewrite destinations can be checked."
        ),
    );
    let mut diagnostics = AnalysisDiagnostics::default();
    diagnostics.unanalyzable(request.analysis.unanalyzable, finding);
    diagnostics
}
