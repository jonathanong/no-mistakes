use no_mistakes::codebase::rules::disable_directive_audit::ConsumedDisableDirective;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::rules::{
    suppress_domain_findings_with_source_locations, suppress_domain_findings_with_sources,
    SuppressedFinding, SuppressionTarget,
};
use no_mistakes::codebase::ts_source::SourceStore;
use no_mistakes::codebase::unique_exports::PreparedUniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;

mod provenance;
use provenance::suppress_rules_with_sources;
mod react;
pub(crate) use react::suppress_react;
use react::suppress_react_with_consumed;

pub(super) struct SuppressionAccounting {
    pub(super) suppressed: Vec<SuppressedFinding>,
    pub(super) consumed: Vec<ConsumedDisableDirective>,
}

impl SuppressionAccounting {
    pub(super) fn into_public(self, requested: bool) -> Vec<SuppressedFinding> {
        if requested {
            self.suppressed
        } else {
            Vec::new()
        }
    }
}

pub(super) fn take_pre_suppressed(
    completed: &mut super::CompletedDomainChecks,
) -> Vec<SuppressedFinding> {
    [
        &mut completed.react.pre_suppressed,
        &mut completed.queues.pre_suppressed,
        &mut completed.rules.pre_suppressed,
        &mut completed.filesystem_rules.pre_suppressed,
        &mut completed.integration.pre_suppressed,
        &mut completed.codebase.pre_suppressed,
    ]
    .into_iter()
    .flat_map(std::mem::take)
    .collect()
}

pub(super) struct Inputs<'a> {
    pub(super) pre_suppressed: Vec<SuppressedFinding>,
    pub(super) root: &'a std::path::Path,
    pub(super) sources: &'a SourceStore,
    pub(super) react: &'a mut Vec<react_traits::Violation>,
    /// React suppression is already applied by ordinary checks. Audit runs
    /// defer it so they can retain the corresponding accounting records.
    pub(super) react_suppression_targets: Option<&'a [Vec<react_traits::ReactSuppressionTarget>]>,
    pub(super) queues: &'a mut Vec<CheckFinding>,
    pub(super) rules: &'a mut Vec<RuleFinding>,
    pub(super) rule_suppression_sources: &'a [Option<String>],
    pub(super) filesystem: &'a mut Vec<RuleFinding>,
    pub(super) integration: &'a mut Vec<IntegrationFinding>,
    pub(super) codebase: &'a mut Vec<PreparedUniqueExportFinding>,
    pub(super) advisories: &'a mut Vec<RuleFinding>,
}

pub(super) fn apply_if_requested(
    retain_accounting: bool,
    input: Inputs<'_>,
) -> SuppressionAccounting {
    // Directive filtering is part of the ordinary check contract. The flag
    // controls whether we retain the accounting records, not whether a
    // directive hides its matching finding.
    let mut accounting = apply(input, retain_accounting);
    if !retain_accounting {
        accounting.suppressed.clear();
        accounting.consumed.clear();
    }
    accounting
}

fn apply(input: Inputs<'_>, retain_accounting: bool) -> SuppressionAccounting {
    let Inputs {
        pre_suppressed,
        root,
        sources,
        react,
        react_suppression_targets,
        queues,
        rules,
        rule_suppression_sources,
        filesystem,
        integration,
        codebase,
        advisories,
    } = input;
    let mut suppressed = pre_suppressed;
    let mut consumed = Vec::new();
    if let Some(react_suppression_targets) = react_suppression_targets {
        suppress_react_with_consumed(
            root,
            sources,
            react,
            react_suppression_targets,
            &mut suppressed,
            Some(&mut consumed),
        );
    }
    suppressed.extend(suppress_domain_findings_with_sources(
        root,
        queues,
        sources,
        |finding| SuppressionTarget {
            domain: "queues",
            rule: "queues-check",
            file: &finding.file,
            line: Some(finding.line),
            reason: &finding.message,
            identity: None,
        },
    ));
    suppress_rules_with_sources(
        root,
        sources,
        rules,
        rule_suppression_sources,
        "rules",
        &mut suppressed,
    );
    suppress_rules(root, sources, filesystem, "filesystem", &mut suppressed);
    suppress_rules(root, sources, advisories, "advisories", &mut suppressed);
    suppressed.extend(suppress_domain_findings_with_sources(
        root,
        integration,
        sources,
        |finding| SuppressionTarget {
            domain: "integration",
            rule: "integration-test-no-mocks",
            file: &finding.file,
            line: Some(finding.line as usize),
            reason: &finding.message,
            identity: None,
        },
    ));
    suppressed.extend(suppress_domain_findings_with_source_locations(
        root,
        codebase,
        sources,
        |finding| SuppressionTarget {
            domain: "codebase",
            rule: &finding.finding.rule,
            file: &finding.finding.file,
            line: Some(finding.finding.line as usize),
            reason: &finding.finding.message,
            identity: None,
        },
        |finding| {
            finding
                .suppression_source_location
                .as_ref()
                .map(|(file, line)| (file.as_str(), usize::try_from(*line).ok()))
        },
    ));
    if retain_accounting {
        consumed.extend(suppressed.iter().map(ConsumedDisableDirective::from));
        consumed.sort();
        consumed.dedup();
    }
    suppressed.sort();
    SuppressionAccounting {
        suppressed,
        consumed,
    }
}

fn suppress_rules(
    root: &std::path::Path,
    sources: &SourceStore,
    findings: &mut Vec<RuleFinding>,
    domain: &'static str,
    suppressed: &mut Vec<SuppressedFinding>,
) {
    suppressed.extend(suppress_domain_findings_with_sources(
        root,
        findings,
        sources,
        |finding| SuppressionTarget {
            domain,
            rule: &finding.rule,
            file: &finding.file,
            line: Some(finding.line),
            reason: &finding.message,
            identity: None,
        },
    ));
}
