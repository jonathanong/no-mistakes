use super::load_fixture;

#[test]
fn declared_function_cardinality_keeps_uncertain_and_set_overloads() {
    let catalog = load_fixture("declared-returns.json").unwrap();
    let functions = catalog
        .functions()
        .map(|function| (function.name.as_str(), function.returns_set))
        .collect::<std::collections::BTreeMap<_, _>>();
    for name in ["fn_remaining", "fn_body_text"] {
        assert_eq!(functions[name], Some(false), "{name}");
    }
    for name in ["fn_set", "fn_table", "fn_explicit_set", "fn_conflict"] {
        assert_eq!(functions[name], Some(true), "{name}");
    }
    for name in ["fn_malformed", "fn_wrong_name", "fn_wrong_schema"] {
        assert_eq!(functions[name], None, "{name}");
    }
    let names = catalog.scalar_function_names();
    assert!(names.contains(&vec!["ScalarName".into()]));
    assert!(!names.contains(&vec!["scalarname".into()]));
    assert!(names.contains(&vec!["fn_remaining".into()]));
    assert!(names.contains(&vec!["public".into(), "fn_remaining".into()]));
    for name in [
        "fn_set",
        "fn_table",
        "fn_mixed",
        "fn_unknown_overload",
        "fn_malformed",
        "fn_wrong_name",
        "fn_wrong_schema",
        "fn_explicit_set",
        "fn_conflict",
    ] {
        assert!(!names.contains(&vec![name.into()]), "{name}");
    }
}

#[test]
fn uncertain_definitions_and_invalid_qualified_names_are_not_scalar() {
    let catalog = load_fixture("declared-returns.json").unwrap();
    let functions = catalog
        .functions()
        .map(|f| (f.key.as_str(), f))
        .collect::<std::collections::BTreeMap<_, _>>();
    for key in [
        "fn_out_only(OUT value integer)",
        "fn_non_function()",
        "fn_multiple()",
        "db.public.fn_three_key()",
        "fn_three_declared()",
    ] {
        assert_eq!(functions[key].returns_set, None, "{key}");
    }
    let names = catalog.scalar_function_names();
    assert!(!names.contains(&vec![
        "db".into(),
        "public".into(),
        "fn_known_scalar".into()
    ]));
    let set = functions["fn_set(uuid)"];
    assert_eq!(
        super::super::function_cardinality::resolve(
            None,
            &set.definition,
            &set.name,
            "",
            Some("public")
        ),
        Some(true)
    );
}

#[test]
fn scalar_metadata_without_a_catalog_schema_cannot_establish_names() {
    let catalog = load_fixture("no-schema-scalar.json").unwrap();
    assert!(catalog
        .functions()
        .all(|function| function.returns_set == Some(false)));
    assert!(catalog.scalar_function_names().is_empty());
}
