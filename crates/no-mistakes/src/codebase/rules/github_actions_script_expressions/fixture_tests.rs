use super::targets;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

fn fixture_root(scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-script-expressions/fixture")
            .join(scenario),
    )
}

fn fixture_files(root: &Path, scenario: &str) -> Vec<PathBuf> {
    let names: &[&str] = match scenario {
        "pass" => &[
            ".github/actions/image/action.yml",
            ".github/actions/pack/action.yml",
            ".github/workflows/ci.yml",
            ".no-mistakes.yml",
        ],
        "fail" => &[
            ".github/actions/pack/action.yml",
            ".github/workflows/ci.yml",
            ".no-mistakes.yml",
        ],
        "allowlist" => &[".github/workflows/ci.yml", ".no-mistakes.yml"],
        "unanalyzable" => &[
            ".github/workflows/alias.yml",
            ".github/workflows/flow.yml",
            ".no-mistakes.yml",
        ],
        other => panic!("unknown fixture scenario {other}"),
    };
    names.iter().map(|name| root.join(name)).collect()
}

fn load_fixture(scenario: &str) -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root = fixture_root(scenario);
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files = fixture_files(&root, scenario);
    (root, config, files)
}

#[test]
fn saved_fixtures_match_allowed_flagged_allowlist_and_suppression() {
    let (root, config, files) = load_fixture("pass");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");

    let (root, config, files) = load_fixture("fail");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    let mut got = targets(&findings);
    got.sort_unstable();
    assert_eq!(
        got,
        vec![
            "env.FOO",
            "github.actor",
            "github.actor",
            "github.actor",
            "github.event.head_commit.message",
            "github.event.issue.title",
            "github.ref",
            "github.sha || 'unknown'",
            "inputs.name",
            "inputs.name",
            "job.services.redis.ports[6379]",
            "secrets.TOKEN",
            "steps.id.outputs.value",
            "vars.FOO",
        ]
    );
    assert!(findings
        .iter()
        .any(|finding| finding.file.ends_with("action.yml")));
    assert!(findings
        .iter()
        .any(|finding| finding.file.ends_with("ci.yml")));

    let (root, config, files) = load_fixture("allowlist");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(targets(&findings), vec!["github.sha"]);

    let root = fixture_root("suppression");
    let findings = crate::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert_eq!(
        targets(&findings),
        vec!["github.event.head_commit.message"],
        "{findings:?}"
    );

    let (root, config, files) = load_fixture("unanalyzable");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.len() == 2
            && findings
                .iter()
                .all(|finding| finding.message.contains("aligned")),
        "{findings:?}"
    );
}
