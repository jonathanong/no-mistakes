use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::rules::{
    suppress_domain_findings_with_sources, SuppressedFinding, SuppressionTarget,
};
use no_mistakes::codebase::ts_source::SourceStore;
use no_mistakes::codebase::unique_exports::UniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;

mod provenance;
use provenance::suppress_rules_with_sources;
mod react;
pub(super) use react::suppress_react;

pub(super) struct Inputs<'a> {
    pub(super) root: &'a std::path::Path,
    pub(super) sources: &'a SourceStore,
    pub(super) react: &'a mut Vec<react_traits::Violation>,
    pub(super) react_suppression_targets: &'a [Vec<react_traits::ReactSuppressionTarget>],
    pub(super) queues: &'a mut Vec<CheckFinding>,
    pub(super) rules: &'a mut Vec<RuleFinding>,
    pub(super) rule_suppression_sources: &'a [Option<String>],
    pub(super) filesystem: &'a mut Vec<RuleFinding>,
    pub(super) integration: &'a mut Vec<IntegrationFinding>,
    pub(super) codebase: &'a mut Vec<UniqueExportFinding>,
    pub(super) advisories: &'a mut Vec<RuleFinding>,
}

pub(super) fn apply_if_requested(
    include_suppressed: bool,
    input: Inputs<'_>,
) -> Vec<SuppressedFinding> {
    if include_suppressed {
        apply(input)
    } else {
        Vec::new()
    }
}

pub(super) fn apply(input: Inputs<'_>) -> Vec<SuppressedFinding> {
    let Inputs {
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
    let mut suppressed = Vec::new();
    suppress_react(
        root,
        sources,
        react,
        react_suppression_targets,
        &mut suppressed,
    );
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
    suppressed.extend(suppress_domain_findings_with_sources(
        root,
        codebase,
        sources,
        |finding| SuppressionTarget {
            domain: "codebase",
            rule: &finding.rule,
            file: &finding.file,
            line: Some(finding.line as usize),
            reason: &finding.message,
            identity: None,
        },
    ));
    suppressed.sort();
    suppressed
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
