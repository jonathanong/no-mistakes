use super::*;

fn scanned_projects(documents: Vec<ParsedWorkflowDocument>, projects: &[&str]) -> BTreeSet<String> {
    let tracked = projects
        .iter()
        .map(|project| format!("{project}/tsconfig.json"))
        .collect::<BTreeSet<_>>();
    collect_ci_projects_with_stats(
        &ParsedWorkflowSet { documents },
        &tracked,
        &project_inputs(&tracked),
    )
    .0
}

#[test]
fn static_step_failures_only_allow_explicit_continuations() {
    let workflow = document(
        ".github/workflows/steps.yml",
        "on: push\njobs:\n  blocked:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n      - if: true\n        run: tsc --noEmit -p blocked/tsconfig.json\n  always:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false'\n      - if: always()\n        run: tsc --noEmit -p always/tsconfig.json\n  failure:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false'\n      - if: failure()\n        run: tsc --noEmit -p failure/tsconfig.json\n  tolerated:\n    runs-on: ubuntu-latest\n    steps:\n      - continue-on-error: true\n        run: 'false'\n      - run: tsc --noEmit -p tolerated/tsconfig.json\n  non-errexit-failure:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: exit 1\n      - run: tsc --noEmit -p non-errexit-failure/tsconfig.json\n  non-errexit-success:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: 'false; echo ok'\n      - run: tsc --noEmit -p non-errexit-success/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &[
                "blocked",
                "always",
                "failure",
                "tolerated",
                "non-errexit-failure",
                "non-errexit-success",
            ],
        ),
        BTreeSet::from([
            "always/tsconfig.json".to_string(),
            "failure/tsconfig.json".to_string(),
            "non-errexit-success/tsconfig.json".to_string(),
            "tolerated/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn bare_exit_preserves_the_preceding_command_status() {
    let workflow = document(
        ".github/workflows/bare-exit.yml",
        "on: push\njobs:\n  failed:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: 'false; exit'\n      - run: tsc --noEmit -p failed/tsconfig.json\n  succeeded:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: 'true; exit'\n      - run: tsc --noEmit -p succeeded/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["failed", "succeeded"]),
        BTreeSet::from(["succeeded/tsconfig.json".to_string()])
    );
}

#[test]
fn static_job_failures_propagate_through_needs() {
    let workflow = document(
        ".github/workflows/job-failures.yml",
        "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n  ordinary:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p ordinary/tsconfig.json\n  literal:\n    needs: setup\n    if: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p literal/tsconfig.json\n  transitive:\n    needs: ordinary\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p transitive/tsconfig.json\n  always:\n    needs: setup\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p always/tsconfig.json\n  failure-handler:\n    needs: setup\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure-handler/tsconfig.json\n      - run: exit 1\n  after-handler:\n    needs: failure-handler\n    if: always() && needs.failure-handler.result == 'failure'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p after-handler/tsconfig.json\n  failure-result:\n    needs: setup\n    if: always() && needs.setup.result == 'failure'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure-result/tsconfig.json\n  tolerated:\n    runs-on: ubuntu-latest\n    steps:\n      - continue-on-error: true\n        run: exit 1\n  after-tolerated:\n    needs: tolerated\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p after-tolerated/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &[
                "ordinary",
                "literal",
                "transitive",
                "always",
                "failure-handler",
                "after-handler",
                "failure-result",
                "after-tolerated",
            ],
        ),
        BTreeSet::from([
            "after-tolerated/tsconfig.json".to_string(),
            "after-handler/tsconfig.json".to_string(),
            "always/tsconfig.json".to_string(),
            "failure-result/tsconfig.json".to_string(),
        ])
    );
}

mod job_failures;
mod resolution;
mod run_scripts;

#[test]
fn masked_failures_and_skipped_needs_preserve_distinct_statuses() {
    let workflow = document(
        ".github/workflows/statuses.yml",
        "on: push\njobs:\n  masked:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false && echo masked; echo completed'\n      - run: tsc --noEmit -p masked/tsconfig.json\n  setup:\n    if: false\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo skipped\n  failure:\n    needs: setup\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure-after-skip/tsconfig.json\n  always:\n    needs: setup\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p always-after-skip/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &["masked", "failure-after-skip", "always-after-skip"],
        ),
        BTreeSet::from([
            "always-after-skip/tsconfig.json".to_string(),
            "masked/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn failure_propagating_shell_constructs_block_later_steps() {
    let workflow = document(
        ".github/workflows/failure-propagation.yml",
        "on: push\njobs:\n  errexit:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'set -e; false; echo unreachable'\n      - run: tsc --noEmit -p errexit/tsconfig.json\n  pipefail:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash\n        run: 'false | true; echo unreachable'\n      - run: tsc --noEmit -p pipefail/tsconfig.json\n  recovered:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false | true || echo recovered'\n      - run: tsc --noEmit -p recovered/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["errexit", "pipefail", "recovered"]),
        BTreeSet::from(["recovered/tsconfig.json".to_string()])
    );
}

#[test]
fn pipefail_preserves_final_pipeline_and_and_list_status_without_errexit() {
    let workflow = document(
        ".github/workflows/custom-pipefail.yml",
        "on: push\njobs:\n  pipeline:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash -o pipefail {0}'\n        run: 'false | true'\n      - run: tsc --noEmit -p pipeline/tsconfig.json\n  and-list:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash\n        run: 'false | true && echo masked'\n      - run: tsc --noEmit -p and-list/tsconfig.json\n  completed:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false | true && echo masked; echo completed'\n      - run: tsc --noEmit -p completed/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["pipeline", "and-list", "completed"]),
        BTreeSet::from(["completed/tsconfig.json".to_string()])
    );
}

#[test]
fn pipefail_tracks_reachable_pipelines_later_in_and_lists() {
    let workflow = document(
        ".github/workflows/later-pipeline.yml",
        "on: push\njobs:\n  final-pipeline:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash -o pipefail {0}'\n        run: 'true | true && false | true'\n      - run: tsc --noEmit -p final-pipeline/tsconfig.json\n",
    );

    assert!(scanned_projects(vec![workflow], &["final-pipeline"]).is_empty());
}
