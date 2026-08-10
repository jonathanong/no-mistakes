use super::*;

fn project_inputs(tracked: &BTreeSet<String>) -> ProjectSourceInputs {
    tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect()
}

fn workflow_document(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

#[test]
fn ci_scanner_credits_only_workflows_with_file_triggers() {
    let job = |project: &str| {
        format!(
            "jobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project {project}/tsconfig.json\n"
        )
    };
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document("missing.yml", &job("missing")),
            workflow_document("empty.yml", "on: push"),
            workflow_document(
                "manual.yml",
                &format!("on: workflow_dispatch\n{}", job("manual")),
            ),
            workflow_document(
                "scheduled.yml",
                &format!("on: schedule\n{}", job("scheduled")),
            ),
            workflow_document(
                "pull-request.yml",
                &format!("on: pull_request\n{}", job("pull-request")),
            ),
            workflow_document(
                "filtered-out.yml",
                &format!("on:\n  push:\n    paths: [docs/**]\n{}", job("app")),
            ),
            workflow_document(
                "filtered-in.yml",
                "on:\n  push:\n    paths: [filtered-app/**]\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project filtered-app\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "app/tsconfig.json".to_string(),
        "filtered-app/tsconfig.json".to_string(),
        "pull-request/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from([
            "filtered-app/tsconfig.json".to_string(),
            "pull-request/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn ci_scanner_excludes_jobs_blocked_by_static_needs() {
    let workflow = serde_yaml::from_str(
        "on: push\njobs:\n  setup:\n    if: false\n  direct-blocked:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-blocked/tsconfig.json\n  transitive-blocked:\n    needs: direct-blocked\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project transitive-blocked/tsconfig.json\n  literal-true-blocked:\n    needs: setup\n    if: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project literal-true-blocked/tsconfig.json\n  always-continues:\n    needs: setup\n    if: '${{always()}}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project always-continues/tsconfig.json\n  cancelled-continues:\n    needs: setup\n    if: '!cancelled()'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project cancelled-continues/tsconfig.json\n  soft-failing:\n    continue-on-error: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: false\n  downstream:\n    needs: soft-failing\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project downstream/tsconfig.json\n",
    )
    .unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: "needs.yml".to_string(),
            value: Ok(workflow),
        }],
    };
    let tracked = [
        "always-continues/tsconfig.json",
        "cancelled-continues/tsconfig.json",
        "direct-blocked/tsconfig.json",
        "downstream/tsconfig.json",
        "literal-true-blocked/tsconfig.json",
        "transitive-blocked/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from([
            "always-continues/tsconfig.json".to_string(),
            "cancelled-continues/tsconfig.json".to_string(),
            "downstream/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn ci_scanner_follows_local_reusable_workflows_from_file_triggered_callers() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/caller.yml",
                "on:\n  push:\n    paths: [app/**]\njobs:\n  checks:\n    uses: ./.github/workflows/checks.yml\n",
            ),
            workflow_document(
                ".github/workflows/checks.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project app/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        tracked
    );
}

#[test]
fn ci_scanner_resolves_boolean_inputs_through_transitive_reusable_calls() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/caller.yml",
                "on: pull_request\njobs:\n  checks:\n    uses: ./.github/workflows/middle.yml\n    with:\n      enabled: true\n",
            ),
            workflow_document(
                ".github/workflows/middle.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\njobs:\n  leaf:\n    if: '${{ inputs.enabled }}'\n    uses: ./.github/workflows/leaf.yml\n    with:\n      enabled: '${{ inputs.enabled }}'\n",
            ),
            workflow_document(
                ".github/workflows/leaf.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n      defaulted:\n        type: boolean\n        default: true\n      omitted:\n        type: boolean\njobs:\n  enabled-job:\n    if: inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - if: '${{ inputs.enabled }}'\n        run: tsc --noEmit --project enabled/tsconfig.json\n  defaulted-job:\n    if: inputs.defaulted\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project defaulted/tsconfig.json\n  false-comparison:\n    if: inputs.enabled == false\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project false-comparison/tsconfig.json\n  omitted-job:\n    if: inputs.omitted\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project omitted/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "defaulted/tsconfig.json".to_string(),
        "enabled/tsconfig.json".to_string(),
        "false-comparison/tsconfig.json".to_string(),
        "omitted/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from([
            "defaulted/tsconfig.json".to_string(),
            "enabled/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn ci_scanner_requires_one_complete_acyclic_enforcing_reusable_path() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/valid.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/leaf.yml\n",
            ),
            workflow_document(
                ".github/workflows/skipped.yml",
                "on: push\njobs:\n  setup:\n    if: false\n  checks:\n    needs: setup\n    uses: ./.github/workflows/skipped-leaf.yml\n",
            ),
            workflow_document(
                ".github/workflows/invalid.yml",
                "on: push\njobs:\n  missing:\n    uses: ./.github/workflows/missing.yml\n  remote:\n    uses: owner/repo/.github/workflows/checks.yml@main\n  non-callable:\n    uses: ./.github/workflows/non-callable.yml\n  cycle:\n    uses: ./.github/workflows/cycle-a.yml\n",
            ),
            workflow_document(
                ".github/workflows/leaf.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/skipped-leaf.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project skipped/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/non-callable.yml",
                "on: workflow_dispatch\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project non-callable/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/cycle-a.yml",
                "on: workflow_call\njobs:\n  next:\n    uses: ./.github/workflows/cycle-b.yml\n",
            ),
            workflow_document(
                ".github/workflows/cycle-b.yml",
                // The cyclic edge earns no credit, but its enforcing sibling remains valid.
                "on: workflow_call\njobs:\n  next:\n    uses: ./.github/workflows/cycle-a.yml\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project cyclic/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "cyclic/tsconfig.json",
        "non-callable/tsconfig.json",
        "skipped/tsconfig.json",
        "valid/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from([
            "cyclic/tsconfig.json".to_string(),
            "valid/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn ci_scanner_does_not_union_partial_trigger_coverage_across_callers() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/a.yml",
                "on:\n  push:\n    paths: [app/a.ts]\njobs:\n  checks:\n    uses: ./.github/workflows/checks.yml\n",
            ),
            workflow_document(
                ".github/workflows/b.yml",
                "on:\n  push:\n    paths: [app/b.ts]\njobs:\n  checks:\n    uses: ./.github/workflows/checks.yml\n",
            ),
            workflow_document(
                ".github/workflows/checks.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project app/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);
    let project_inputs = BTreeMap::from([(
        "app/tsconfig.json".to_string(),
        BTreeSet::from(["app/a.ts".to_string(), "app/b.ts".to_string()]),
    )]);

    assert!(ci_typechecked_projects(&workflows, &tracked, &project_inputs).is_empty());
}

