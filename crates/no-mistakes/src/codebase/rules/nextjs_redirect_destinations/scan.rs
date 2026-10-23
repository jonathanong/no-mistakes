use super::super::RuleFinding;
use super::config::{display_app_root, resolve_config_path, resolved_app_root};
use super::options::Options;
use super::routes::{
    build_route_set, destination_matches, should_skip_destination, strip_query_and_hash,
};
use super::words::{contains_word, word_line};
use super::RULE_ID;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    route_files: &[PathBuf],
    target_roots: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    resolver: &super::PreparedConfigResolver<'_>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for target_root in target_roots {
        findings.extend(scan_target(
            root,
            opts,
            files,
            route_files,
            target_root,
            sources,
            resolver,
        ));
    }
    findings
}

fn scan_target(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    route_files: &[PathBuf],
    target_root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
    resolver: &super::PreparedConfigResolver<'_>,
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
    let route_set = build_route_set(route_files, &app_root);
    let app_root_label = display_app_root(opts.app_root.as_deref());
    let rel = relative_slash_path(root, &config_path);
    let analysis = super::delegate::analyze_config(&config_path, &source);
    let mut findings = section_findings(
        &rel,
        &source,
        "redirects",
        "redirect",
        &analysis.redirects,
        &route_set,
        &app_root_label,
    );
    if opts.include_rewrites {
        findings.extend(section_findings(
            &rel,
            &source,
            "rewrites",
            "rewrite",
            &analysis.rewrites,
            &route_set,
            &app_root_label,
        ));
    }
    super::delegate::append_delegated(
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
            specifier: analysis.specifier.as_deref(),
        },
        &mut findings,
    );
    findings
}

pub(super) fn section_findings(
    rel: &str,
    source: &str,
    name: &str,
    kind: &str,
    extracted: &super::extract::ExtractedDestinations,
    route_set: &BTreeSet<String>,
    app_root_label: &str,
) -> Vec<RuleFinding> {
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
    if extracted.incomplete {
        findings.push(finding(rel, word_line(source, name), format!(
            "{rel}: nextjs-redirect-destinations extraction is incomplete in {name}(); use bounded immutable constants, arrays and map callbacks so every {kind} destination can be checked."
        )));
    }
    for destination in &extracted.destinations {
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
