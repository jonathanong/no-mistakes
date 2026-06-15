// Included into `napi_api::tests` via `include!`; shares that module's
// imports. N-API parity tests for the issue-419 query commands.

#[test]
fn data_pw_json_returns_report() {
    let options = json!({
        "root": fixture_root("data-pw"),
        "value": "search-bar"
    })
    .to_string();
    let output = data_pw_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["value"], "search-bar");
    assert_eq!(value["source"].as_array().unwrap().len(), 2);
    assert_eq!(value["test"].as_array().unwrap().len(), 1);

    // value is required
    let error =
        data_pw_json_impl(json!({ "root": fixture_root("data-pw") }).to_string()).unwrap_err();
    assert!(error.reason.contains("value is required"));
}

#[test]
fn effects_json_returns_report() {
    let options = json!({
        "root": fixture_root("effects"),
        "kind": "valkey",
        "entry": "app/server.ts"
    })
    .to_string();
    let output = effects_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["kind"], "valkey");
    assert_eq!(value["callSites"].as_array().unwrap().len(), 4);
    assert_eq!(value["byCategory"]["cache"], 2);

    let error =
        effects_json_impl(json!({ "root": fixture_root("effects") }).to_string()).unwrap_err();
    assert!(error.reason.contains("kind is required"));

    let error =
        effects_json_impl(json!({ "root": fixture_root("effects"), "kind": "valkey" }).to_string())
            .unwrap_err();
    assert!(error.reason.contains("entry is required"));
}

#[test]
fn rsc_callers_json_returns_report() {
    let options = json!({
        "root": fixture_root("rsc-callers"),
        "component": "app/ui/Button.tsx"
    })
    .to_string();
    let output = rsc_callers_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["component"], "app/ui/Button.tsx");
    let files: Vec<&str> = value["callers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["file"].as_str().unwrap())
        .collect();
    assert!(files.contains(&"app/ui/Card.tsx"));
    assert!(!files.contains(&"app/ui/ClientThing.tsx"));

    let error = rsc_callers_json_impl(json!({ "root": fixture_root("rsc-callers") }).to_string())
        .unwrap_err();
    assert!(error.reason.contains("component is required"));
}

#[test]
fn registry_extension_json_returns_report() {
    let options = json!({
        "root": fixture_root("registry-extension"),
        "registryFile": "register-call.ts"
    })
    .to_string();
    let output = registry_extension_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["patternKind"], "register-call");
    assert_eq!(value["entries"].as_array().unwrap().len(), 2);

    let error = registry_extension_json_impl(
        json!({ "root": fixture_root("registry-extension") }).to_string(),
    )
    .unwrap_err();
    assert!(error.reason.contains("registryFile is required"));
}
