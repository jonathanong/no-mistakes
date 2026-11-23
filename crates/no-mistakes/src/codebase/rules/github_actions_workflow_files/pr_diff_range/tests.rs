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