#[test]
fn ci_scanner_fails_open_for_dynamic_inputs_but_rejects_static_false_callers() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  dynamic:\n    uses: ./.github/workflows/dynamic.yml\n    with:\n      enabled: '${{ needs.detect.outputs.enabled }}'\n  disabled:\n    if: false\n    uses: ./.github/workflows/disabled.yml\n  nonblocking:\n    continue-on-error: '${{ true }}'\n    uses: ./.github/workflows/nonblocking.yml\n",
            ),
            workflow_document(
                ".github/workflows/dynamic.yml",
                // A dynamic boolean may enable the gate, so only statically false paths are rejected.
                "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\njobs:\n  typecheck:\n    if: inputs.enabled == true\n    runs-on: ubuntu-latest\n    steps:\n      - if: inputs.enabled != false\n        run: tsc --noEmit --project dynamic/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/disabled.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project disabled/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/nonblocking.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project nonblocking/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "disabled/tsconfig.json",
        "dynamic/tsconfig.json",
        "nonblocking/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from(["dynamic/tsconfig.json".to_string()])
    );
}

#[test]
fn ci_scanner_validates_call_inputs_and_normalizes_boolean_condition_spacing() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with:\n      enabled: false\n  quoted-mismatch:\n    uses: ./.github/workflows/strict.yml\n    with:\n      enabled: 'true'\n  missing-required:\n    uses: ./.github/workflows/strict.yml\n  nonmapping-with:\n    uses: ./.github/workflows/strict.yml\n    with: true\n  unknown-input:\n    uses: ./.github/workflows/strict.yml\n    with:\n      enabled: true\n      extra: true\n  invalid-default:\n    uses: ./.github/workflows/invalid-default.yml\n",
            ),
            workflow_document(
                ".github/workflows/valid.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\njobs:\n  negated:\n    if: '${{ ! inputs.enabled }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project negated/tsconfig.json\n  compared:\n    if: '${{ inputs.enabled==false }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project compared/tsconfig.json\n  invalid-comparison:\n    if: '${{ inputs.enabled == maybe }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-comparison/tsconfig.json\n  42:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project numeric-job/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/strict.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n        required: true\njobs:\n  typecheck:\n    if: inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/invalid-default.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n        default: 'true'\njobs:\n  typecheck:\n    if: inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-default/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/mixed.yml",
                "on:\n  push:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n        default: true\njobs:\n  typecheck:\n    if: inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-default/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "compared/tsconfig.json",
        "direct-default/tsconfig.json",
        "invalid-default/tsconfig.json",
        "invalid-comparison/tsconfig.json",
        "invalid/tsconfig.json",
        "negated/tsconfig.json",
        "numeric-job/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from([
            "compared/tsconfig.json".to_string(),
            "invalid-comparison/tsconfig.json".to_string(),
            "negated/tsconfig.json".to_string(),
            "numeric-job/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn reusable_callees_own_their_working_directories_and_shells() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/caller.yml",
                "on: push\ndefaults:\n  run:\n    working-directory: caller-only\n    shell: python {0}\njobs:\n  checks:\n    uses: ./.github/workflows/checks.yml\n",
            ),
            workflow_document(
                ".github/workflows/checks.yml",
                "on: workflow_call\ndefaults:\n  run:\n    working-directory: callee-app\n    shell: bash --noprofile --norc -eo pipefail {0}\njobs:\n  valid:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit\n  unsupported-shell:\n    runs-on: ubuntu-latest\n    defaults:\n      run:\n        working-directory: rejected\n        shell: python {0}\n    steps:\n      - run: tsc --noEmit\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "callee-app/tsconfig.json".to_string(),
        "rejected/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from(["callee-app/tsconfig.json".to_string()])
    );
}
