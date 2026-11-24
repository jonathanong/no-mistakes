use super::runs::check_source;

const BASE: &str = "${{ github.event.pull_request.base.sha }}";
const HEAD: &str = "${{ github.event.pull_request.head.sha }}";

fn workflow(on: &str, run: &str) -> String {
    format!(
        "on: {on}\njobs:\n  changes:\n    runs-on: ubuntu-latest\n    steps:\n      - run: {run}\n"
    )
}

fn hits(on: &str, run: &str) -> usize {
    check_source(".github/workflows/ci.yml", &workflow(on, run)).len()
}

#[test]
fn two_dot_and_two_argument_base_head_ranges_are_reported() {
    assert_eq!(
        hits(
            "pull_request",
            &format!("git diff --name-only {BASE} {HEAD}")
        ),
        1
    );
    assert_eq!(
        hits("pull_request", &format!("git diff '{BASE}..{HEAD}'")),
        1
    );
    assert_eq!(
        hits(
            "[pull_request]",
            &format!("files=$(git diff {BASE} {HEAD})")
        ),
        1
    );
}

#[test]
fn base_paired_with_github_sha_is_reported_in_every_form() {
    for range in [
        format!("{BASE}...${{{{ github.sha }}}}"),
        format!("{BASE}..${{{{ github.sha }}}}"),
        "origin/${{ github.base_ref }} ${{ github.sha }}".to_string(),
    ] {
        assert_eq!(
            hits("pull_request", &format!("git diff {range}")),
            1,
            "{range}"
        );
    }
}

#[test]
fn merge_base_forms_are_not_reported() {
    assert_eq!(
        hits("pull_request", &format!("git diff {BASE}...{HEAD}")),
        0
    );
    assert_eq!(hits("pull_request", "git diff HEAD^1 HEAD"), 0);
    assert_eq!(hits("pull_request", "git diff --stat -- docs"), 0);
    assert_eq!(hits("pull_request", &format!("git log {BASE} {HEAD}")), 0);
    assert_eq!(
        hits("pull_request", &format!("git diff -U 3 {BASE} {HEAD}")),
        0
    );
}

#[test]
fn merge_group_ranges_and_other_events_are_not_reported() {
    let range = "${{ github.event.merge_group.base_sha }} ${{ github.event.merge_group.head_sha }}";
    assert_eq!(hits("merge_group", &format!("git diff {range}")), 0);
    assert_eq!(hits("push", &format!("git diff {BASE} {HEAD}")), 0);
}

#[test]
fn merge_group_workflows_reading_pull_request_fields_are_reported() {
    assert_eq!(hits("merge_group", &format!("echo {BASE}")), 1);
    assert_eq!(
        hits("[pull_request, merge_group]", &format!("echo {HEAD}")),
        1
    );
    assert_eq!(hits("pull_request", &format!("echo {BASE}")), 0);
}

#[test]
fn guarded_and_env_resolved_forms_follow_the_declared_values() {
    let guarded = format!(
        "on: [merge_group, pull_request]\njobs:\n  a:\n    runs-on: x\n    steps:\n      - if: github.event_name == 'pull_request'\n        run: echo {BASE}\n"
    );
    assert!(check_source(".github/workflows/ci.yml", &guarded).is_empty());
    let env = format!(
        "on: pull_request\njobs:\n  a:\n    runs-on: x\n    env:\n      B: {BASE}\n      H: {HEAD}\n    steps:\n      - run: git diff \"$B\" ${{H}}\n"
    );
    assert_eq!(check_source(".github/workflows/ci.yml", &env).len(), 1);
}

#[test]
fn composite_actions_and_invalid_yaml_are_ignored() {
    let body = workflow("pull_request", &format!("git diff {BASE} {HEAD}"));
    assert!(check_source(".github/actions/x/action.yml", &body).is_empty());
    assert!(check_source(".github/workflows/ci.yml", "on: [").is_empty());
}

const WF: &str = ".github/workflows/ci.yml";

