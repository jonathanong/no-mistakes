use super::*;

#[test]
fn literal_expression_exclusions_create_a_zero_instance_workflow_boundary() {
    let tracked = BTreeSet::from(["excluded-expression/tsconfig.json".to_string()]);
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow(
            ".github/workflows/excluded-expression.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        enabled: [true]\n        exclude:\n          - enabled: '${{ true }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project excluded-expression/tsconfig.json\n",
        )],
    };
    assert!(ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)).is_empty());
}
