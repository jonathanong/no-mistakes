use super::loops::poll_loops;
use no_mistakes_rule_kernel::finding::RuleFinding;

fn polls() -> Vec<Vec<String>> {
    vec![vec!["gh".into(), "run".into(), "view".into()]]
}

fn lines(script: &str, skip_for: bool) -> Vec<usize> {
    poll_loops(script, &polls(), skip_for)
        .into_iter()
        .map(|hit| hit.line)
        .collect()
}

#[test]
fn until_condition_poll_with_sleep_body_is_reported() {
    let script = "until gh run view 123 --json status; do sleep 30; done";
    assert_eq!(lines(script, false), vec![1]);
}

#[test]
fn command_substitution_poll_in_a_while_loop_is_reported() {
    let script =
        "echo start\nwhile [ \"$(gh run view 1 -q .status)\" != completed ]; do\n  sleep 5\ndone";
    assert_eq!(lines(script, false), vec![2]);
}

#[test]
fn body_poll_in_a_for_loop_is_reported_unless_for_loops_are_ignored() {
    let script = "for i in 1 2 3; do\n  gh run view 1 || true\n  sleep 5\ndone";
    assert_eq!(lines(script, false), vec![1]);
    assert!(lines(script, true).is_empty());
}

#[test]
fn nested_loop_reports_each_loop_that_holds_both() {
    let script = "while true; do\n  until gh run view 1; do sleep 1; done\ndone";
    assert_eq!(lines(script, false), vec![2]);
}

#[test]
fn abstains_without_a_sleep_a_poll_or_a_loop() {
    assert!(lines("sleep 30\ngh run view 1", false).is_empty());
    assert!(lines("while true; do sleep 1; done", false).is_empty());
    assert!(lines("until gh run view 1; do echo wait; done", false).is_empty());
    assert!(lines("echo gh run view 1; sleep 3", false).is_empty());
}

#[test]
fn abstains_on_unbalanced_scripts_and_heredocs() {
    assert!(lines("until gh run view 1; do sleep 1", false).is_empty());
    assert!(lines("done\nuntil gh run view 1; do sleep 1; done", false).is_empty());
    let heredoc = "cat <<EOF\nuntil gh run view 1; do sleep 1; done\nEOF";
    assert!(lines(heredoc, false).is_empty());
}

#[test]
fn unconfigured_commands_are_not_reported() {
    let script = "until curl -fs https://example.invalid/ready; do sleep 2; done";
    assert!(lines(script, false).is_empty());
}

#[test]
fn loops_are_ordered_and_prefixes_do_not_hide_polls() {
    let two = "until gh run view 1; do sleep 1; done\nuntil gh run view 2; do sleep 1; done";
    assert_eq!(lines(two, false), vec![1, 2]);
    assert_eq!(
        lines("until FOO=1 gh run view 1; do sleep 1; done", false),
        vec![1]
    );
    assert_eq!(
        lines("until `gh run view 1`; do sleep 1; done", false),
        vec![1]
    );
    assert_eq!(
        lines("while s=$(gh run view 1); do sleep 1; done", false),
        vec![1]
    );
    assert!(lines("until echo gh run view 1; do sleep 1; done", false).is_empty());
}

fn fixture(name: &str, config: &str) -> Result<Vec<RuleFinding>, anyhow::Error> {
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-no-poll-wait-job");
    let root = crate::codebase::ts_resolver::normalize_path(&base.join("fixture").join(name));
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(base.join("configs").join(config)).unwrap())
            .unwrap();
    let names: &[&str] = match name {
        "report" => &[
            ".github/actions/wait/action.yml",
            ".github/workflows/wait.yml",
        ],
        "pass" => &[".github/workflows/pass.yml"],
        _ => &[".github/workflows/retry.yml"],
    };
    let files: Vec<_> = names.iter().map(|file| root.join(file)).collect();
    super::check_with_files(&root, &config, &files)
}

#[test]
fn saved_fixtures_report_only_configured_waiting_loops() {
    assert_eq!(fixture("report", "report.yml").unwrap().len(), 4);
    assert!(fixture("pass", "report.yml").unwrap().is_empty());
    assert!(fixture("report", "unconfigured.yml").unwrap().is_empty());
    assert_eq!(fixture("bounded", "report.yml").unwrap().len(), 1);
    assert!(fixture("bounded", "bounded.yml").unwrap().is_empty());
}

#[test]
fn blank_poll_commands_are_configuration_errors() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: super::RULE_ID.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        options: serde_yaml::from_str("pollCommands: ['  ']").unwrap(),
        ..Default::default()
    });
    let error = super::check_with_files(std::path::Path::new("."), &config, &[]).unwrap_err();
    assert!(error.to_string().contains("must name a command"), "{error}");
}

#[test]
fn unplaceable_and_folded_scripts_still_report_a_line() {
    let polls = polls();
    let flow = "jobs: {a: {steps: [{run: \"until gh run view 1; do sleep 1; done\"}]}}\n";
    let found = check_source(".github/workflows/ci.yml", flow, &polls, false);
    assert_eq!(found.iter().map(|hit| hit.line).collect::<Vec<_>>(), [1]);
    let folded = "jobs:\n  a:\n    steps:\n      - run: >\n          until gh run view 1;\n          do sleep 1; done\n";
    let found = check_source(".github/workflows/ci.yml", folded, &polls, false);
    assert_eq!(found.len(), 1);
    assert!(found[0].line >= 4, "{}", found[0].line);
}

fn check_source(
    rel: &str,
    source: &str,
    polls: &[Vec<String>],
    skip_for: bool,
) -> Vec<RuleFinding> {
    let Some(action) = super::super::gha_kind(rel) else {
        return Vec::new();
    };
    let source = source.trim_start_matches('\u{feff}');
    let Ok(value) = serde_yaml::from_str(source) else {
        return Vec::new();
    };
    super::runs::check_parsed(rel, source, &value, action, polls, skip_for)
}

#[test]
fn empty_poll_commands_still_diagnose_invalid_selected_yaml() {
    use crate::config::v2::{NoMistakesConfig, Unanalyzable};

    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("github-actions-invalid-yaml"),
    );
    let config_path = crate::test_support::rule_fixture_root(super::RULE_ID)
        .parent()
        .unwrap()
        .join("configs/unconfigured.yml");
    let mut config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();
    config.rules[0].options = serde_yaml::from_str("pollCommands: []").unwrap();
    let files = ["broken.yml", "control.yml"].map(|name| root.join(".github/workflows").join(name));
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for (policy, findings, advisories) in
        [(Unanalyzable::Report, 1, 0), (Unanalyzable::Advise, 0, 1)]
    {
        config.unanalyzable = Some(policy);
        let output =
            super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                .unwrap();
        assert_eq!(
            (output.findings.len(), output.advisories.len()),
            (findings, advisories)
        );
        for row in output.findings.iter().chain(&output.advisories) {
            assert_eq!(row.file, ".github/workflows/broken.yml");
            assert_eq!((row.line, row.target.as_deref()), (3, Some("unanalyzable")));
        }
    }
}
