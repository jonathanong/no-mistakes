use super::*;

// Output cases moved to cli_rra_snapshot_require_storybook_stories.

fn audit_report(root: &std::path::Path, config: &str, audit: bool) -> serde_json::Value {
    let json = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({ "root": root, "config": root.join(config), "includeSuppressed": audit })
            .to_string(),
    ))
    .unwrap();
    crate::test_support::without_hygiene_advisory_json(serde_json::from_str(&json).unwrap())
}

#[test]
fn malformed_story_direct_matches_aggregate() {
    let root = fixture("malformed-stories");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    assert_eq!(
        check(&root, &config, None).unwrap(),
        crate::codebase::rules::run_check(&root, None, None).unwrap()
    );
}

#[test]
fn malformed_story_audit_preserves_suppressed_channel() {
    let root = fixture("malformed-stories");
    for (config, suppressed) in [
        (".no-mistakes.yml", 2),
        ("advise.yml", 2),
        ("ignore.yml", 0),
        ("suppressed.yml", 2),
        ("filtered.yml", 0),
    ] {
        let baseline = audit_report(&root, config, false);
        let audit = audit_report(&root, config, true);
        assert_eq!(audit["rules"], baseline["rules"]);
        assert_eq!(audit["advisories"], baseline["advisories"]);
        assert_eq!(
            audit["suppressed"].as_array().unwrap().len(),
            suppressed,
            "{config}: {audit}"
        );
    }
}
