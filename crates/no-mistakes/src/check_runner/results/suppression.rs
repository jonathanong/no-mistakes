use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::rules::{
    suppress_domain_findings_with_sources, SuppressedFinding, SuppressionTarget,
};
use no_mistakes::codebase::ts_source::SourceStore;
use no_mistakes::codebase::unique_exports::UniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;

pub(super) struct Inputs<'a> {
    pub(super) root: &'a std::path::Path,
    pub(super) sources: &'a SourceStore,
    pub(super) react: &'a mut Vec<react_traits::Violation>,
    pub(super) queues: &'a mut Vec<CheckFinding>,
    pub(super) rules: &'a mut Vec<RuleFinding>,
    pub(super) filesystem: &'a mut Vec<RuleFinding>,
    pub(super) integration: &'a mut Vec<IntegrationFinding>,
    pub(super) codebase: &'a mut Vec<UniqueExportFinding>,
}

pub(super) fn apply(input: Inputs<'_>) -> Vec<SuppressedFinding> {
    let Inputs {
        root,
        sources,
        react,
        queues,
        rules,
        filesystem,
        integration,
        codebase,
    } = input;
    let mut suppressed = Vec::new();
    suppress_react(root, sources, react, &mut suppressed);
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
        },
    ));
    suppress_rules(root, sources, rules, "rules", &mut suppressed);
    suppress_rules(root, sources, filesystem, "filesystem", &mut suppressed);
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
        },
    ));
    suppressed.sort();
    suppressed.dedup();
    suppressed
}

/// A component-level React diagnostic covers every local fetch. Preserve its
/// single stable public finding unless all of those fetches are suppressed.
fn suppress_react(
    root: &std::path::Path,
    sources: &SourceStore,
    findings: &mut Vec<react_traits::Violation>,
    suppressed: &mut Vec<SuppressedFinding>,
) {
    findings.retain(|finding| {
        let lines = if finding.suppression_lines.is_empty() {
            vec![finding.line]
        } else {
            finding
                .suppression_lines
                .iter()
                .copied()
                .map(Some)
                .collect()
        };
        let mut locations = lines
            .into_iter()
            .map(|line| react_traits::Violation {
                line,
                suppression_lines: Vec::new(),
                ..finding.clone()
            })
            .collect::<Vec<_>>();
        suppressed.extend(suppress_domain_findings_with_sources(
            root,
            &mut locations,
            sources,
            react_target,
        ));
        !locations.is_empty()
    });
}

fn react_target(finding: &react_traits::Violation) -> SuppressionTarget<'_> {
    SuppressionTarget {
        domain: "react",
        rule: &finding.rule,
        file: &finding.file,
        line: finding.line,
        reason: finding
            .detail
            .as_deref()
            .unwrap_or("component fetch assertion failed"),
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
        },
    ));
}
