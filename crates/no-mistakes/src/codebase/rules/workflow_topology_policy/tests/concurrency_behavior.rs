use super::super::concurrency_scope::{self, ScopeClass};
use super::support::{self, concurrency};
use crate::codebase::rules::sort_findings;
use crate::codebase::workflow_topology::model::{ConcurrencyValue, WorkflowTopology};

fn messages(topology: &WorkflowTopology, yaml: &str) -> Vec<String> {
    let opts: super::super::Options = super::config(yaml).rules[0].try_rule_options().unwrap();
    let policy = super::super::concurrency_compile::compile(&opts.concurrency_policy).unwrap();
    let mut findings = super::super::evaluate_concurrency::lint(topology, &policy);
    sort_findings(&mut findings);
    findings
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

fn workflow_lock(
    path: &str,
    group: &str,
    cancel: ConcurrencyValue,
    queue: &str,
) -> WorkflowTopology {
    let lock = concurrency(group, cancel, queue);
    support::topology(
        vec![support::workflow(path, Some(lock), &["build"])],
        vec![support::job(path, "build", None)],
    )
}

#[test]
fn classifies_every_scope_table_row() {
    let cases = [
        (
            "github.event.pull_request.number",
            ScopeClass::Known("pull-request"),
        ),
        (
            "github.event.workflow_run.pull_requests",
            ScopeClass::Known("pull-request"),
        ),
        (
            "github.event.workflow_run.pull_requests.number",
            ScopeClass::Known("pull-request"),
        ),
        ("github.sha", ScopeClass::Known("sha")),
        (
            "github.event.pull_request.head.sha",
            ScopeClass::Known("sha"),
        ),
        (
            "github.event.workflow_run.head_sha",
            ScopeClass::Known("sha"),
        ),
        ("github.ref", ScopeClass::Known("ref")),
        ("github.ref_name", ScopeClass::Known("ref")),
        (
            "github.event.workflow_run.head_branch",
            ScopeClass::Known("ref"),
        ),
        ("github.run_id", ScopeClass::Known("run")),
        ("inputs.service", ScopeClass::Known("input-resource")),
        ("github.event.inputs", ScopeClass::Known("input-resource")),
        (
            "github.event.inputs.name",
            ScopeClass::Known("input-resource"),
        ),
        ("github.event_name", ScopeClass::Known("event")),
        ("github.event.action", ScopeClass::Known("event")),
        ("github.event.label.name", ScopeClass::Known("event")),
        ("github.event.issue.number", ScopeClass::Known("event")),
        ("github.event.schedule", ScopeClass::Known("event")),
        (
            "github.event.workflow_run.event",
            ScopeClass::Known("event"),
        ),
        ("github.event.workflow_run.id", ScopeClass::Known("event")),
        (
            "github.event.workflow_run.workflow_id",
            ScopeClass::Known("event"),
        ),
        ("github.workflow", ScopeClass::Ignored),
        ("github.run_attempt", ScopeClass::Ignored),
        ("github.actor", ScopeClass::Unsupported),
        ("github.repository_owner", ScopeClass::Unsupported),
    ];
    for (reference, expected) in cases {
        assert_eq!(
            concurrency_scope::classify(reference),
            expected,
            "{reference}"
        );
    }
}

#[test]
fn normalizes_group_references_by_the_scope_table() {
    let overlapping = "${{ github.event.pull_request.number || github.event.label.name || github.event.inputs || inputs.service || github.event.workflow_run.head_sha || github.event.workflow_run.id || github.workflow || github.run_attempt }}";
    assert_eq!(
        concurrency_scope::actual_scope(overlapping),
        vec![
            "pull-request".to_string(),
            "sha".to_string(),
            "event".to_string(),
            "input-resource".to_string()
        ]
    );
    assert_eq!(
        concurrency_scope::actual_scope(
            "fix-${{ github.event.workflow_run.head_branch }}-${{ github.event.workflow_run.workflow_id }}"
        ),
        vec!["ref".to_string(), "event".to_string()]
    );
    assert_eq!(
        concurrency_scope::actual_scope("${{ github.event.workflow_run.pull_requests[0].number }}"),
        vec!["pull-request".to_string()]
    );
    assert!(concurrency_scope::actual_scope("staging-database").is_empty());
    assert_eq!(
        concurrency_scope::actual_scope(
            "${{ github.repository_owner || github.actor || github.sha }}"
        ),
        vec![
            "sha".to_string(),
            "unsupported:github.actor".to_string(),
            "unsupported:github.repository_owner".to_string()
        ]
    );
}

#[test]
fn reports_the_invalid_intent_examples() {
    let ci = concurrency(
        "ci-${{ github.event.pull_request.number }}-${{ github.event.pull_request.head.sha }}",
        ConcurrencyValue::Bool(true),
        "single",
    );
    let deploy = concurrency(
        "deploy-${{ github.sha }}",
        ConcurrencyValue::Bool(false),
        "single",
    );
    let publish = concurrency(
        "release-${{ github.run_id }}",
        ConcurrencyValue::Bool(false),
        "single",
    );
    let lint = concurrency(
        "lint-${{ github.ref }}",
        ConcurrencyValue::Bool(false),
        "single",
    );
    let topology = support::topology(
        vec![
            support::workflow(".github/workflows/ci.yml", Some(ci), &["build"]),
            support::workflow(".github/workflows/deploy.yml", Some(deploy), &["deploy"]),
            support::workflow(".github/workflows/release.yml", None, &["publish"]),
            support::workflow(".github/workflows/lint.yml", Some(lint), &["lint"]),
            support::workflow(".github/workflows/plain.yml", None, &["job"]),
        ],
        vec![
            support::job(".github/workflows/ci.yml", "build", None),
            support::job(".github/workflows/deploy.yml", "deploy", None),
            support::job(".github/workflows/release.yml", "publish", Some(publish)),
            support::job(".github/workflows/lint.yml", "lint", None),
            support::job(".github/workflows/plain.yml", "job", None),
        ],
    );
    let found = messages(
        &topology,
        r#"
concurrencyPolicy:
  .github/workflows/ci.yml:
    pending: coalesce-latest
    cancellation: conditional
    scope: [pull-request, sha]
  .github/workflows/deploy.yml:
    pending: coalesce-latest
    cancellation: retain-running
    scope: [fixed-resource]
  ".github/workflows/release.yml#publish":
    pending: fifo
    cancellation: retain-running
    scope: [run]
  .github/workflows/gone.yml:
    pending: coalesce-latest
    cancellation: retain-running
    scope: [fixed-resource]
"#,
    );
    assert_eq!(
        found,
        vec![
            "concurrency cancellation mismatch: .github/workflows/ci.yml: expected conditional, got cancel-running".to_string(),
            "concurrency intent missing: .github/workflows/lint.yml".to_string(),
            "concurrency intent stale: .github/workflows/gone.yml".to_string(),
            "concurrency pending mismatch: .github/workflows/release.yml#publish: expected fifo, got coalesce-latest".to_string(),
            "concurrency scope mismatch: .github/workflows/deploy.yml: expected , got sha".to_string(),
        ]
    );
}

#[test]
fn accepts_the_valid_conditional_example() {
    let lock = concurrency(
        "${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}",
        ConcurrencyValue::Text("${{ github.event_name == 'pull_request' }}".to_string()),
        "single",
    );
    let topology = support::topology(
        vec![support::workflow("ci.yml", Some(lock), &["build"])],
        vec![support::job("ci.yml", "build", None)],
    );
    let found = messages(
        &topology,
        r#"
concurrencyPolicy:
  ci.yml:
    pending: coalesce-latest
    cancellation: conditional
    scope: [pull-request, ref]
"#,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn derives_pending_cancellation_and_scope() {
    let fifo = messages(
        &workflow_lock(
            "ci.yml",
            "staging-database",
            ConcurrencyValue::Bool(false),
            "max",
        ),
        r#"
concurrencyPolicy:
  ci.yml:
    pending: fifo
    cancellation: retain-running
    scope: [fixed-resource]
"#,
    );
    assert!(fifo.is_empty(), "{fifo:?}");

    let other_queue = messages(
        &workflow_lock(
            "ci.yml",
            "staging-database",
            ConcurrencyValue::Bool(true),
            "parallel",
        ),
        r#"
concurrencyPolicy:
  ci.yml:
    pending: coalesce-latest
    cancellation: cancel-running
    scope: [fixed-resource]
"#,
    );
    assert!(other_queue.is_empty(), "{other_queue:?}");

    let normalized = messages(
        &workflow_lock(
            "ci.yml",
            "${{ github.sha }}",
            ConcurrencyValue::Bool(false),
            "single",
        ),
        r#"
concurrencyPolicy:
  ci.yml:
    pending: coalesce-latest
    cancellation: retain-running
    scope: [sha, pull-request]
"#,
    );
    assert_eq!(
        normalized,
        vec!["concurrency scope mismatch: ci.yml: expected pull-request, sha, got sha".to_string()]
    );
}

#[test]
fn checks_conditional_cancel_expressions() {
    let invalid = messages(
        &workflow_lock(
            "ci.yml",
            "staging-database",
            ConcurrencyValue::Text("github.ref == main".to_string()),
            "single",
        ),
        r#"
concurrencyPolicy:
  ci.yml:
    pending: coalesce-latest
    cancellation: cancel-running
    scope: [fixed-resource]
"#,
    );
    assert_eq!(
        invalid,
        vec![
            "concurrency cancellation mismatch: ci.yml: expected cancel-running, got conditional"
                .to_string(),
            "conditional cancel-in-progress expression invalid: ci.yml: github.ref == main"
                .to_string(),
        ]
    );

    let trimmed = messages(
        &workflow_lock(
            "ci.yml",
            "staging-database",
            ConcurrencyValue::Text("  ${{ github.ref }}  ".to_string()),
            "single",
        ),
        r#"
concurrencyPolicy:
  ci.yml:
    pending: coalesce-latest
    cancellation: conditional
    scope: [fixed-resource]
"#,
    );
    assert!(trimmed.is_empty(), "{trimmed:?}");

    let empty = messages(
        &workflow_lock(
            "ci.yml",
            "staging-database",
            ConcurrencyValue::Text("${{ }}".to_string()),
            "single",
        ),
        r#"
concurrencyPolicy:
  ci.yml:
    pending: coalesce-latest
    cancellation: conditional
    scope: [fixed-resource]
"#,
    );
    assert_eq!(
        empty,
        vec!["conditional cancel-in-progress expression invalid: ci.yml: ${{ }}".to_string()]
    );
}

#[test]
fn workflow_and_job_locks_are_separate_owners() {
    let workflow = concurrency("literal", ConcurrencyValue::Bool(false), "single");
    let job = concurrency("literal", ConcurrencyValue::Bool(false), "max");
    let topology = support::topology(
        vec![support::workflow("ci.yml", Some(workflow), &["publish"])],
        vec![support::job("ci.yml", "publish", Some(job))],
    );
    let found = messages(
        &topology,
        r#"
concurrencyPolicy:
  ci.yml:
    pending: coalesce-latest
    cancellation: retain-running
    scope: [fixed-resource]
  "ci.yml#publish":
    pending: coalesce-latest
    cancellation: retain-running
    scope: [fixed-resource]
"#,
    );
    assert_eq!(
        found,
        vec![
            "concurrency pending mismatch: ci.yml#publish: expected coalesce-latest, got fifo"
                .to_string()
        ]
    );
}

#[test]
fn empty_policy_does_not_report_owners() {
    let topology = workflow_lock("ci.yml", "literal", ConcurrencyValue::Bool(true), "max");
    let opts: super::super::Options = super::config("concurrencyPolicy: {}\n").rules[0]
        .try_rule_options()
        .unwrap();
    let policy = super::super::concurrency_compile::compile(&opts.concurrency_policy).unwrap();
    assert!(policy.is_empty());
    assert!(super::super::evaluate_concurrency::lint(&topology, &policy).is_empty());
}

#[test]
fn two_runs_produce_the_same_findings() {
    let topology = workflow_lock(
        "ci.yml",
        "${{ github.actor }}",
        ConcurrencyValue::Bool(true),
        "single",
    );
    let yaml = r#"
concurrencyPolicy:
  ci.yml:
    pending: coalesce-latest
    cancellation: retain-running
    scope: [fixed-resource]
"#;
    assert_eq!(messages(&topology, yaml), messages(&topology, yaml));
}

#[test]
fn stale_row_for_a_job_without_concurrency() {
    let topology = support::topology(
        vec![support::workflow("ci.yml", None, &["build"])],
        vec![support::job("ci.yml", "build", None)],
    );
    let found = messages(
        &topology,
        r#"
concurrencyPolicy:
  "ci.yml#build":
    pending: coalesce-latest
    cancellation: retain-running
    scope: [fixed-resource]
"#,
    );
    assert_eq!(
        found,
        vec!["concurrency intent stale: ci.yml#build".to_string()]
    );
}
