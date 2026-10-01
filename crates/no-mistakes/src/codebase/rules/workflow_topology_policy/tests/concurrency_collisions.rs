use super::support::{self, concurrency};
use crate::codebase::rules::run_filesystem_rules;
use crate::codebase::rules::sort_findings;
use crate::codebase::workflow_topology::load_workflow_topology;
use crate::codebase::workflow_topology::model::{ConcurrencyValue, WorkflowTopology};
use crate::codebase::workflow_topology::render_mermaid::render_workflow_topology_mermaid;
use crate::config::v2::schema::CiConfig;

fn collisions(topology: &WorkflowTopology) -> Vec<String> {
    let mut findings = super::super::evaluate_collisions::lint(topology);
    sort_findings(&mut findings);
    findings
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

fn gated(topology: &WorkflowTopology, yaml: &str) -> Vec<String> {
    let opts: super::super::Options = super::config(yaml).rules[0].try_rule_options().unwrap();
    let policy = super::super::concurrency_compile::compile(&opts.concurrency_policy).unwrap();
    let mut findings = super::super::evaluate::lint(topology, &opts, &policy);
    sort_findings(&mut findings);
    findings
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

fn lock(group: &str) -> crate::codebase::workflow_topology::model::WorkflowConcurrency {
    concurrency(group, ConcurrencyValue::Bool(false), "single")
}

fn workflows(groups: &[(&str, &str)]) -> WorkflowTopology {
    let nodes = groups
        .iter()
        .map(|(path, group)| support::workflow(path, Some(lock(group)), &[]))
        .collect();
    support::topology(nodes, Vec::new())
}

fn workflow_and_job(path: &str, workflow_group: &str, job_group: &str) -> WorkflowTopology {
    support::topology(
        vec![support::workflow(
            path,
            Some(lock(workflow_group)),
            &["build"],
        )],
        vec![support::job(path, "build", Some(lock(job_group)))],
    )
}

fn collision(group: &str, ids: &str) -> String {
    format!("concurrency group collision: {group}: {ids}")
}

#[test]
fn identical_literal_groups_collide() {
    let found = collisions(&workflows(&[
        (".github/workflows/one.yml", "shared"),
        (".github/workflows/two.yml", "shared"),
    ]));
    assert_eq!(
        found,
        vec![collision(
            "shared",
            ".github/workflows/one.yml, .github/workflows/two.yml"
        )]
    );
}

#[test]
fn groups_collide_case_insensitively() {
    let found = collisions(&workflows(&[
        (".github/workflows/one.yml", "Deploy-Prod"),
        (".github/workflows/two.yml", "deploy-prod"),
    ]));
    assert_eq!(
        found,
        vec![collision(
            "deploy-prod",
            ".github/workflows/one.yml, .github/workflows/two.yml"
        )]
    );
}

#[test]
fn unicode_letters_lowercase_without_a_locale() {
    let found = collisions(&workflows(&[
        (".github/workflows/one.yml", "Ω-lock"),
        (".github/workflows/two.yml", "ω-lock"),
    ]));
    assert_eq!(
        found,
        vec![collision(
            "ω-lock",
            ".github/workflows/one.yml, .github/workflows/two.yml"
        )]
    );
}

#[test]
fn identical_expression_text_collides_across_files() {
    let found = collisions(&workflows(&[
        (".github/workflows/a.yml", "${{ github.ref }}"),
        (".github/workflows/b.yml", "${{ github.ref }}"),
    ]));
    assert_eq!(
        found,
        vec![collision(
            "${{ github.ref }}",
            ".github/workflows/a.yml, .github/workflows/b.yml"
        )]
    );
}

#[test]
fn github_workflow_placeholder_partitions_by_file() {
    let across_files = collisions(&workflows(&[
        (
            ".github/workflows/a.yml",
            "${{ github.workflow }}-${{ github.ref }}",
        ),
        (
            ".github/workflows/b.yml",
            "${{ github.workflow }}-${{ github.ref }}",
        ),
    ]));
    assert!(across_files.is_empty(), "{across_files:?}");

    let path = ".github/workflows/ci.yml";
    let same_file = collisions(&workflow_and_job(
        path,
        "${{ github.workflow }}-${{ github.ref }}",
        "${{ github.workflow }}-${{ github.ref }}",
    ));
    assert_eq!(
        same_file,
        vec![collision(
            "${{ github.workflow }}-${{ github.ref }}",
            ".github/workflows/ci.yml, .github/workflows/ci.yml#build"
        )]
    );
}

#[test]
fn github_workflow_whitespace_still_partitions() {
    for group in ["${{github.workflow}}", "${{ github.workflow }}"] {
        let across_files = collisions(&workflows(&[
            (".github/workflows/a.yml", group),
            (".github/workflows/b.yml", group),
        ]));
        assert!(across_files.is_empty(), "{group}: {across_files:?}");

        let same_file = collisions(&workflow_and_job(".github/workflows/ci.yml", group, group));
        assert_eq!(
            same_file,
            vec![collision(
                &group.to_lowercase(),
                ".github/workflows/ci.yml, .github/workflows/ci.yml#build"
            )],
            "{group}"
        );
    }
}

#[test]
fn longer_github_workflow_expression_is_not_partitioned() {
    let group = "${{ github.workflow || 'x' }}";
    let found = collisions(&workflows(&[
        (".github/workflows/a.yml", group),
        (".github/workflows/b.yml", group),
    ]));
    assert_eq!(
        found,
        vec![collision(
            group,
            ".github/workflows/a.yml, .github/workflows/b.yml"
        )]
    );
}

#[test]
fn uppercase_github_workflow_placeholder_is_not_partitioned() {
    let group = "${{ GITHUB.WORKFLOW }}";
    let found = collisions(&workflows(&[
        (".github/workflows/a.yml", group),
        (".github/workflows/b.yml", group),
    ]));
    assert_eq!(
        found,
        vec![collision(
            &group.to_lowercase(),
            ".github/workflows/a.yml, .github/workflows/b.yml"
        )]
    );
}

#[test]
fn three_owners_produce_one_finding_with_sorted_ids() {
    let found = collisions(&workflows(&[
        (".github/workflows/c.yml", "shared"),
        (".github/workflows/a.yml", "shared"),
        (".github/workflows/b.yml", "Shared"),
    ]));
    assert_eq!(
        found,
        vec![collision(
            "shared",
            ".github/workflows/a.yml, .github/workflows/b.yml, .github/workflows/c.yml"
        )]
    );
}

#[test]
fn owners_without_concurrency_are_ignored() {
    let path = ".github/workflows/ci.yml";
    let topology = support::topology(
        vec![
            support::workflow(path, Some(lock("shared")), &["build"]),
            support::workflow(".github/workflows/other.yml", None, &["build"]),
        ],
        vec![support::job(path, "build", None)],
    );
    assert!(collisions(&topology).is_empty());
}

#[test]
fn a_single_owner_is_not_a_collision() {
    assert!(collisions(&workflows(&[(".github/workflows/one.yml", "shared")])).is_empty());
}

#[test]
fn flag_off_omits_collisions_and_keeps_other_findings() {
    let topology = workflows(&[
        (".github/workflows/one.yml", "shared"),
        (".github/workflows/two.yml", "shared"),
    ]);
    let off = gated(
        &topology,
        "requiredJobs: [missing]\nforbidConcurrencyGroupCollisions: false\n",
    );
    assert_eq!(off, vec!["required job missing: missing".to_string()]);
    let omitted = gated(&topology, "requiredJobs: [missing]\n");
    assert_eq!(omitted, off);
}

#[test]
fn flag_on_reports_collisions_through_the_rule() {
    let topology = workflows(&[
        (".github/workflows/one.yml", "shared"),
        (".github/workflows/two.yml", "shared"),
    ]);
    let found = gated(&topology, "forbidConcurrencyGroupCollisions: true\n");
    assert_eq!(
        found,
        vec![collision(
            "shared",
            ".github/workflows/one.yml, .github/workflows/two.yml"
        )]
    );
}

fn fixture_messages(name: &str) -> Vec<String> {
    let root = super::fixture(&format!("group-collisions/{name}"));
    run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml")))
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

#[test]
fn group_collision_fixture_passes_when_groups_are_partitioned() {
    let found = fixture_messages("pass");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn group_collision_fixture_reports_each_shared_group() {
    let first = fixture_messages("fail");
    let second = fixture_messages("fail");
    assert_eq!(first, second);
    assert_eq!(
        first,
        vec![
            collision(
                "${{ github.ref }}",
                ".github/workflows/a.yml, .github/workflows/b.yml"
            ),
            collision(
                "${{ github.workflow }}-${{ github.ref }}",
                ".github/workflows/ci.yml, .github/workflows/ci.yml#build"
            ),
            collision(
                "shared",
                ".github/workflows/one.yml, .github/workflows/two.yml"
            ),
        ]
    );
}

#[test]
fn mermaid_keeps_one_lock_per_expression_while_the_rule_collides() {
    // Expression groups share a finding here and still draw one Mermaid lock
    // per declaration. Literal `shared` is the opposite: one diagram lock.
    let root = super::fixture("group-collisions/fail");
    let topology = load_workflow_topology(&root, &CiConfig::default(), &[]);
    let diagram = render_workflow_topology_mermaid(&topology);
    let lock_nodes = diagram
        .lines()
        .filter(|line| line.contains("{{\"lock:"))
        .count();
    assert_eq!(lock_nodes, 5, "{diagram}");
    assert_eq!(diagram.matches("lock: shared").count(), 1, "{diagram}");
    assert_eq!(
        diagram.matches("lock: ${{ github.ref }}").count(),
        2,
        "{diagram}"
    );
    assert_eq!(
        diagram
            .matches("lock: ${{ github.workflow }}-${{ github.ref }}")
            .count(),
        2,
        "{diagram}"
    );
    assert_eq!(fixture_messages("fail").len(), 3);
}
