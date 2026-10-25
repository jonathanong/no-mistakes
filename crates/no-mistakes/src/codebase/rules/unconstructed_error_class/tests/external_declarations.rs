use super::*;

fn assert_channels(root: &std::path::Path, failures: &[&str], advisories: &[&str]) {
    let report = diagnostic_report(root);
    assert_eq!(reported_channel(&report, "rules"), failures);
    assert_eq!(reported_channel(&report, "advisories"), advisories);
}

#[test]
fn package_declarations_prove_only_class_ancestry() {
    let root = fixture().parent().unwrap().join("declarations/fixture");
    assert_channels(
        &root,
        &[
            "src/classes.ts:5 ImportedError",
            "src/classes.ts:13 ImportedCjsError",
        ],
        &[
            "src/classes.ts:6 unanalyzable",
            "src/classes.ts:8 unanalyzable",
        ],
    );
}

#[test]
fn declaration_ancestry_preserves_policy_and_effort_channels() {
    let root = fixture().parent().unwrap().join("declarations/fixture");
    let mut baseline = None;
    for (config, failures, advisories) in [
        (".no-mistakes.yml", 2, 2),
        ("max.yml", 2, 2),
        ("low.yml", 2, 2),
        ("advise.yml", 2, 2),
        ("ignore.yml", 2, 0),
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
            if config == "ignore.yml" { 0 } else { 1 },
            "{config}: {report}"
        );
        if config == ".no-mistakes.yml" {
            baseline = Some(report["rules"].clone());
        }
        if matches!(config, "max.yml" | "low.yml") {
            assert_eq!(Some(&report["rules"]), baseline.as_ref());
        }
    }
}

#[test]
fn declared_namespace_bases_follow_lexical_class_shadowing() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("declaration-scopes/fixture");
    assert_channels(
        &root,
        &[
            "src/classes.ts:2 DirectError",
            "src/classes.ts:3 NestedError",
            "src/classes.ts:11 MergedExportedError",
            "src/classes.ts:13 AmbientMergedError",
        ],
        &[
            "src/classes.ts:6 unanalyzable",
            "src/classes.ts:8 unanalyzable",
        ],
    );
}

#[test]
fn declaration_export_boundaries_remain_unknown_without_positive_class_proof() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("declaration-exports/fixture");
    assert_channels(
        &root,
        // Dynamic ancestry in a package declaration follows the explicit report policy.
        &[
            "src/classes.ts:7 StarError",
            "src/classes.ts:13 unanalyzable",
        ],
        &[
            "src/classes.ts:9 unanalyzable",
            "src/classes.ts:10 unanalyzable",
            "src/classes.ts:11 unanalyzable",
            "src/classes.ts:12 unanalyzable",
            "src/classes.ts:14 unanalyzable",
            "src/classes.ts:15 unanalyzable",
            "src/classes.ts:17 unanalyzable",
            "src/classes.ts:18 unanalyzable",
            "src/classes.ts:19 unanalyzable",
            "src/classes.ts:20 unanalyzable",
        ],
    );
}

#[test]
fn private_members_of_other_nonambient_namespace_blocks_never_supply_class_proof() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("declaration-private/fixture");
    assert_channels(&root, &["src/classes.ts:9 MergedPrivateError"], &[]);
}

#[test]
fn repository_and_declaration_closures_preserve_class_and_opaque_binding_proof() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("declaration-local/fixture");
    assert_channels(
        &root,
        &[
            "src/classes.ts:3 RepositoryChild",
            "src/classes.ts:4 DeclaredChild",
            "src/classes.ts:10 NamespaceChild",
        ],
        &[
            "src/classes.ts:5 unanalyzable",
            "src/classes.ts:15 unanalyzable",
            "src/classes.ts:17 unanalyzable",
            "src/classes.ts:18 unanalyzable",
        ],
    );
}

#[test]
fn cyclic_declaration_classes_keep_shared_uncertainty_policy() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("declaration-exports/fixture");
    for (config, failures, advisories) in [("advise.yml", 1, 11), ("ignore.yml", 1, 0)] {
        let output = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({"root":root,"config":root.join(config)}).to_string(),
        ))
        .unwrap();
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            failures,
            "{report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{report}"
        );
    }
}
