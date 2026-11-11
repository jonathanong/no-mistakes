use crate::codebase::fact_panic::tests::with_injected_panic;
use serde_json::{json, Value};
use std::path::PathBuf;

#[test]
fn fact_panic_batch_check_matches_standalone_with_independent_findings() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/fact-panic-isolation/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    with_injected_panic(&root.join("first.ts"), || {
        with_injected_panic(&root.join("second.ts"), || {
            for config in [".no-mistakes.yml", "clean.yml"] {
                let standalone =
                    crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                        json!({ "root": root, "config": config }).to_string(),
                    ))
                    .unwrap();
                let standalone: Value = serde_json::from_str(&standalone).unwrap();
                let batch =
                    super::analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                        json!({
                            "root": root,
                            "config": config,
                            "reports": [
                                { "type": "check" },
                                { "type": "dependencies", "files": ["healthy.ts"] }
                            ]
                        })
                        .to_string(),
                    ))
                    .unwrap();
                let batch: Value = serde_json::from_str(&batch).unwrap();
                let report = &batch["reports"][0]["result"];
                assert_eq!(report, &standalone);
                let failures = report["incomplete"].as_array().unwrap();
                assert_eq!(failures.len(), 2);
                for name in ["first.ts", "second.ts"] {
                    assert!(failures.iter().any(|failure| {
                        failure["consumer"] == "fact-extraction"
                            && PathBuf::from(failure["file"].as_str().unwrap()).ends_with(name)
                            && failure["message"]
                                .as_str()
                                .unwrap()
                                .contains("injected fact extraction failure")
                    }));
                }
                assert_eq!(
                    report["rules"].as_array().unwrap().len(),
                    usize::from(config == ".no-mistakes.yml")
                );
            }
        })
    });
}
