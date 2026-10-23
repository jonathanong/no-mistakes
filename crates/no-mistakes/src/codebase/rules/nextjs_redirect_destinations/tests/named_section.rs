use super::super::extract::{extract_named_destinations_from_program, ExtractedDestinations};
use super::super::scan::{check_named_section as check, NamedCheck};
use super::super::RuleFinding;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn extract_named_destinations(
    path: &Path,
    source: &str,
    name: &str,
) -> ExtractedDestinations {
    crate::ast::with_program(path, source, |program, source| {
        extract_named_destinations_from_program(program, source, name, false)
    })
    .unwrap_or_default()
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
    check(NamedCheck {
        rel,
        source,
        name,
        kind,
        extracted: &extracted,
        route_set,
        app_root_label,
        analysis: crate::config::v2::EffectiveRuleAnalysis {
            effort: crate::config::v2::Effort::Max,
            unanalyzable: crate::config::v2::Unanalyzable::Report,
        },
    })
    .findings
}
