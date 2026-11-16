use super::*;

#[test]
fn parameter_bounds_preserve_scopes_and_supplemental_fact_lookup() {
    let file = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture")
            .join("issue-3501/negative/parameter-scopes.ts"),
    );
    let alternate = file
        .parent()
        .unwrap()
        .join("../negative/parameter-scopes.ts");
    let numeric = parameter("value", Some(ValueType::Number));
    let mut open = parameter("value", None);
    open.scope = 2;
    let mut mixed_numeric = numeric.clone();
    mixed_numeric.scope = 3;
    let mut mixed_open = open.clone();
    mixed_open.scope = 3;
    let mut ts = TsFileFacts::default();
    ts.value_flow.parameters = vec![numeric, open, mixed_numeric, mixed_open];
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    {
        let bindings = Bindings::new(&facts);
        assert!(bindings.closed_parameter(&file, 1, "value"));
        assert!(!bindings.parameter(&file, 1, "value"));
        for scope in [2, 3] {
            assert!(bindings.parameter(&file, scope, "value"));
            assert!(!bindings.closed_parameter(&file, scope, "value"));
        }
        assert!(bindings.parameter(&file, 0, "value"));
        assert!(!bindings.closed_parameter(&file, 0, "value"));
        assert!(!bindings.parameter(&file, 9, "value"));
        assert!(!bindings.parameter(&file, 0, "missing"));
        assert!(bindings.plain_parameter(&file, 1, "value"));
        assert!(!bindings.plain_parameter(&file, 9, "value"));
        assert!(bindings.closed_parameter(&alternate, 1, "value"));
    }
    facts.ts.insert(
        alternate.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts::default()),
            ..Default::default()
        }
        .into(),
    );
    let bindings = Bindings::new(&facts);
    assert!(bindings.closed_parameter(&file, 1, "value"));
    assert!(!bindings.closed_parameter(&alternate, 1, "value"));
    assert!(!bindings.parameter(&alternate, 0, "value"));
}
