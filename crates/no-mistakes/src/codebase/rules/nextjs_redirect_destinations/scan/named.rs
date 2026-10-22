use super::super::super::RuleFinding;
use super::super::extract::{extract_named_destinations_with, ExtractedDestination};
use super::super::routes::{destination_matches, should_skip_destination, strip_query_and_hash};
use super::{contains_word, finding, word_line};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::{EffectiveRuleAnalysis, Effort, Unanalyzable};
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) const REWRITE: &str = "return a literal array of objects with string destination fields, for example return [{ source: '/a', destination: '/b', permanent: true }]";

pub struct NamedCheck<'a> {
    pub rel: &'a str,
    pub source: &'a str,
    pub config_path: &'a Path,
    pub name: &'a str,
    pub kind: &'a str,
    pub route_set: &'a BTreeSet<String>,
    pub app_root_label: &'a str,
    pub analysis: EffectiveRuleAnalysis,
}

pub fn check_named_section(check: NamedCheck<'_>) -> AnalysisDiagnostics {
    let low = check.analysis.effort == Effort::Low;
    let extracted =
        extract_named_destinations_with(check.config_path, check.source, check.name, low);
    if !extracted.body_found {
        return missing_body(&check, low);
    }
    if extracted.destinations.is_empty() && extracted.saw_destination_property {
        return no_string_destinations(&check, low);
    }
    let mut diagnostics = AnalysisDiagnostics::default();
    if extracted.incomplete {
        push_shape(
            &mut diagnostics,
            low,
            check.analysis.unanalyzable,
            incomplete_finding(&check, low),
        );
    }
    collect_destinations(
        &mut diagnostics,
        &extracted.destinations,
        check.rel,
        check.kind,
        check.route_set,
        check.app_root_label,
    );
    diagnostics
}

fn missing_body(check: &NamedCheck<'_>, low: bool) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    if !contains_word(check.source, check.name) {
        return diagnostics;
    }
    let message = if low {
        format!(
            "{}: nextjs-redirect-destinations could not locate the {}() body at effort low; {REWRITE}",
            check.rel, check.name
        )
    } else {
        format!(
            "{}: nextjs-redirect-destinations could not locate the {}() body; \
the {} definition is not a method or function/arrow property. Update the Next.js config \
shape or the extractor so stale {} destinations cannot slip through.",
            check.rel, check.name, check.kind, check.kind
        )
    };
    push_shape(
        &mut diagnostics,
        low,
        check.analysis.unanalyzable,
        finding(check.rel, word_line(check.source, check.name), message),
    );
    diagnostics
}

fn no_string_destinations(check: &NamedCheck<'_>, low: bool) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    let message = if low {
        format!(
            "{}: nextjs-redirect-destinations found destination properties in {}() that are not string literals at effort low; {REWRITE}",
            check.rel, check.name
        )
    } else {
        format!(
            "{}: nextjs-redirect-destinations found destination properties in {}() \
but extracted no string destinations; use string literal destinations or update the extractor \
so {} construction changes are not silent.",
            check.rel, check.name, check.kind
        )
    };
    push_shape(
        &mut diagnostics,
        low,
        check.analysis.unanalyzable,
        finding(check.rel, word_line(check.source, check.name), message),
    );
    diagnostics
}

fn incomplete_finding(check: &NamedCheck<'_>, low: bool) -> RuleFinding {
    let message = if low {
        format!(
            "{}: nextjs-redirect-destinations extraction is unanalyzable in {}() at effort low; {REWRITE}",
            check.rel, check.name
        )
    } else {
        format!(
            "{}: nextjs-redirect-destinations extraction is incomplete in {}(); use bounded immutable constants, arrays and map callbacks so every {} destination can be checked.",
            check.rel, check.name, check.kind
        )
    };
    finding(check.rel, word_line(check.source, check.name), message)
}

fn collect_destinations(
    diagnostics: &mut AnalysisDiagnostics,
    destinations: &[ExtractedDestination],
    rel: &str,
    kind: &str,
    route_set: &BTreeSet<String>,
    app_root_label: &str,
) {
    for destination in destinations {
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
