#[test]
fn shared_resolver_visibility_keeps_additive_test_impact_fields_stable() {
    let root = repo_fixture(&["fixtures", "importers", "tests-scoped-alias"]);
    let file = "packages/unit/src/subject.ts";
    let query = |tests| {
        parse_json(
            crate::napi_api::queries::importers_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "file": file, "tests": tests}),
            ))
            .unwrap(),
        )
    };
    let baseline = query(false);
    let with_tests = query(true);
    // A broader runner alias catalog must never answer the ordinary fields.
    assert_eq!(baseline["directImporters"], json!([]));
    assert_eq!(baseline["dependentsCount"], 0);
    assert_eq!(with_tests["directImporters"], baseline["directImporters"]);
    assert_eq!(with_tests["dependentsCount"], baseline["dependentsCount"]);
    assert_eq!(
        with_tests["testImpact"]["tests"],
        json!(["packages/unit/tests/owner.test.ts"])
    );
}
