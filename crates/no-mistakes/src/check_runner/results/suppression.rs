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

pub(super) struct Inputs<'a> {
    pub(super) root: &'a std::path::Path,
    pub(super) sources: &'a SourceStore,
    pub(super) react: &'a mut Vec<react_traits::Violation>,
    pub(super) queues: &'a mut Vec<CheckFinding>,
    pub(super) rules: &'a mut Vec<RuleFinding>,
    pub(super) rule_suppression_sources: &'a [Option<String>],
    pub(super) filesystem: &'a mut Vec<RuleFinding>,
    pub(super) integration: &'a mut Vec<IntegrationFinding>,
    pub(super) codebase: &'a mut Vec<UniqueExportFinding>,
    pub(super) advisories: &'a mut Vec<RuleFinding>,
}

pub(super) fn apply(input: Inputs<'_>) -> Vec<SuppressedFinding> {
    let Inputs {
        root,
        sources,
        react,
        queues,
        rules,
        rule_suppression_sources,
        filesystem,
        integration,
        codebase,
        advisories,
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

/// A component-level React diagnostic covers every local fetch. Preserve its
/// single stable public finding unless all of those fetches are suppressed.
struct ReactSuppressionFinding {
    finding: react_traits::Violation,
    identity: String,
}

pub(super) fn suppress_react(
    root: &std::path::Path,
    sources: &SourceStore,
    findings: &mut Vec<react_traits::Violation>,
    suppressed: &mut Vec<SuppressedFinding>,
) {
    findings.retain(|finding| {
        let identity = format!("{}@{}", finding.component, finding.file);
        let mut locations = if !finding.suppression_targets.is_empty() {
            finding
                .suppression_targets
                .iter()
                .map(|target| ReactSuppressionFinding {
                    finding: react_traits::Violation {
                        file: target.file.clone(),
                        line: Some(target.line),
                        suppression_lines: Vec::new(),
                        suppression_targets: Vec::new(),
                        ..finding.clone()
                    },
                    identity: identity.clone(),
                })
                .collect()
        } else if !finding.suppression_lines.is_empty() {
            finding
                .suppression_lines
                .iter()
                .map(|line| ReactSuppressionFinding {
                    finding: react_traits::Violation {
                        line: Some(*line),
                        suppression_lines: Vec::new(),
                        suppression_targets: Vec::new(),
                        ..finding.clone()
                    },
                    identity: identity.clone(),
                })
                .collect()
        } else {
            vec![ReactSuppressionFinding {
                finding: react_traits::Violation {
                    suppression_lines: Vec::new(),
                    suppression_targets: Vec::new(),
                    ..finding.clone()
                },
                identity,
            }]
        };
        suppressed.extend(suppress_domain_findings_with_sources(
            root,
            &mut locations,
            sources,
            react_target,
        ));
        !locations.is_empty()
    });
}

fn react_target(entry: &ReactSuppressionFinding) -> SuppressionTarget<'_> {
    let finding = &entry.finding;
    SuppressionTarget {
        domain: "react",
        rule: &finding.rule,
        file: &finding.file,
        line: finding.line,
        reason: finding
            .detail
            .as_deref()
            .unwrap_or("component fetch assertion failed"),
        identity: Some(&entry.identity),
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
