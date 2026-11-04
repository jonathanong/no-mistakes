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
