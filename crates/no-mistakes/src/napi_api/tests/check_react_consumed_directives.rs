use super::*;

fn root(shape: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/react-consumed-directives")
        .join(shape)
}

fn report(shape: &str, config: &str, include_suppressed: bool) -> serde_json::Value {
    let root = root(shape);
    serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": root.join(config), "includeSuppressed": include_suppressed,
        })))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn every_react_directive_is_consumed_without_expanding_the_component_ledger() {
    for config in [".no-mistakes.yml", "report.yml"] {
        let plain = report("all", config, false);
        let audited = report("all", config, true);
        assert_eq!(plain["rules"], json!([]), "{plain}");
        assert_eq!(plain["advisories"], json!([]), "{plain}");
        assert_eq!(plain["react"], json!([]), "{plain}");
        assert!(plain.get("suppressed").is_none());
        assert_eq!(audited["rules"], plain["rules"]);
        assert_eq!(audited["advisories"], plain["advisories"]);
        let ledger = audited["suppressed"].as_array().unwrap();
        // Public accounting remains one deterministic first-target record per component.
        assert_eq!(ledger.len(), 1, "{audited}");
        assert_eq!(
            ledger[0]["directive"],
            json!({ "kind": "nextLine", "line": 3 })
        );
        assert_eq!(ledger[0]["line"], 4);
    }
}

#[test]
fn partially_hidden_fetch_contributions_consume_directives_without_a_component_ledger_row() {
    for config in [".no-mistakes.yml", "report.yml"] {
        let audited = report("partial", config, true);
        assert_eq!(audited["rules"], json!([]), "{audited}");
        assert_eq!(audited["advisories"], json!([]), "{audited}");
        assert_eq!(audited["react"].as_array().unwrap().len(), 1, "{audited}");
        assert_eq!(audited["suppressed"], json!([]), "{audited}");
        assert!(audited["react"][0]
            .get("detail")
            .is_none_or(serde_json::Value::is_null));
    }
}

#[test]
fn batched_react_check_uses_complete_internal_consumption() {
    for shape in ["all", "partial"] {
        let root = root(shape);
        let output = crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(json!({
                "root": root, "config": root.join("report.yml"),
                "reports": [{ "type": "check", "includeSuppressed": true }],
            })),
        )
        .unwrap();
        let batched: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            batched["reports"][0]["result"],
            report(shape, "report.yml", true)
        );
    }
}
