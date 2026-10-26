use super::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/fixture/effort")
}

fn check(config: &str, extra: serde_json::Value) -> serde_json::Value {
    let root = root();
    let mut options = json!({ "root": root, "config": root.join(config) });
    options
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap()
}

#[test]
fn public_offset_effort_preserves_max_default_and_routes_low_residuals() {
    let max = check(".no-mistakes.max.yml", json!({}));
    assert_eq!(max, check(".no-mistakes.yml", json!({})));
    let low = check(".no-mistakes.low.yml", json!({}));
    let ignore = check(".no-mistakes.low-ignore.yml", json!({}));
    let advise = check(".no-mistakes.low-advise.yml", json!({}));
    assert!(max["rules"].as_array().unwrap().len() >= low["rules"].as_array().unwrap().len());
    assert_eq!(ignore["rules"], advise["rules"]);
    assert_eq!(
        advise["advisories"].as_array().unwrap().len(),
        2,
        "{advise}"
    );
    assert_eq!(ignore["advisories"], json!([]));
    assert!(ignore["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["file"] == "src/query.ts" && finding["line"] == 8));
    assert_eq!(
        advise["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| finding["line"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [11, 12]
    );
    assert!(
        ignore["rules"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| max["rules"].as_array().unwrap().contains(finding)),
        "{ignore}\n{max}"
    );
    let detailed = check(
        ".no-mistakes.low-advise.yml",
        json!({"includeSqlVerdicts": true}),
    );
    assert_eq!(detailed["rules"], advise["rules"]);
    assert_eq!(detailed["advisories"], advise["advisories"]);
    assert!(detailed["sqlVerdicts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|verdict| verdict["status"] == "abstain" && verdict["advisory"] == true));
}

#[test]
fn analyze_project_offset_effort_matches_check_override() {
    let root = root();
    for effort in ["low", "max"] {
        let expected = check(".no-mistakes.low-advise.yml", json!({"effort": effort}));
        let project: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({
                        "root": root, "config": root.join(".no-mistakes.low-advise.yml"),
                        "reports": [{"type": "check", "effort": effort}]
                    })
                    .to_string(),
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(project["reports"][0]["result"], expected, "{project}");
    }
}

#[test]
fn public_offset_low_suppression_is_auditable() {
    let report = check(
        ".no-mistakes.low-advise.yml",
        json!({"includeSuppressed": true}),
    );
    assert!(
        !report["suppressed"].as_array().unwrap().is_empty(),
        "{report}"
    );
    assert_eq!(
        report["advisories"].as_array().unwrap().len(),
        2,
        "{report}"
    );
}

// Native assertions belong here because check_runner is also compiled by the CLI binary.
#[path = "patch_rule_adapters.rs"]
mod patch_rule_adapters;
