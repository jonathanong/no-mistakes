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

#[test]
fn reusable_workflow_outputs_flow_to_callers_needs_context() {
    let caller = document(
        ".github/workflows/caller.yml",
        "on: push\njobs:\n  call-false:\n    uses: ./.github/workflows/boolean.yml\n    with: {enabled: false}\n  blocked:\n    needs: call-false\n    if: needs.call-false.outputs.enabled == 'true'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p blocked/tsconfig.json\n  call-bracket:\n    uses: ./.github/workflows/boolean.yml\n    with: {enabled: true}\n  enabled:\n    needs: call-bracket\n    if: needs['call-bracket']['outputs']['enabled'] == 'true'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p enabled/tsconfig.json\n  call-dynamic:\n    uses: ./.github/workflows/dynamic.yml\n  dynamic:\n    needs: call-dynamic\n    if: needs.call-dynamic.outputs.enabled == 'true'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dynamic/tsconfig.json\n",
    );
    let boolean_callee = document(
        ".github/workflows/boolean.yml",
        "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\n    outputs:\n      enabled: {value: '${{ inputs.enabled }}'}\njobs:\n  complete:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo complete\n",
    );
    let dynamic_callee = document(
        ".github/workflows/dynamic.yml",
        "on:\n  workflow_call:\n    outputs:\n      enabled: {value: '${{ jobs.complete.outputs.enabled }}'}\njobs:\n  complete:\n    runs-on: ubuntu-latest\n    outputs: {enabled: '${{ steps.output.outputs.enabled }}'}\n    steps:\n      - id: output\n        run: echo complete\n",
    );

    assert_eq!(
        scanned_projects(
            vec![caller, boolean_callee, dynamic_callee],
            &["blocked", "enabled", "dynamic"],
        ),
        BTreeSet::from([
            "dynamic/tsconfig.json".to_string(),
            "enabled/tsconfig.json".to_string(),
        ])
    );
}

mod concurrency;
mod dispatchers;
mod job_failures;
mod job_results;
mod matrix_fail_fast;
mod matrix_identity;
mod ref_kind;
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

    assert!(scanned_projects(vec![workflow], &["errexit", "pipefail", "recovered"]).is_empty());
}

#[test]
fn pipefail_preserves_final_pipeline_and_and_list_status_without_errexit() {
    let workflow = document(
        ".github/workflows/custom-pipefail.yml",
        "on: push\njobs:\n  pipeline:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash -o pipefail {0}'\n        run: 'false | true'\n      - run: tsc --noEmit -p pipeline/tsconfig.json\n  inline:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false | true && tsc --noEmit -p inline/tsconfig.json'\n  and-list:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash\n        run: 'false | true && echo masked'\n      - run: tsc --noEmit -p and-list/tsconfig.json\n  completed:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false | true && echo masked; echo completed'\n      - run: tsc --noEmit -p completed/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &["pipeline", "inline", "and-list", "completed"],
        ),
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

#[test]
fn unsafe_runtime_pipefail_and_unreachable_suffixes_earn_no_coverage() {
    let workflow = document(
        ".github/workflows/runtime-pipefail.yml",
        "on: push\njobs:\n  command-mutation:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash -e {0}\n        run: |\n          command -p eval 'set -o pipefail'\n          false | true\n          tsc --noEmit -p command-mutation/tsconfig.json\n  builtin-mutation:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash -e {0}\n        run: |\n          builtin -- eval 'set -o pipefail'\n          false | true\n          tsc --noEmit -p builtin-mutation/tsconfig.json\n  suffix:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash -eo pipefail {0}\n        run: |\n          tsc --noEmit -p suffix/tsconfig.json\n          false | true\n          set +e\n",
    );

    assert!(scanned_projects(
        vec![workflow],
        &["command-mutation", "builtin-mutation", "suffix"],
    )
    .is_empty());
}

#[test]
fn known_success_outcomes_skip_failure_conditioned_steps() {
    let workflow = document(
        ".github/workflows/masked-success.yml",
        "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - id: setup\n        run: 'false && true; true'\n      - if: \"${{ steps.setup.outcome == 'failure' }}\"\n        run: tsc --noEmit -p app/tsconfig.json\n",
    );

    assert!(scanned_projects(vec![workflow], &["app"]).is_empty());
}

#[test]
fn unknown_nonfinal_and_predecessors_do_not_truncate_pipefail_prefixes() {
    let workflow = document(
        ".github/workflows/unknown-prefix.yml",
        "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash -eo pipefail {0}\n        run: 'tsc --noEmit -p before/tsconfig.json && unknown && false | true; tsc --noEmit -p after/tsconfig.json'\n",
    );

    assert!(scanned_projects(vec![workflow], &["before", "after"]).is_empty());
}

#[test]
fn shell_replacements_and_escaped_shell_mutations_earn_no_coverage() {
    let workflow = document(
        ".github/workflows/shell-replacements.yml",
        "on: push\njobs:\n  direct:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'X=1 s\"et\" -o pipefail; false | true; tsc --noEmit -p direct/tsconfig.json'\n  dispatcher:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'CMD=eval; X=1 \"$CMD\" true'\n      - run: tsc --noEmit -p dispatcher/tsconfig.json\n  escaped:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'X=1 >/dev/null false'\n      - run: tsc --noEmit -p escaped/tsconfig.json\n",
    );

    assert!(scanned_projects(vec![workflow], &["direct", "dispatcher", "escaped"]).is_empty());
}
