use super::*;

#[test]
fn runtime_typescript_constructor_bindings_are_not_native_sets() {
    let root = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation");
    for kind in ["import-equals", "enum", "namespace", "ambient-namespace"] {
        let path = root.join(format!("tests/finite-set-{kind}.test.ts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = extract_embedded_sql_from_source(&path, &source, &tagged_options());
        assert_eq!(facts.calls.len(), 1, "{kind}");
        if kind == "ambient-namespace" {
            assert_eq!(
                texts(&facts.calls[0]),
                [
                    "/* constructorBinding */ SELECT 1 FROM safe_one",
                    "/* constructorBinding */ SELECT 1 FROM safe_two",
                ]
            );
        } else {
            assert!(
                facts.calls[0].variants.is_empty(),
                "runtime binding: {kind}"
            );
        }
    }
}

#[test]
fn repeated_guarded_values_keep_one_correlated_runtime_choice() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-set-correlated.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(&path, &source, &tagged_options());
    assert_eq!(facts.calls.len(), 2);
    for (index, (names, head)) in [
        (
            ["one", "two", "three", "four", "five", "six"].as_slice(),
            "correlatedOrdinary",
        ),
        (["one", "two", "three"].as_slice(), "correlatedTagged"),
    ]
    .into_iter()
    .enumerate()
    {
        let expected = names
            .iter()
            .map(|name| format!("/* {head} */ SELECT 1 FROM {name} JOIN {name} USING (id)"))
            .collect::<Vec<_>>();
        assert_eq!(texts(&facts.calls[index]), expected);
    }
}

#[test]
fn parameter_defaults_cannot_capture_and_change_guarded_parameters() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-set-default-capture.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(&path, &source, &tagged_options());
    assert_eq!(facts.calls.len(), 1);
    assert!(facts.calls[0].variants.is_empty());
}
