use super::*;

fn project_inputs(tracked: &BTreeSet<String>) -> ProjectSourceInputs {
    tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect()
}

#[test]
fn ci_scanner_credits_only_workflows_with_file_triggers() {
    let workflow = |path: &str, yaml: &str| ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    };
    let job = |project: &str| {
        format!(
            "jobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project {project}/tsconfig.json\n"
        )
    };
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow("missing.yml", &job("missing")),
            workflow("empty.yml", "on: push"),
            workflow(
                "manual.yml",
                &format!("on: workflow_dispatch\n{}", job("manual")),
            ),
            workflow(
                "scheduled.yml",
                &format!("on: schedule\n{}", job("scheduled")),
            ),
            workflow(
                "pull-request.yml",
                &format!("on: pull_request\n{}", job("pull-request")),
            ),
            workflow(
                "filtered-out.yml",
                &format!("on:\n  push:\n    paths: [docs/**]\n{}", job("app")),
            ),
            workflow(
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
        "on: push\njobs:\n  setup:\n    if: false\n  direct-blocked:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-blocked/tsconfig.json\n  transitive-blocked:\n    needs: direct-blocked\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project transitive-blocked/tsconfig.json\n  literal-true-blocked:\n    needs: setup\n    if: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project literal-true-blocked/tsconfig.json\n  always-continues:\n    needs: setup\n    if: '${{ always() }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project always-continues/tsconfig.json\n  cancelled-continues:\n    needs: setup\n    if: '!cancelled()'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project cancelled-continues/tsconfig.json\n  soft-failing:\n    continue-on-error: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: false\n  downstream:\n    needs: soft-failing\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project downstream/tsconfig.json\n",
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
