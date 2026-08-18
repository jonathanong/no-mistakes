use super::super::RuleFinding;
use super::config::{display_app_root, resolve_config_path, resolved_app_root};
use super::extract::extract_named_destinations;
use super::options::Options;
use super::routes::{
    build_route_set, destination_matches, should_skip_destination, strip_query_and_hash,
};
use super::RULE_ID;
use crate::codebase::ts_source::{
    byte_offset_to_line, has_disable_file_comment, relative_slash_path,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for target_root in target_roots {
        findings.extend(scan_target(root, opts, files, target_root, sources));
    }
    findings
}

fn scan_target(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let Some(config_path) = resolve_config_path(target_root, files, opts.config_path.as_deref())
    else {
        return Vec::new();
    };
    let Some(source) = super::super::read_source(sources, &config_path) else {
        return Vec::new();
    };
    if has_disable_file_comment(&source, RULE_ID) {
        return Vec::new();
    }
    let app_root = resolved_app_root(target_root, opts.app_root.as_deref());
    let route_set = build_route_set(files, &app_root);
    let app_root_label = display_app_root(opts.app_root.as_deref());
    let rel = relative_slash_path(root, &config_path);
    let mut findings = check_named_section(
        &rel,
        &source,
        &config_path,
        "redirects",
        "redirect",
        &route_set,
        &app_root_label,
    );
    if opts.include_rewrites {
        findings.extend(check_named_section(
            &rel,
            &source,
            &config_path,
            "rewrites",
            "rewrite",
            &route_set,
            &app_root_label,
        ));
    }
    findings
}

pub(super) fn check_named_section(
    rel: &str,
    source: &str,
    config_path: &Path,
    name: &str,
    kind: &str,
    route_set: &BTreeSet<String>,
    app_root_label: &str,
) -> Vec<RuleFinding> {
    let extracted = extract_named_destinations(config_path, source, name);
    if !extracted.body_found {
        if contains_word(source, name) {
            return vec![finding(
                rel,
                word_line(source, name),
                format!(
                    "{rel}: nextjs-redirect-destinations could not locate the {name}() body; \
the {kind} definition is not a method or function/arrow property. Update the Next.js config \
shape or the extractor so stale {kind} destinations cannot slip through."
                ),
            )];
        }
        return Vec::new();
    }
    if extracted.destinations.is_empty() && extracted.saw_destination_property {
        return vec![finding(
            rel,
            word_line(source, name),
            format!(
                "{rel}: nextjs-redirect-destinations found destination properties in {name}() \
but extracted no string destinations; use string literal destinations or update the extractor \
so {kind} construction changes are not silent."
            ),
        )];
    }
    let mut findings = Vec::new();
    for destination in extracted.destinations {
        let dest_path = strip_query_and_hash(&destination.value);
        if should_skip_destination(dest_path) {
            continue;
        }
        if destination_matches(route_set, dest_path) {
            continue;
        }
        findings.push(finding(
            rel,
            destination.line,
            format!(
                "{rel}: {kind} destination '{}' has no matching App Router page \
(expected {app_root_label}/**/page.tsx for route {dest_path}); remove the {kind} or restore \
the page so users are not sent to a 404.",
                destination.value
            ),
        ));
    }
    findings
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

fn finding(file: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: None,
    }
}
