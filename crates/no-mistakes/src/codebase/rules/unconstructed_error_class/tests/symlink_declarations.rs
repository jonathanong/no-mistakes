use super::*;

#[test]
fn pnpm_declaration_context_cannot_prove_a_hoisted_non_error_base() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("package-resolution/pnpm-symlink-probe");
    for (config, failures, advisories, suppressed) in [
        (".no-mistakes.yml", 1, 2, 1),
        ("advise.yml", 1, 2, 1),
        ("ignore.yml", 1, 0, 0),
    ] {
        let json = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({"root":root,"config":root.join(config),"includeSuppressed":true})
                .to_string(),
        ))
        .unwrap();
        let report: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            failures,
            "{config}: {report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{config}: {report}"
        );
        assert_eq!(
            report["suppressed"].as_array().unwrap().len(),
            suppressed,
            "{config}: {report}"
        );
        let definite: Vec<_> = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|finding| finding["target"] != "unanalyzable")
            .map(|finding| finding["target"].as_str().unwrap())
            .collect();
        // Safe ./ reexports through a directory symlink still prove an Error base.
        assert_eq!(definite, ["SafeWithin"]);
    }
    let physical = run_check(
        &root,
        Some(&root.join(".no-mistakes.physical-control.yml")),
        None,
    )
    .unwrap();
    // The exact same package declaration bytes under their real dependency
    // context reach the private Error base rather than the benign hoisted dep.
    assert_eq!(
        reported(&physical),
        [
            "control/physical.ts:3 PhysicalBare",
            "control/physical.ts:4 PhysicalRelative",
            "control/physical.ts:6 PhysicalWithin",
        ]
    );
}
