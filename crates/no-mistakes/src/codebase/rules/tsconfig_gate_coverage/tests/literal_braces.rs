use super::*;

#[test]
fn ci_scanner_preserves_literal_closing_braces_outside_interpolation() {
    let workflow = serde_yaml::from_str(
        "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: |\n          echo '{\"nested\":{\"ok\":true}}'\n          tsc --noEmit --project app/tsconfig.json\n",
    )
    .unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/typecheck.yml".to_string(),
            value: Ok(workflow),
        }],
    };
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        tracked
    );
}
