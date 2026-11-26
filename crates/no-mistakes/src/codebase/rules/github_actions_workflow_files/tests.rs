use super::{no_poll_wait_job, pr_diff_range, SourceStore};
use crate::codebase::rules::{
    analysis_policy::AnalysisDiagnostics, github_actions_docker_host_ports,
    github_actions_vitest_project_selection,
};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

type Check =
    fn(&Path, &NoMistakesConfig, &[PathBuf], &SourceStore, bool) -> anyhow::Result<AnalysisDiagnostics>;

#[test]
fn parsed_workflow_checks_apply_or_defer_file_suppression() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("github-actions-invalid-yaml"),
    );
    let config: NoMistakesConfig = serde_yaml::from_str(
        &std::fs::read_to_string(root.parent().unwrap().join("configs/advise.yml")).unwrap(),
    )
    .unwrap();
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
        assert!(deferred.advisories.iter().any(|row| row.file.ends_with("/suppressed.yml")));

        let suppressed = check(&root, &config, &files, &sources, false).unwrap();
        assert_eq!(suppressed.findings, deferred.findings);
        assert_eq!(suppressed.advisories.len(), 1);
        assert_eq!(suppressed.advisories[0].file, ".github/workflows/broken.yml");
    }
}
