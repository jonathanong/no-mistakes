use super::*;
use crate::codebase::rules::{analysis_policy, source_store_for_files};
use crate::config::v2::{Effort, NoMistakesConfig, Unanalyzable};

fn saved_config(root: &Path, name: &str) -> NoMistakesConfig {
    serde_yaml::from_str(&std::fs::read_to_string(root.join(name)).unwrap()).unwrap()
}

fn identity(finding: &RuleFinding) -> String {
    format!(
        "{}:{}:{}",
        Path::new(&finding.file)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        finding.target.as_deref().unwrap(),
        finding
            .message
            .split("step ")
            .nth(1)
            .unwrap()
            .split(':')
            .next()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
    )
}

pub(super) fn assert_legacy_abstain(findings: &[RuleFinding]) {
    // These old abstention cases are declared write bounds, not unknown grants.
    let mut actual: Vec<_> = findings.iter().map(identity).collect();
    actual.sort();
    assert_eq!(
        actual,
        [
            "callable-job-write.yml:build:1",
            "callable-write.yml:build:1"
        ]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("declared permissions may allow")));
}

#[test]
fn saved_bounds_stay_definite_across_policies_efforts_and_suppressions() {
    let root = super::tests::fixture_root("permission-bounds");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let expected = [
        "job-write.yml:write:1",
        "job-write.yml:write:2",
        "reusable-job-write.yml:write:1",
        "reusable-job-write.yml:write:2",
        "reusable-workflow-write.yml:inherited:1",
        "reusable-workflow-write.yml:inherited:2",
        "reusable-write-all.yml:inherited:1",
        "reusable-write-all.yml:inherited:2",
        "suppression.yml:write:3",
    ];
    for effort in [Effort::Max, Effort::Low] {
        for policy in [
            Unanalyzable::Report,
            Unanalyzable::Advise,
            Unanalyzable::Ignore,
        ] {
            let suffix = match policy {
                Unanalyzable::Report => "report",
                Unanalyzable::Advise => "advise",
                Unanalyzable::Ignore => "ignore",
            };
            let name = match effort {
                Effort::Max if policy == Unanalyzable::Report => ".no-mistakes.yml".to_string(),
                Effort::Max => format!(".no-mistakes.max-{suffix}.yml"),
                Effort::Low => format!(".no-mistakes.low-{suffix}.yml"),
            };
            let mut config = saved_config(&root, &name);
            let sources = source_store_for_files(&files);
            analysis_policy::prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
            assert_eq!(
                config.analysis_warnings.len(),
                usize::from(effort == Effort::Low)
            );
            let output =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                    .unwrap();
            let mut ordinary: Vec<_> = output
                .findings
                .iter()
                .filter(|finding| !finding.message.contains("cannot analyze"))
                .map(identity)
                .collect();
            ordinary.sort();
            assert_eq!(ordinary, expected, "{:?}", output.findings);
            assert_eq!(
                output.findings.len(),
                9 + 2 * usize::from(policy == Unanalyzable::Report)
            );
            assert_eq!(
                output.advisories.len(),
                2 * usize::from(policy == Unanalyzable::Advise)
            );
            for finding in output.findings.iter().chain(&output.advisories) {
                let text = std::fs::read_to_string(root.join(&finding.file)).unwrap();
                assert!(text
                    .lines()
                    .nth(finding.line - 1)
                    .unwrap()
                    .contains("uses: actions/checkout"));
                if finding.message.contains("cannot analyze") {
                    assert!(finding.file.ends_with("uncertain-inputs.yml"));
                } else if finding.file.ends_with("job-write.yml")
                    && !finding.file.ends_with("reusable-job-write.yml")
                {
                    assert!(finding.message.contains("the GITHUB_TOKEN can write"));
                } else {
                    assert!(finding.message.contains("declared permissions may allow"));
                }
            }
            // File/line/next-line suppression filters definite bounds too; raw audit keeps them.
            let raw =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
            assert_eq!(
                raw.findings.len(),
                12 + 2 * usize::from(policy == Unanalyzable::Report)
            );
            assert_eq!(raw.advisories.len(), output.advisories.len());
            assert_eq!(sources.physical_read_count(), 11);
        }
    }
}

#[test]
fn declared_bound_labels_do_not_replace_unspecified_write_assumption_labels() {
    let root = super::tests::fixture_root("assume-unspecified");
    for (name, assumed) in [(".no-mistakes.yml", true), (".no-mistakes.off.yml", false)] {
        let findings = super::check(&root, &saved_config(&root, name)).unwrap();
        let bound = findings
            .iter()
            .find(|finding| finding.file.ends_with("callable-write.yml"))
            .unwrap();
        assert!(bound.message.contains("declared permissions may allow"));
        assert!(!bound.message.contains("assumeWriteWhenUnspecified"));
        let missing = findings
            .iter()
            .find(|finding| finding.file.ends_with("callable.yml"));
        assert_eq!(missing.is_some(), assumed);
        if let Some(missing) = missing {
            assert!(missing.message.contains("the GITHUB_TOKEN can write"));
            assert!(missing.message.contains("assumeWriteWhenUnspecified"));
            assert!(!missing.message.contains("declared permissions may allow"));
        }
    }
}
