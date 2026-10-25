use super::*;

fn root() -> PathBuf {
    fixture()
        .parent()
        .unwrap()
        .join("shared-package-targets/fixture")
}

fn report(config: &str, effort: Option<&str>) -> serde_json::Value {
    let root = root();
    let mut options = serde_json::json!({"root": root, "config": root.join(config)});
    if let Some(effort) = effort {
        options["effort"] = effort.into();
    }
    serde_json::from_str(
        &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            options.to_string(),
        ))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn complete_package_ancestry_silences_facades_alternatives_helpers_and_workers() {
    let report = report(".no-mistakes.yml", None);
    // These are the only failing classes. In particular, constructing UsedError
    // and UsedTwin through their facade cannot credit their unused siblings.
    assert_eq!(
        reported_channel(&report, "rules"),
        ["src/errors.ts:3 UnusedError", "src/errors.ts:5 UnusedTwin",],
        "{report}"
    );
    // Each positive target has actual declaration ancestry proof; silence must
    // not come from treating all external constructors as safe.
    assert_eq!(
        reported_channel(&report, "advisories"),
        [
            "src/mutation-before.ts:6 unanalyzable",
            "src/opaque.ts:7 unanalyzable",
        ],
        "{report}"
    );
}

#[test]
fn package_constructor_proof_preserves_policy_effort_and_error_recall() {
    for (config, advisories) in [
        (".no-mistakes.yml", 2),
        ("advise.yml", 2),
        ("ignore.yml", 0),
    ] {
        for effort in [Some("low"), Some("max")] {
            let report = report(config, effort);
            assert_eq!(
                reported_channel(&report, "rules"),
                ["src/errors.ts:3 UnusedError", "src/errors.ts:5 UnusedTwin",],
                "{config}, {effort:?}: {report}"
            );
            assert_eq!(
                report["advisories"].as_array().unwrap().len(),
                advisories,
                "{config}, {effort:?}: {report}"
            );
        }
    }
}
