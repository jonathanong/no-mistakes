use crate::cli::Format;
use crate::codebase::queries::call_sites::{run_json as call_json, CallSitesArgs};
use crate::codebase::queries::dead_exports::{run_json as dead_json, DeadExportsArgs};
use serde_json::{json, Value};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/unreadable-language-caller")
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|error| panic!("{error}: {text}"))
}

#[test]
fn unreadable_configured_caller_makes_language_queries_partial() {
    let root = root();
    let root_display = root.display().to_string();
    let dead = parse(
        &dead_json(DeadExportsArgs {
            file: PathBuf::from("python/greeter.py"),
            names: vec!["Greeter".to_string(), "Unused".to_string()],
            root: Some(root.clone()),
            tsconfig: None,
            format: Some(Format::Json),
            json: true,
        })
        .unwrap(),
    );
    let calls = parse(
        &call_json(CallSitesArgs {
            file: PathBuf::from("python/greeter.py"),
            export_name: "Greeter".to_string(),
            root: Some(root),
            tsconfig: None,
            format: Some(Format::Json),
            json: true,
        })
        .unwrap(),
    );
    assert_eq!(dead["partial"], true);
    assert_eq!(calls["partial"], true);
    assert_eq!(dead["incompleteFiles"], calls["incompleteFiles"]);
    assert_eq!(dead["incompleteFiles"][0]["file"], "python/bad.py");
    assert_eq!(dead["incompleteFiles"][0]["reason"], "operational-error");
    assert_eq!(result(&dead, "Greeter")["referenced"], true);
    assert_eq!(result(&dead, "Unused")["referenced"], false);
    assert_eq!(calls["callSites"][0]["file"], "python/caller.py");
    assert_eq!(calls["callSites"][0]["referenceOnly"], true);
    let napi_dead = parse(
        &crate::napi_api::queries::dead_exports_json_impl(json!({
            "root": root_display,
            "file": "python/greeter.py",
            "names": ["Greeter", "Unused"],
        }))
        .unwrap(),
    );
    assert_eq!(dead, napi_dead);
}

#[test]
fn unreadable_swift_and_dotnet_callers_make_queries_partial() {
    let root = root();
    for (file, bad) in [
        (
            "swift/Sources/App/Greeter.swift",
            "swift/Sources/App/Bad.swift",
        ),
        ("dotnet/Greeter.cs", "dotnet/Bad.cs"),
    ] {
        let dead = parse(
            &dead_json(DeadExportsArgs {
                file: PathBuf::from(file),
                names: vec!["Greeter".to_string()],
                root: Some(root.clone()),
                tsconfig: None,
                format: Some(Format::Json),
                json: true,
            })
            .unwrap(),
        );
        let calls = parse(
            &call_json(CallSitesArgs {
                file: PathBuf::from(file),
                export_name: "Greeter".to_string(),
                root: Some(root.clone()),
                tsconfig: None,
                format: Some(Format::Json),
                json: true,
            })
            .unwrap(),
        );
        assert_eq!(dead["partial"], true, "{file}");
        assert_eq!(calls["partial"], true, "{file}");
        assert_eq!(dead["incompleteFiles"][0]["file"], bad, "{file}");
        assert_eq!(
            dead["incompleteFiles"][0]["reason"], "operational-error",
            "{file}"
        );
        assert_eq!(calls["incompleteFiles"], dead["incompleteFiles"], "{file}");
    }
}

fn result<'a>(report: &'a Value, name: &str) -> &'a Value {
    report["results"]
        .as_array()
        .expect("results")
        .iter()
        .find(|row| row["name"] == name)
        .unwrap_or_else(|| panic!("missing {name}"))
}
