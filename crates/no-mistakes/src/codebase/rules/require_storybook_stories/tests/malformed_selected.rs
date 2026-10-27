use super::*;

// Output cases moved to cli_rra_snapshot_require_storybook_stories.

fn report(name: &str, audit: bool) -> serde_json::Value {
    let root = fixture("malformed-selected");
    let result = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({
            "root": root, "config": root.join(name), "includeSuppressed": audit,
        })
        .to_string(),
    ))
    .unwrap();
    crate::test_support::without_hygiene_advisory_json(serde_json::from_str(&result).unwrap())
}

#[test]
fn malformed_selected_direct_matches_aggregate() {
    let root = fixture("malformed-selected");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    assert_eq!(
        check(&root, &config, None).unwrap(),
        crate::codebase::rules::run_check(&root, None, None).unwrap()
    );
}

#[test]
fn malformed_selected_audit_records_only_real_suppressions() {
    for config in [".no-mistakes.yml", "advise.yml", "ignore.yml"] {
        let baseline = report(config, false);
        let audit = report(config, true);
        assert_eq!(audit["rules"], baseline["rules"]);
        assert_eq!(audit["advisories"], baseline["advisories"]);
        let suppressed = audit["suppressed"].as_array().unwrap();
        if config == "ignore.yml" {
            assert!(suppressed.is_empty(), "{audit}");
        } else {
            assert_eq!(suppressed.len(), 2, "{audit}");
            assert_eq!(
                suppressed
                    .iter()
                    .map(|finding| finding["file"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["components/FileDisabled.tsx", "components/LineDisabled.tsx"]
            );
            assert!(suppressed.iter().all(|finding| finding["line"] == 1
                && finding["reason"]
                    .as_str()
                    .unwrap()
                    .contains("coverage is incomplete")));
        }
    }
}