fn count(yaml: &str) -> usize {
    check_source(WF, yaml).len()
}

#[test]
fn malformed_workflow_shapes_are_skipped() {
    let steps =
        "    steps:\n      - nope\n      - uses: example/action\n      - run: git diff a b\n";
    assert_eq!(count("on: pull_request\njobs:\n  a: nope\n"), 0);
    assert_eq!(count(&format!("on: pull_request\njobs:\n  a:\n{steps}")), 0);
    assert_eq!(count("on: pull_request\n"), 0);
    assert_eq!(count("jobs:\n  a:\n    steps:\n      - run: echo\n"), 0);
    assert_eq!(count("on: 5\njobs: {}\n"), 0);
}

#[test]
fn trigger_forms_select_pull_request_judging() {
    let run = format!("git diff {BASE} {HEAD}");
    for on in [
        "pull_request_target",
        "[push, pull_request]",
        "true: pull_request",
    ] {
        let yaml = if on.starts_with("true") {
            format!("{on}\njobs:\n  a:\n    steps:\n      - run: {run}\n")
        } else {
            format!("on: {on}\njobs:\n  a:\n    steps:\n      - run: {run}\n")
        };
        assert_eq!(count(&yaml), 1, "{on}");
    }
    let mapping = format!("on:\n  pull_request:\njobs:\n  a:\n    steps:\n      - run: {run}\n");
    assert_eq!(count(&mapping), 1);
}

#[test]
fn shell_prefixes_and_git_globals_do_not_hide_a_diff() {
    for command in [
        "git -C . diff",
        "git --no-pager diff",
        "if git diff",
        "FOO=1 git diff",
        "sudo git diff",
        "files=$(git diff",
    ] {
        assert_eq!(
            hits("pull_request", &format!("{command} {BASE} {HEAD}")),
            1,
            "{command}"
        );
    }
    assert_eq!(hits("pull_request", &format!("echo {BASE} {HEAD}")), 0);
    assert_eq!(
        hits("pull_request", &format!("git status {BASE} {HEAD}")),
        0
    );
    assert_eq!(hits("pull_request", "git"), 0);
}

#[test]
fn unrecognized_ranges_are_silent() {
    for range in [
        "${{ github.event.pull_request.base.sha",
        "$A $B",
        "$ $1",
        "a b c",
        "a",
        "--stat",
    ] {
        assert_eq!(
            hits("pull_request", &format!("git diff {range}")),
            0,
            "{range}"
        );
    }
}

#[test]
fn merge_group_reads_follow_env_references() {
    let yaml = |run: &str| {
        format!(
            "on: merge_group\nenv:\n  B: {BASE}\n  N: 1\n  U: {HEAD}\njobs:\n  a:\n    steps:\n      - run: {run}\n"
        )
    };
    assert_eq!(count(&yaml("echo \"$B\"")), 1);
    assert_eq!(count(&yaml("echo ${B}")), 1);
    assert_eq!(count(&yaml("echo $N")), 0);
}

#[test]
fn saved_fixtures_report_only_unsound_ranges() {
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-pr-diff-range/fixture");
    let run = |name: &str, files: &[&str]| {
        let root = crate::codebase::ts_resolver::normalize_path(&base.join(name));
        let files: Vec<_> = files.iter().map(|file| root.join(file)).collect();
        let mut config = crate::config::v2::NoMistakesConfig::default();
        config.rules.push(crate::config::v2::schema::RuleDef {
            rule: super::RULE_ID.to_string(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            ..Default::default()
        });
        super::check_with_files(&root, &config, &files)
            .unwrap()
            .len()
    };
    let workflow = |file: &str| format!(".github/workflows/{file}");
    assert_eq!(
        run("report", &[&workflow("pull.yml"), &workflow("queue.yml")]),
        5
    );
    let pass = [
        workflow("pass.yml"),
        workflow("pull-only.yml"),
        workflow("push.yml"),
    ];
    assert_eq!(run("pass", &[&pass[0], &pass[1], &pass[2]]), 0);
}
