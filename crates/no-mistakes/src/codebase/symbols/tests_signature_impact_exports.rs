#[test]
fn signature_impact_classifies_value_alias_exports_as_exports() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "value-alias-date-barrel.mts" && entry["symbol"] == "parse"
    }));
    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-value-alias-date-barrel.mts" && entry["symbol"] == "parse"
    }));
    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "default-alias-date-barrel.mts" && entry["symbol"] == "default"
    }));
    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "value-alias-private-caller-date-barrel.mts"
            && entry["symbol"] == "parsePrivate"
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "value-alias-private-caller-date-barrel.mts"
            && entry["symbol"] == "renderPrivateAliasDate"
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "value-alias-date-barrel.mts"
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "namespace-value-alias-date-barrel.mts"
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "default-alias-date-barrel.mts"
    }));
}

#[test]
fn value_alias_exports_ignore_unrelated_imports() {
    let imports = vec![crate::codebase::ts_symbols::NamedImport {
        source: "./utils.mts".to_string(),
        imported: "formatDate".to_string(),
        local: "formatDate".to_string(),
        line: 1,
        is_type_only: false,
    }];

    assert!(!impact::value_alias_export(
        "export const parse = formatDate;",
        &imports,
        "parse",
        "parseDate"
    ));
}
