use super::{collect, shell::rerun_loops, Shape};
use crate::codebase::rules::source_store_for_files;
use std::path::PathBuf;

fn fixture(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-reliability")
        .join(relative)
}

fn test_commands() -> Vec<Vec<String>> {
    vec![vec!["pnpm".into(), "test:unit".into()]]
}

#[test]
fn workflow_shell_precedence_honors_an_explicit_bash_on_windows() {
    let path = fixture("workflows/explicit-shells.yml");
    let sources = source_store_for_files(std::slice::from_ref(&path));
    let findings = collect(&path, &sources, &test_commands()).unwrap();
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [9, 23]);
}

#[test]
fn workflow_default_shell_uses_runner_labels() {
    let path = fixture("workflows/runner-labels.yml");
    let sources = source_store_for_files(std::slice::from_ref(&path));
    let findings = collect(&path, &sources, &test_commands()).unwrap();
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [1, 22, 27, 33]);
}

#[test]
fn retry_actions_distinguish_defaults_explicit_attempts_and_unknown_values() {
    let path = fixture("workflows/action-attempts.yml");
    let sources = source_store_for_files(std::slice::from_ref(&path));
    let findings = collect(&path, &sources, &[]).unwrap();
    let retries: Vec<_> = findings
        .iter()
        .filter_map(|finding| match finding.shape {
            Shape::Retry("GitHub Actions retry action", count) => Some((finding.line, count)),
            _ => None,
        })
        .collect();
    assert_eq!(
        retries,
        [
            (8, Some(2)),
            (9, Some(1)),
            (12, Some(1)),
            (13, None),
            (16, None),
            (19, Some(0)),
            (22, None),
        ]
    );
}

#[test]
fn a_document_without_a_jobs_mapping_has_no_workflow_steps() {
    let path = fixture("workflows/not-a-workflow.yml");
    let sources = source_store_for_files(std::slice::from_ref(&path));
    assert!(collect(&path, &sources, &test_commands())
        .unwrap()
        .is_empty());
}

#[test]
fn shell_counts_literal_iterations_and_reports_dynamic_bounds() {
    let source = std::fs::read_to_string(fixture("shell/loop-bounds.sh")).unwrap();
    assert_eq!(
        rerun_loops(&source, &test_commands()),
        [
            Some(3),
            None,
            None,
            None,
            Some(1),
            None,
            None,
            None,
            Some(1),
            None
        ]
    );
}

#[test]
fn shell_skips_here_documents_and_resumes_after_the_final_delimiter() {
    let source = std::fs::read_to_string(fixture("shell/here-documents.sh")).unwrap();
    assert_eq!(
        rerun_loops(&source, &test_commands()),
        [Some(1), Some(2), Some(3), Some(4)]
    );
}

#[test]
fn incomplete_shell_keeps_the_findings_from_complete_preceding_loops() {
    let source = std::fs::read_to_string(fixture("shell/incomplete-snippets.yml")).unwrap();
    let snippets: std::collections::BTreeMap<String, String> =
        serde_yaml::from_str(&source).unwrap();
    assert!(!snippets.is_empty());
    for (name, script) in snippets {
        assert_eq!(rerun_loops(&script, &test_commands()), [Some(1)], "{name}");
    }
}

#[test]
fn shell_splits_or_and_while_without_variables() {
    let script = "for i in 1 2; do\n  pnpm test:unit || true\ndone\nwhile true; do\n  echo \"try\" && pnpm test:unit\ndone\n";
    assert_eq!(rerun_loops(script, &test_commands()), [Some(1), None]);
}

#[test]
fn shell_detects_retry_signals_and_enforces_setup_order() {
    let source = std::fs::read_to_string(fixture("shell/retry-signals.sh")).unwrap();
    assert_eq!(
        rerun_loops(&source, &test_commands()),
        [
            Some(2),
            Some(2),
            Some(2),
            Some(2),
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            None,
            Some(1),
            Some(2),
            Some(1),
            Some(1),
        ]
    );
}
