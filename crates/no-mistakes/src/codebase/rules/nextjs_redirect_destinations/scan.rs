use super::super::RuleFinding;
use super::config::{display_app_root, resolve_config_path, resolved_app_root};
use super::options::Options;
use super::RULE_ID;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_source::{
    byte_offset_to_line, has_disable_file_comment, relative_slash_path,
};
use crate::config::v2::EffectiveRuleAnalysis;
use std::path::{Path, PathBuf};

mod named;
pub(super) use named::{check_named_section, NamedCheck};

pub(super) fn scan_diagnostics(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    route_files: &[PathBuf],
    target_roots: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    analysis: EffectiveRuleAnalysis,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    for target_root in target_roots {
        let next = scan_target(
            root,
            opts,
            files,
            route_files,
            target_root,
            sources,
            analysis,
        );
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
    sources: &crate::codebase::ts_source::SourceStore,
    analysis: EffectiveRuleAnalysis,
) -> AnalysisDiagnostics {
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
    let mut diagnostics = check_named_section(NamedCheck {
        rel: &rel,
        source: &source,
        config_path: &config_path,
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
            config_path: &config_path,
            name: "rewrites",
            kind: "rewrite",
            route_set: &route_set,
            app_root_label: &app_root_label,
            analysis,
        }));
    }
    diagnostics
}

pub(super) fn contains_word(source: &str, word: &str) -> bool {
    source
        .match_indices(word)
        .any(|(index, _)| is_word_at(source, index, word.len()))
}

pub(super) fn word_line(source: &str, word: &str) -> usize {
    source
        .match_indices(word)
        .find(|(index, _)| is_word_at(source, *index, word.len()))
        .map(|(index, _)| byte_offset_to_line(source, index) as usize)
        .unwrap_or(1)
}

fn is_word_at(source: &str, index: usize, len: usize) -> bool {
    let before = source[..index].chars().next_back();
    let after = source[index + len..].chars().next();
    !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
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
