use super::*;
use serde_json::json;

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-audit")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn request() -> serde_json::Value {
    json!({"plan":fixture("plan.json"), "observations":fixture("observations.json")})
}

#[test]
fn audit_accepts_file_object_and_text_artifacts_and_batches_equivalently() {
    let baseline = tests_audit_json_impl(request()).unwrap();
    let plan = std::fs::read_to_string(fixture("plan.json")).unwrap();
    let observations = std::fs::read_to_string(fixture("observations.json")).unwrap();
    let object = json!({"planJson":serde_json::from_str::<serde_json::Value>(&plan).unwrap(),
        "observationsJson":serde_json::from_str::<serde_json::Value>(&observations).unwrap()});
    assert_eq!(tests_audit_json_impl(object.clone()).unwrap(), baseline);
    assert_eq!(
        tests_audit_json_impl(json!({"planJson":plan,"observationsJson":observations})).unwrap(),
        baseline
    );
    let camel = std::fs::read_to_string(fixture("camel-observations.json")).unwrap();
    for options in [
        json!({"plan":fixture("plan.json"),"observations":fixture("camel-observations.json")}),
        json!({"plan":fixture("plan.json"),"observationsJson":camel}),
        json!({"plan":fixture("plan.json"),"observationsJson":serde_json::from_str::<serde_json::Value>(&camel).unwrap()}),
    ] {
        assert_eq!(tests_audit_json_impl(options).unwrap(), baseline);
    }
    let mut report = object;
    report["type"] = json!("testsAudit");
    let aggregate = crate::napi_api::analyze_project::analyze_project_json_impl(json!({
        "root":PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-audit"),
        "reports":[report]
    }))
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&aggregate).unwrap();
    assert_eq!(
        value["reports"][0]["result"],
        serde_json::from_str::<serde_json::Value>(&baseline).unwrap()
    );
}

#[test]
fn audit_options_reject_missing_conflicting_invalid_and_unknown_inputs() {
    for options in [
        json!({}),
        json!({"plan":fixture("plan.json")}),
        json!({"plan":fixture("plan.json"),"planJson":{},"observations":fixture("observations.json")}),
    ] {
        assert!(tests_audit_json_impl(options)
            .unwrap_err()
            .reason
            .contains("Exactly one"));
    }
    let mut options = request();
    options["observationsJson"] = json!({});
    assert!(tests_audit_json_impl(options)
        .unwrap_err()
        .reason
        .contains("Exactly one"));
    for options in [
        json!({"planJson":"invalid","observationsJson":{}}),
        json!({"planJson":{},"observationsJson":{}}),
        json!({"plan":fixture("invalid.json"),"observations":fixture("observations.json")}),
        json!({"plan":fixture("missing.json"),"observations":fixture("observations.json")}),
        json!({"unknown":true}),
        json!({"plan":fixture("plan.json"),"observationsJson":"invalid"}),
        json!({"plan":fixture("plan.json"),"observationsJson":{}}),
        json!({"plan":fixture("plan.json"),"observations":fixture("invalid.json")}),
    ] {
        assert!(tests_audit_json_impl(options).is_err());
    }
    let mut ambiguous: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture("camel-observations.json")).unwrap())
            .unwrap();
    ambiguous["schema_version"] = json!(1);
    let mut nested: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture("camel-observations.json")).unwrap())
            .unwrap();
    nested["tests"][0]["test_file"] = json!("shadowed.test.mts");
    for artifact in [ambiguous, nested] {
        assert!(tests_audit_json_impl(
            json!({"plan":fixture("plan.json"),"observationsJson":artifact})
        )
        .unwrap_err()
        .reason
        .contains("Ambiguous audit artifact field"));
    }
    let mut plan: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture("plan.json")).unwrap()).unwrap();
    plan["schema_version"] = json!(2);
    assert!(tests_audit_json_impl(
        json!({"planJson":plan,"observations":fixture("observations.json")})
    )
    .unwrap_err()
    .reason
    .contains("schema_version"));
}

#[test]
fn audit_inline_errors_name_the_artifact_and_repair() {
    for field in ["plan", "observations"] {
        for artifact in [json!("invalid"), json!({})] {
            let mut options = request();
            options.as_object_mut().unwrap().remove(field);
            options[format!("{field}Json")] = artifact;
            let reason = tests_audit_json_impl(options).unwrap_err().reason;
            assert!(reason.contains(&format!("{field}Json")), "{reason}");
            assert!(
                reason.contains("regenerate a schema_version 1 artifact"),
                "{reason}"
            );
        }
    }
}

#[test]
fn audit_only_batch_performs_no_repository_preparation() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let result = crate::diagnostics::with_observer(Some(observer.clone()), || {
        let mut report = request();
        report["type"] = json!("testsAudit");
        crate::napi_api::analyze_project::analyze_project_json_impl(json!({
            "root":fixture("nonexistent-root"),
            "config":fixture("nonexistent-config.yml"),
            "reports":[report]
        }))
        .unwrap()
    });
    let batch: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(
        batch["reports"][0]["result"],
        serde_json::from_str::<serde_json::Value>(&tests_audit_json_impl(request()).unwrap())
            .unwrap()
    );
    assert!(observer.source_read_snapshot().is_empty());
    assert!(
        observer.snapshot().work.is_empty(),
        "{:?}",
        observer.snapshot().work
    );
}

#[test]
fn mixed_audit_batch_preserves_repository_report_and_work() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/simple/fixture");
    let repository_report =
        json!({"type":"dependencies", "files":["a.mts"], "relationships":["import"]});
    let run = |reports| {
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let result = crate::diagnostics::with_observer(Some(observer.clone()), || {
            crate::napi_api::analyze_project::analyze_project_json_impl(
                json!({"root":root,"reports":reports}),
            )
            .unwrap()
        });
        (
            serde_json::from_str::<serde_json::Value>(&result).unwrap(),
            observer,
        )
    };
    let (baseline, baseline_work) = run(vec![repository_report.clone()]);
    let mut audit = request();
    audit["type"] = json!("testsAudit");
    let (mixed, mixed_work) = run(vec![repository_report, audit]);
    assert_eq!(mixed["reports"][0], baseline["reports"][0]);
    assert_eq!(mixed_work.snapshot().work, baseline_work.snapshot().work);
    assert_eq!(
        mixed_work.source_read_snapshot(),
        baseline_work.source_read_snapshot()
    );
}
