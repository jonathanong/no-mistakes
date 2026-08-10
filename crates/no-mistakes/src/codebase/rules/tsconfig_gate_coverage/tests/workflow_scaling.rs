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
