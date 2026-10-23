use super::extract::{extract_named_destinations_from_program, ExtractedDestinations};
use super::scan::{check_named_section, NamedCheck};
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
    pub(super) specifier: Option<String>,
}

pub(super) fn analyze_config(path: &Path, source: &str, low: bool) -> ConfigAnalysis {
    crate::ast::with_program(path, source, |program, source| ConfigAnalysis {
        redirects: extract_named_destinations_from_program(program, source, "redirects", low),
        rewrites: extract_named_destinations_from_program(program, source, "rewrites", low),
        specifier: default_module_specifier(program).map(str::to_string),
    })
    .unwrap_or_default()
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
    pub(super) specifier: Option<&'a str>,
    pub(super) analysis: EffectiveRuleAnalysis,
}

pub(super) fn append_delegated(request: &DelegateRequest<'_>) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    let follow_redirects = !contains_word(request.source, "redirects");
    let follow_rewrites = request.include_rewrites && !contains_word(request.source, "rewrites");
    if !follow_redirects && !follow_rewrites {
        return diagnostics;
    }
    let Some(specifier) = request.specifier else {
        return diagnostics;
    };
    let Some(delegated) =
        resolve_relative(request.config_path, specifier, request.files).or_else(|| {
            request
                .resolver
                .resolve_alias(specifier, request.config_path)
                .filter(|path| request.files.contains(path))
        })
    else {
        return diagnostics;
    };
    let Some(delegated_source) = super::super::read_source(request.sources, &delegated) else {
        return diagnostics;
    };
    if has_disable_file_comment(&delegated_source, RULE_ID) {
        return diagnostics;
    }
    let rel = relative_slash_path(request.root, &delegated);
    let analysis = analyze_config(
        &delegated,
        &delegated_source,
        request.analysis.effort == Effort::Low,
    );
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
