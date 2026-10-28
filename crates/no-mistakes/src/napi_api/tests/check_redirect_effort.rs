use super::*;

#[test]
fn public_check_preserves_delegated_low_findings_and_emits_advisories_once() {
    for name in ["low-reexport", "low-path-alias"] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/nextjs-redirect-destinations/fixture")
            .join(name);
        for policy in ["report", "advise", "ignore"] {
            let options = json!({ "root": root, "config": root.join(format!(".no-mistakes-low-{policy}.yml")) });
            let report: serde_json::Value = serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                    .unwrap(),
            )
            .unwrap();
            let findings = report["rules"].as_array().unwrap();
            let advisories = report["advisories"].as_array().unwrap();
            assert_eq!(
                findings.len(),
                if policy == "report" { 4 } else { 2 },
                "{report}"
            );
            assert_eq!(
                advisories.len(),
                if policy == "advise" { 2 } else { 0 },
                "{report}"
            );
            assert!(
                findings
                    .iter()
                    .chain(advisories)
                    .all(|row| row["file"] == "next-shared.ts"),
                "{report}"
            );
            assert_eq!(report["warnings"], json!([]), "{report}");
        }
    }
}

#[test]
fn public_check_preserves_delegated_suppression() {
    for name in ["low-delegate-disabled", "low-delegate-suppressed"] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/nextjs-redirect-destinations/fixture")
            .join(name);
        for include_suppressed in [false, true] {
            let options = json!({ "root": root, "config": root.join(".no-mistakes.yml"), "includeSuppressed": include_suppressed });
            let report: serde_json::Value = serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(report["rules"], json!([]), "{report}");
            assert_eq!(report["advisories"], json!([]), "{report}");
            if include_suppressed && name == "low-delegate-suppressed" {
                assert_eq!(
                    report["suppressed"].as_array().unwrap().len(),
                    2,
                    "{report}"
                );
            }
        }
    }
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn delegated_check_parses_each_config_once_for_both_sections_and_channels() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/nextjs-redirect-destinations/fixture/low-path-alias")
        .canonicalize()
        .unwrap();
    // Counters include Rayon workers, so isolate paths from parallel fixture consumers.
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    no_mistakes::ast::begin_parse_count(&root);
    // Another request using the saved fixture must not inflate this request's counts.
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                check_json_impl(crate::napi_api::options::test_json_arg(
                    json!({ "root": source, "config": source.join(".no-mistakes-low-advise.yml") })
                        .to_string(),
                ))
                .unwrap();
            })
            .join()
            .unwrap();
    });
    let options = json!({ "root": root, "config": root.join(".no-mistakes-low-advise.yml") });
    let report: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap();
    let counts = no_mistakes::ast::finish_parse_count(&root);
    assert_eq!(report["advisories"].as_array().unwrap().len(), 2);
    for file in ["next.config.ts", "next-shared.ts"] {
        assert_eq!(counts.get(&root.join(file)), Some(&1), "{counts:#?}");
    }
}
