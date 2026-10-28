use super::*;

fn tagged_options() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        crate::codebase::postgres::embedded::TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ])
}

#[test]
fn exact_literal_set_guard_bounds_only_its_identical_parameter() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-set-guard.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(&path, &source, &tagged_options());
    assert_eq!(facts.calls.len(), 16);
    assert_eq!(
        super::texts(&facts.calls[0]),
        [
            "/* guardedRead */ SELECT 1 FROM safe_one",
            "/* guardedRead */ SELECT 1 FROM safe_two",
            "/* guardedRead */ SELECT 1 FROM safe_three",
        ]
    );
    assert_eq!(
        super::texts(&facts.calls[1]),
        [
            "/* guardedDdl */ DROP TABLE first",
            "/* guardedDdl */ DROP TABLE second",
        ]
    );
    assert_eq!(
        super::texts(&facts.calls[2]),
        [
            "/* guardedTemplate */ SELECT 1 FROM safe_one",
            "/* guardedTemplate */ SELECT 1 FROM safe_two",
        ]
    );
    for call in &facts.calls[3..] {
        assert!(call.variants.is_empty(), "unproven guard: {call:?}");
    }
}

#[test]
fn source_visible_intrinsic_mutation_voids_all_local_set_guards() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-set-prototype.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(&path, &source, &tagged_options());
    assert_eq!(facts.calls.len(), 1);
    assert!(facts.calls[0].variants.is_empty());
}

#[test]
fn escaped_constructor_is_checked_for_source_visible_mutation() {
    let root = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation");
    let positive = root.join("tests/finite-set-escaped-positive.test.ts");
    let positive_source = std::fs::read_to_string(&positive).unwrap();
    assert!(!positive_source.contains("Set"));
    let positive_facts =
        extract_embedded_sql_from_source(&positive, &positive_source, &tagged_options());
    assert_eq!(positive_facts.calls.len(), 1);
    assert_eq!(
        super::texts(&positive_facts.calls[0]),
        [
            "/* escapedConstructor */ SELECT 1 FROM safe_one",
            "/* escapedConstructor */ SELECT 1 FROM safe_two",
        ]
    );

    let negative = root.join("tests/finite-set-escaped-negative.test.ts");
    let negative_source = std::fs::read_to_string(&negative).unwrap();
    assert!(!negative_source.contains("Set"));
    let negative_facts =
        extract_embedded_sql_from_source(&negative, &negative_source, &tagged_options());
    assert_eq!(negative_facts.calls.len(), 1);
    assert!(negative_facts.calls[0].variants.is_empty());
}

#[test]
fn imported_constructor_cannot_prove_global_membership() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-set-imported.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(&path, &source, &tagged_options());
    assert_eq!(facts.calls.len(), 1);
    assert!(facts.calls[0].variants.is_empty());
}

#[test]
fn runtime_constructor_declarations_shadow_the_global_even_when_hoisted() {
    let root = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation");
    for name in [
        "finite-set-function.test.ts",
        "finite-set-class.test.ts",
        "finite-set-forward-var.test.ts",
        "finite-set-enclosing-var.test.ts",
    ] {
        let path = root.join("tests").join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = extract_embedded_sql_from_source(&path, &source, &tagged_options());
        assert_eq!(facts.calls.len(), 1, "{name}");
        assert!(facts.calls[0].variants.is_empty(), "{name}");
    }
}

#[test]
fn type_only_constructor_name_does_not_shadow_the_global() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-set-type-only.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(&path, &source, &tagged_options());
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(
        super::texts(&facts.calls[0]),
        ["/* typeOnlyConstructor */ SELECT 1 FROM safe_one"]
    );
}

#[test]
fn sloppy_arguments_alias_invalidates_a_guarded_parameter() {
    let root = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation");
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let positive = root.join("tests/finite-set-arguments-positive.test.cjs");
    let positive_source = std::fs::read_to_string(&positive).unwrap();
    let positive_facts = extract_embedded_sql_from_source(&positive, &positive_source, &options);
    assert_eq!(positive_facts.calls.len(), 1);
    assert_eq!(
        super::texts(&positive_facts.calls[0]),
        [
            "/* guardedWithoutAlias */ SELECT 1 FROM safe_one",
            "/* guardedWithoutAlias */ SELECT 1 FROM safe_two",
        ]
    );

    let negative = root.join("tests/finite-set-arguments-negative.test.cjs");
    let negative_source = std::fs::read_to_string(&negative).unwrap();
    let negative_facts = extract_embedded_sql_from_source(&negative, &negative_source, &options);
    assert_eq!(negative_facts.calls.len(), 1);
    assert!(negative_facts.calls[0].variants.is_empty());
}
