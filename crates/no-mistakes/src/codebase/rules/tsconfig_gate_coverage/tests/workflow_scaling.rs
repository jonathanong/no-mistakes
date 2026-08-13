use super::*;

fn workflow(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

fn scan(documents: Vec<ParsedWorkflowDocument>, project: &str) -> BTreeSet<String> {
    let tracked = BTreeSet::from([project.to_string()]);
    ci_typechecked_projects(
        &ParsedWorkflowSet { documents },
        &tracked,
        &project_inputs(&tracked),
    )
}

fn scan_with_stats(
    documents: Vec<ParsedWorkflowDocument>,
    project: &str,
) -> (BTreeSet<String>, usize) {
    let tracked = BTreeSet::from([project.to_string()]);
    ci_typechecked_projects_with_stats(
        &ParsedWorkflowSet { documents },
        &tracked,
        &project_inputs(&tracked),
    )
}

#[test]
fn repeated_reusable_activations_share_results() {
    let mut documents = vec![workflow(
        ".github/workflows/root.yml",
        "on: push\njobs:\n  call:\n    uses: ./.github/workflows/1.yml\n",
    )];
    for level in 1..=8 {
        let jobs = if level == 8 {
            "jobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project shared/tsconfig.json\n".to_string()
        } else {
            let calls = (0..8)
                .map(|index| {
                    format!(
                        "  call-{index}:\n    uses: ./.github/workflows/{}.yml\n",
                        level + 1
                    )
                })
                .collect::<String>();
            format!("jobs:\n{calls}")
        };
        documents.push(workflow(
            &format!(".github/workflows/{level}.yml"),
            &format!("on: workflow_call\n{jobs}"),
        ));
    }

    // Root plus eight reusable levels reaches depth 9, leaving one level before the limit.
    let (projects, computations) = scan_with_stats(documents, "shared/tsconfig.json");
    assert_eq!(
        projects,
        BTreeSet::from(["shared/tsconfig.json".to_string()])
    );
    assert_eq!(computations, 9);
}

#[test]
fn skipped_needs_are_case_insensitive() {
    let documents = vec![workflow(
        ".github/workflows/root.yml",
        "on: push\njobs:\n  Setup:\n    if: false\n  typecheck:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project blocked/tsconfig.json\n",
    )];

    assert!(scan(documents, "blocked/tsconfig.json").is_empty());
}

fn remote_calls(count: usize, target: impl Fn(usize) -> String) -> String {
    (0..count)
        .map(|index| {
            format!(
                "  call-{index}:\n    uses: owner/repository/.github/workflows/{}@main\n",
                target(index)
            )
        })
        .collect()
}

fn remote_limit_root(calls: &str) -> ParsedWorkflowDocument {
    workflow(
        ".github/workflows/root.yml",
        &format!(
            "on: push\njobs:\n{calls}  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unique/tsconfig.json\n"
        ),
    )
}

#[test]
fn reusable_workflow_limit_counts_unique_targets_once() {
    let fifty = remote_calls(50, |index| format!("{index}.yml"));
    assert_eq!(
        scan(vec![remote_limit_root(&fifty)], "unique/tsconfig.json"),
        BTreeSet::from(["unique/tsconfig.json".to_string()])
    );

    let fifty_one = remote_calls(51, |index| format!("{index}.yml"));
    assert!(scan(vec![remote_limit_root(&fifty_one)], "unique/tsconfig.json").is_empty());

    let skipped_overflow = format!(
        "{fifty}  skipped-overflow:\n    if: false\n    uses: owner/repository/.github/workflows/overflow.yml@main\n"
    );
    assert!(scan(
        vec![remote_limit_root(&skipped_overflow)],
        "unique/tsconfig.json"
    )
    .is_empty());

    let zero_matrix_overflow = format!(
        "{fifty}  zero-matrix-overflow:\n    strategy:\n      matrix: {{}}\n    uses: owner/repository/.github/workflows/overflow.yml@main\n"
    );
    assert!(scan(
        vec![remote_limit_root(&zero_matrix_overflow)],
        "unique/tsconfig.json"
    )
    .is_empty());

    let repeated = remote_calls(60, |_| "shared.yml".to_string());
    assert_eq!(
        scan(vec![remote_limit_root(&repeated)], "unique/tsconfig.json"),
        BTreeSet::from(["unique/tsconfig.json".to_string()])
    );
}

#[test]
fn reusable_workflow_limit_spans_nested_local_and_remote_targets() {
    let root = workflow(
        ".github/workflows/root.yml",
        "on: push\njobs:\n  local:\n    uses: ./.github/workflows/nested.yml\n",
    );
    let nested = |remote_count| {
        let calls = remote_calls(remote_count, |index| format!("{index}.yml"));
        workflow(
            ".github/workflows/nested.yml",
            &format!(
                "on: workflow_call\njobs:\n{calls}  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unique/tsconfig.json\n"
            ),
        )
    };

    assert_eq!(
        scan(vec![root.clone(), nested(49)], "unique/tsconfig.json"),
        BTreeSet::from(["unique/tsconfig.json".to_string()])
    );
    assert!(scan(vec![root, nested(50)], "unique/tsconfig.json").is_empty());
}

fn layered_input_workflows(levels: usize, fanout: usize) -> Vec<ParsedWorkflowDocument> {
    let root_calls = (0..fanout)
        .map(|branch| {
            format!(
                "  call-{branch}:\n    uses: ./.github/workflows/1.yml\n    with:\n      slot-1: branch-{branch}\n"
            )
        })
        .collect::<String>();
    let mut documents = vec![workflow(
        ".github/workflows/root.yml",
        &format!("on: push\njobs:\n{root_calls}"),
    )];

    for level in 1..=levels {
        let inputs = (1..=level)
            .map(|slot| {
                format!("      slot-{slot}:\n        required: true\n        type: string\n")
            })
            .collect::<String>();
        let jobs = if level == levels {
            "jobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project app/tsconfig.json\n".to_string()
        } else {
            let calls = (0..fanout)
                .map(|branch| {
                    let forwarded = (1..=level)
                        .map(|slot| format!("      slot-{slot}: ${{{{ inputs.slot-{slot} }}}}\n"))
                        .collect::<String>();
                    format!(
                        "  call-{branch}:\n    uses: ./.github/workflows/{}.yml\n    with:\n{forwarded}      slot-{}: branch-{branch}\n",
                        level + 1,
                        level + 1,
                    )
                })
                .collect::<String>();
            format!("jobs:\n{calls}")
        };
        documents.push(workflow(
            &format!(".github/workflows/{level}.yml"),
            &format!("on:\n  workflow_call:\n    inputs:\n{inputs}{jobs}"),
        ));
    }
    documents
}

#[test]
fn reusable_activation_computation_budget_fails_closed_for_layered_input_fanout() {
    // Four choices at each level preserve all preceding input values. This
    // produces exponentially many memo keys without exceeding the reusable
    // workflow's 50-target or 10-level structural limits.
    let control = scan_with_stats(layered_input_workflows(4, 4), "app/tsconfig.json");
    assert_eq!(control.0, BTreeSet::from(["app/tsconfig.json".to_string()]));
    assert_eq!(control.1, 341);

    let exhausted = scan_with_stats(layered_input_workflows(5, 4), "app/tsconfig.json");
    assert!(exhausted.0.is_empty());
    assert_eq!(exhausted.1, 1024);

    let mut multiple_refs = layered_input_workflows(4, 4);
    multiple_refs[0]
        .value
        .as_mut()
        .expect("parsed root workflow")
        .as_mapping_mut()
        .expect("root workflow mapping")
        .insert(
            Value::String("on".to_string()),
            serde_yaml::from_str("push:\n  branches: [one, two, three, four]")
                .expect("valid trigger fixture"),
        );
    let exhausted_across_refs = scan_with_stats(multiple_refs, "app/tsconfig.json");
    assert!(exhausted_across_refs.0.is_empty());
    assert_eq!(exhausted_across_refs.1, 1024);
}
