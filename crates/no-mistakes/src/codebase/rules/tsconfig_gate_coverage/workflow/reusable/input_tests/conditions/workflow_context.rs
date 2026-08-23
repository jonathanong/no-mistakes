use super::*;

#[test]
fn reusable_conditions_evaluate_to_json_of_known_arrays() {
    let parsed = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  enabled:\n    if: contains(toJSON(fromJSON('[\"push\", \"schedule\"]')), github.event_name)\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project enabled/tsconfig.json\n  disabled:\n    if: contains(toJSON(fromJSON('[\"schedule\"]')), github.event_name)\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project disabled/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "enabled/tsconfig.json".to_string(),
        "disabled/tsconfig.json".to_string(),
    ]);
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from(["enabled/tsconfig.json".to_string()])
    );
}

#[test]
fn workflow_names_and_path_fallbacks_reach_direct_and_reusable_conditions() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/named.yml",
                "name: Named Checks\non: push\njobs:\n  direct:\n    if: github.workflow == 'Named Checks'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p named-direct/tsconfig.json\n  call:\n    uses: ./.github/workflows/callee.yml\n",
            ),
            document(
                ".github/workflows/unnamed.yml",
                "on: push\njobs:\n  direct:\n    if: github.workflow == '.github/workflows/unnamed.yml'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p unnamed-direct/tsconfig.json\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on: workflow_call\njobs:\n  forwarded:\n    if: github.workflow == 'Named Checks'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p forwarded/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "named-direct/tsconfig.json".to_string(),
        "unnamed-direct/tsconfig.json".to_string(),
        "forwarded/tsconfig.json".to_string(),
    ]);
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        tracked
    );
}

#[test]
fn malformed_static_format_conditions_do_not_credit_typechecks() {
    let parsed = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/formats.yml",
            "on: push\njobs:\n  malformed:\n    if: format('{1}', 'value') == 'value'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p malformed/tsconfig.json\n  valid:\n    if: format('{0}', 'value') == 'value'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p valid/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "malformed/tsconfig.json".to_string(),
        "valid/tsconfig.json".to_string(),
    ]);
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from(["valid/tsconfig.json".to_string()])
    );
}
