use super::{no_poll_wait_job, pr_diff_range, SourceStore};
use crate::codebase::rules::{
    analysis_policy::AnalysisDiagnostics, github_actions_docker_host_ports,
    github_actions_vitest_project_selection, RuleFinding,
};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

type Check = fn(
    &Path,
    &NoMistakesConfig,
    &[PathBuf],
    &SourceStore,
    bool,
) -> anyhow::Result<AnalysisDiagnostics>;

fn fixture(mode: &str) -> (PathBuf, NoMistakesConfig) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("github-actions-invalid-yaml"),
    );
    let path = root.parent().unwrap().join(format!("configs/{mode}.yml"));
    let config = serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    (root, config)
}

#[test]
fn parsed_workflow_checks_apply_or_defer_file_suppression() {
    let (root, config) = fixture("advise");
    let files = ["control", "broken", "suppressed"]
        .map(|file| root.join(format!(".github/workflows/{file}.yml")));
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let checks: [Check; 4] = [
        github_actions_docker_host_ports::diagnostics_with_files_and_sources,
        github_actions_vitest_project_selection::diagnostics_with_files_and_sources,
        no_poll_wait_job::diagnostics_with_files_and_sources,
        pr_diff_range::diagnostics_with_files_and_sources,
    ];
    for check in checks {
        let deferred = check(&root, &config, &files, &sources, true).unwrap();
        assert_eq!(deferred.findings.len(), 1);
        assert_eq!(deferred.advisories.len(), 2);
        assert!(deferred
            .advisories
            .iter()
            .any(|row| row.file.ends_with("/suppressed.yml")));

        let suppressed = check(&root, &config, &files, &sources, false).unwrap();
        assert_eq!(suppressed.findings, deferred.findings);
        assert_eq!(suppressed.advisories.len(), 1);
        assert_eq!(
            suppressed.advisories[0].file,
            ".github/workflows/broken.yml"
        );
    }
}

type StandaloneCheck = fn(&Path, &NoMistakesConfig) -> anyhow::Result<Vec<RuleFinding>>;

#[test]
fn standalone_docker_and_vitest_checks_discover_and_report_selected_yaml() {
    let (root, config) = fixture("report");
    let checks: [(StandaloneCheck, &str, bool); 2] = [
        (
            github_actions_docker_host_ports::check,
            github_actions_docker_host_ports::RULE_ID,
            true,
        ),
        (
            github_actions_vitest_project_selection::check,
            github_actions_vitest_project_selection::RULE_ID,
            false,
        ),
    ];
    for (check, rule_id, includes_other_project) in checks {
        let findings = check(&root, &config).unwrap();
        let file_finding = |file: &str| findings.iter().find(|finding| finding.file == file);
        assert!(findings.iter().all(|finding| finding.rule == rule_id));
        let broken = file_finding(".github/workflows/broken.yml").unwrap();
        assert_eq!(broken.target.as_deref(), Some("unanalyzable"));
        assert_eq!(broken.line, 3);
        let control = file_finding(".github/workflows/control.yml").unwrap();
        assert_ne!(control.target.as_deref(), Some("unanalyzable"));
        assert!(file_finding(".github/workflows/outside.yml").is_none());
        assert_eq!(
            file_finding(".github/workflows/other-project.yml").is_some(),
            includes_other_project
        );
    }
}
