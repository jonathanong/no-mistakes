use super::super::super::RuleFinding;
use super::super::extract::{extract_named_destinations_with, ExtractedDestinations};
use super::super::routes::{destination_matches, should_skip_destination, strip_query_and_hash};
use super::{contains_word, finding, word_line};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::{EffectiveRuleAnalysis, Effort, Unanalyzable};
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) const REWRITE: &str = "return a literal array of objects with string destination fields, for example return [{ source: '/a', destination: '/b', permanent: true }]";

#[cfg(test)]
pub fn check_named_section(
    rel: &str,
    source: &str,
    config_path: &Path,
    name: &str,
    kind: &str,
    route_set: &BTreeSet<String>,
    app_root_label: &str,
) -> Vec<RuleFinding> {
    check_named_section_with(
        rel,
        source,
        config_path,
        name,
        kind,
        route_set,
        app_root_label,
        EffectiveRuleAnalysis {
            effort: Effort::Max,
            unanalyzable: Unanalyzable::Report,
        },
    )
    .findings
}

#[allow(clippy::too_many_arguments)]
pub(super) fn check_named_section_with(
    rel: &str,
    source: &str,
    config_path: &Path,
    name: &str,
    kind: &str,
    route_set: &BTreeSet<String>,
    app_root_label: &str,
    analysis: EffectiveRuleAnalysis,
) -> AnalysisDiagnostics {
    let low = analysis.effort == Effort::Low;
    let extracted = extract_named_destinations_with(config_path, source, name, low);
    if !extracted.body_found {
        return missing_body(rel, source, name, kind, low, analysis.unanalyzable);
    }
    if extracted.destinations.is_empty() && extracted.saw_destination_property {
        return no_string_destinations(rel, source, name, kind, low, analysis.unanalyzable);
    }
    let mut diagnostics = AnalysisDiagnostics::default();
    if extracted.incomplete {
        push_shape(
            &mut diagnostics,
            low,
            analysis.unanalyzable,
            incomplete_finding(rel, source, name, kind, low),
        );
        if low {
            return diagnostics;
        }
    }
    collect_destinations(
        &mut diagnostics,
        &extracted,
        rel,
        kind,
        route_set,
        app_root_label,
    );
    diagnostics
}

fn missing_body(
    rel: &str,
    source: &str,
    name: &str,
    kind: &str,
    low: bool,
    policy: Unanalyzable,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    if !contains_word(source, name) {
        return diagnostics;
    }
    let message = if low {
        format!("{rel}: nextjs-redirect-destinations could not locate the {name}() body at effort low; {REWRITE}")
    } else {
        format!(
            "{rel}: nextjs-redirect-destinations could not locate the {name}() body; \
the {kind} definition is not a method or function/arrow property. Update the Next.js config \
shape or the extractor so stale {kind} destinations cannot slip through."
        )
    };
    push_shape(
        &mut diagnostics,
        low,
        policy,
        finding(rel, word_line(source, name), message),
    );
    diagnostics
}

fn no_string_destinations(
    rel: &str,
    source: &str,
    name: &str,
    kind: &str,
    low: bool,
    policy: Unanalyzable,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    let message = if low {
        format!("{rel}: nextjs-redirect-destinations found destination properties in {name}() that are not string literals at effort low; {REWRITE}")
    } else {
        format!(
            "{rel}: nextjs-redirect-destinations found destination properties in {name}() \
but extracted no string destinations; use string literal destinations or update the extractor \
so {kind} construction changes are not silent."
        )
    };
    push_shape(
        &mut diagnostics,
        low,
        policy,
        finding(rel, word_line(source, name), message),
    );
    diagnostics
}

fn incomplete_finding(rel: &str, source: &str, name: &str, kind: &str, low: bool) -> RuleFinding {
    let message = if low {
        format!("{rel}: nextjs-redirect-destinations extraction is unanalyzable in {name}() at effort low; {REWRITE}")
    } else {
        format!(
            "{rel}: nextjs-redirect-destinations extraction is incomplete in {name}(); use bounded immutable constants, arrays and map callbacks so every {kind} destination can be checked."
        )
    };
    finding(rel, word_line(source, name), message)
}

fn collect_destinations(
    diagnostics: &mut AnalysisDiagnostics,
    extracted: &ExtractedDestinations,
    rel: &str,
    kind: &str,
    route_set: &BTreeSet<String>,
    app_root_label: &str,
) {
    for destination in &extracted.destinations {
        let dest_path = strip_query_and_hash(&destination.value);
        if should_skip_destination(dest_path) || destination_matches(route_set, dest_path) {
            continue;
        }
        diagnostics.findings.push(finding(
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
}

fn push_shape(
    diagnostics: &mut AnalysisDiagnostics,
    low: bool,
    policy: Unanalyzable,
    finding: RuleFinding,
) {
    if low {
        diagnostics.unanalyzable(policy, finding);
    } else {
        diagnostics.findings.push(finding);
    }
}
