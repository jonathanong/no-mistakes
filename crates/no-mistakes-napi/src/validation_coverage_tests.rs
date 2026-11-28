use serde_json::json;
use std::path::{Path, PathBuf};

fn missing_config() -> &'static str {
    "/nonexistent-no-mistakes-validation-root/missing.yml"
}

#[test]
fn infra_and_swift_json_impls_reject_invalid_and_missing_options() {
    let bad = json!({ "unknownField": true });
    assert!(crate::infra_swift::infra_resource_refs_json_impl(bad.clone()).is_err());
    assert!(crate::infra_swift::infra_outputs_json_impl(bad.clone()).is_err());
    assert!(crate::infra_swift::infra_test_for_json_impl(bad.clone()).is_err());
    assert!(crate::infra_swift::swift_importers_json_impl(bad.clone()).is_err());
    assert!(crate::infra_swift::swift_test_targets_json_impl(bad).is_err());

    assert!(crate::infra_swift::infra_resource_refs_json_impl(json!({})).is_err());
    assert!(crate::infra_swift::infra_outputs_json_impl(json!({})).is_err());
    assert!(crate::infra_swift::infra_test_for_json_impl(json!({})).is_err());
    assert!(crate::infra_swift::swift_importers_json_impl(json!({})).is_err());
    assert!(crate::infra_swift::swift_test_targets_json_impl(json!({})).is_err());

    let missing = missing_config();
    assert!(crate::infra_swift::infra_resource_refs_json_impl(json!({
        "config": missing,
        "address": "aws_s3_bucket.logs"
    }))
    .is_err());
    assert!(crate::infra_swift::infra_outputs_json_impl(json!({
        "config": missing,
        "moduleDir": "modules/app"
    }))
    .is_err());
    assert!(crate::infra_swift::infra_test_for_json_impl(json!({
        "config": missing,
        "tfFile": "main.tf"
    }))
    .is_err());
    assert!(crate::infra_swift::swift_importers_json_impl(json!({
        "config": missing,
        "file": "Sources/App/Main.swift"
    }))
    .is_err());
    assert!(crate::infra_swift::swift_test_targets_json_impl(json!({
        "config": missing,
        "file": "Sources/App/Main.swift"
    }))
    .is_err());
}

#[test]
fn cli_parity_json_impls_reject_invalid_options_and_missing_documents() {
    let bad = json!({ "unknownField": true });
    let missing = missing_config();

    assert!(crate::fetches_json_impl(bad.clone()).is_err());
    assert!(crate::fetches_json_impl(json!({ "config": missing })).is_err());

    assert!(crate::resolve_config_json_impl(bad.clone()).is_err());
    assert!(crate::resolve_config_json_impl(json!({
        "config": missing,
        "view": "resolved"
    }))
    .is_err());
    assert!(crate::resolve_config_json_impl(json!({
        "config": missing,
        "view": "declared"
    }))
    .is_err());

    assert!(crate::tests_targets_json_impl(bad.clone()).is_err());
    assert!(crate::tests_targets_json_impl(json!({})).is_err());
    assert!(crate::tests_targets_json_impl(json!({ "framework": "invalid-fw" })).is_err());
    assert!(crate::tests_targets_json_impl(json!({ "framework": "vitest" })).is_err());
    assert!(crate::tests_targets_json_impl(json!({
        "framework": "vitest",
        "files": ["src/a.ts"],
        "config": missing
    }))
    .is_err());

    assert!(crate::ci_impact_json_impl(bad.clone()).is_err());
    assert!(crate::ci_impact_json_impl(json!({ "config": missing, "files": ["a.ts"] })).is_err());
    assert!(crate::ci_env_json_impl(bad.clone()).is_err());
    assert!(crate::ci_env_json_impl(json!({})).is_err());
    assert!(crate::ci_env_json_impl(json!({ "var": "CI", "config": missing })).is_err());

    assert!(crate::ci_topology_impact_json_impl(bad.clone()).is_err());
    assert!(crate::ci_topology_impact_json_impl(json!({})).is_err());
    assert!(crate::ci_topology_impact_json_impl(json!({ "base": "HEAD" })).is_err());
    assert!(
        crate::ci_topology_impact_json_impl(json!({ "base": "HEAD", "head": "HEAD" })).is_err()
    );
    assert!(crate::ci_topology_impact_json_impl(json!({
        "root": "/nonexistent-no-mistakes-validation-root",
        "base": "HEAD",
        "head": "HEAD",
        "entryWorkflow": ".github/workflows/ci.yml"
    }))
    .is_err());

    assert!(crate::impacted_checks_json_impl(bad.clone()).is_err());
    assert!(crate::impacted_checks_json_impl(json!({
        "config": missing,
        "changedFiles": ["src/a.ts"]
    }))
    .is_err());

    assert!(crate::tests_why_json_impl(bad.clone()).is_err());
    assert!(crate::tests_why_json_impl(json!({
        "framework": "vitest",
        "test": "a.test.ts",
        "config": missing
    }))
    .is_err());

    for f in [
        crate::tests_comment_markdown_impl,
        crate::tests_graph_json_impl,
        crate::tests_graph_mermaid_impl,
    ] {
        assert!(f(bad.clone()).is_err());
        assert!(f(json!({})).is_err());
        assert!(f(json!({ "planJson": "{" })).is_err());
        assert!(f(json!({ "planJson": { "invalid": true } })).is_err());
        assert!(f(json!({ "plan": missing })).is_err());
    }

    assert!(crate::tests_audit_json_impl(bad).is_err());
    assert!(crate::tests_audit_json_impl(json!({})).is_err());
    assert!(crate::tests_audit_json_impl(json!({ "planJson": "{" })).is_err());
    assert!(crate::tests_audit_json_impl(json!({ "plan": missing })).is_err());
}

#[test]
fn codebase_project_and_query_json_impls_reject_invalid_options() {
    let bad = json!({ "unknownField": true });
    let missing = missing_config();

    assert!(crate::symbols_json_impl(bad.clone()).is_err());
    assert!(crate::symbols_json_impl(json!({})).is_err());
    assert!(crate::symbols_json_impl(json!({ "files": ["a.ts"], "mode": "bad" })).is_err());
    assert!(crate::symbols_json_impl(json!({ "files": ["a.ts"], "kinds": ["bad"] })).is_err());
    assert!(crate::symbols_json_impl(json!({ "files": ["a.ts"], "include": "bad" })).is_err());

    assert!(crate::import_usages_json_impl(bad.clone()).is_err());
    assert!(crate::dependencies_json_impl(bad.clone()).is_err());
    assert!(crate::dependencies_json_impl(json!({})).is_err());
    assert!(crate::dependencies_json_impl(json!({
        "files": ["a.ts"],
        "relationships": ["bad-rel"]
    }))
    .is_err());
    assert!(crate::dependencies_json_impl(json!({
        "files": ["a.ts"],
        "projection": "invalid"
    }))
    .is_err());

    assert!(crate::data_pw_json_impl(bad.clone()).is_err());
    assert!(crate::data_pw_json_impl(json!({})).is_err());
    assert!(crate::data_pw_json_impl(json!({ "value": "btn", "include": "bad" })).is_err());
    assert!(crate::data_pw_json_impl(json!({ "value": "btn", "config": missing })).is_err());

    assert!(crate::effects_json_impl(bad.clone()).is_err());
    assert!(crate::effects_json_impl(json!({})).is_err());
    assert!(crate::effects_json_impl(json!({ "kind": "db" })).is_err());
    assert!(crate::effects_json_impl(json!({
        "kind": "db",
        "entry": "src/a.ts",
        "config": missing
    }))
    .is_err());

    assert!(crate::rsc_callers_json_impl(bad.clone()).is_err());
    assert!(crate::rsc_callers_json_impl(json!({})).is_err());
    assert!(crate::rsc_callers_json_impl(json!({
        "component": "src/a.tsx",
        "config": missing
    }))
    .is_err());

    assert!(crate::registry_extension_json_impl(bad.clone()).is_err());
    assert!(crate::registry_extension_json_impl(json!({})).is_err());
    assert!(crate::registry_extension_json_impl(json!({ "registryFile": missing })).is_err());

    assert!(crate::flow_json_impl(bad.clone()).is_err());
    assert!(crate::flow_json_impl(json!({})).is_err());
    assert!(crate::flow_json_impl(json!({
        "target": "src/a.ts",
        "relationships": ["bad-rel"]
    }))
    .is_err());
    assert!(crate::flow_json_impl(json!({
        "target": "src/a.ts",
        "direction": "sideways"
    }))
    .is_err());
    assert!(crate::flow_json_impl(json!({
        "target": "src/a.ts",
        "config": missing
    }))
    .is_err());

    assert!(crate::server_contracts_json_impl(bad.clone()).is_err());
    assert!(crate::react_analyze_json_impl(bad.clone()).is_err());
    assert!(crate::react_check_json_impl(bad.clone()).is_err());
    assert!(crate::react_usages_json_impl(bad.clone()).is_err());

    assert!(crate::queries::importers_json_impl(bad.clone()).is_err());
    assert!(crate::queries::importers_json_impl(json!({ "file": "   " })).is_err());
    assert!(crate::queries::exports_of_json_impl(bad.clone()).is_err());
    assert!(crate::queries::exports_of_json_impl(json!({ "file": "" })).is_err());
    assert!(crate::queries::dead_exports_json_impl(bad.clone()).is_err());
    assert!(crate::queries::dead_exports_json_impl(json!({ "file": "" })).is_err());
    assert!(crate::queries::call_sites_json_impl(bad.clone()).is_err());
    assert!(crate::queries::call_sites_json_impl(json!({ "file": "" })).is_err());
    assert!(crate::queries::call_sites_json_impl(json!({
        "file": "a.ts",
        "exportName": " "
    }))
    .is_err());
    assert!(crate::queries::resolve_check_json_impl(bad.clone()).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({})).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({ "file": "" })).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({ "files": [] })).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({ "files": [" "] })).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({
        "file": "a.ts",
        "files": ["b.ts"]
    }))
    .is_err());

    assert!(crate::postgres_lock_inventory_json_impl(bad.clone()).is_err());
    assert!(crate::postgres_lock_inventory_json_impl(json!({ "config": missing })).is_err());
    assert!(crate::generate_postgres_catalog_json_impl(bad.clone()).is_err());
    assert!(crate::generate_postgres_catalog_json_impl(json!({ "output": 123 })).is_err());
    assert!(crate::lockfile_diff_json_impl(bad).is_err());
    assert!(crate::lockfile_diff_json_impl(json!({})).is_err());
}

#[test]
fn observed_and_filesystem_reject_malformed_inputs() {
    use napi::bindgen_prelude::Buffer;

    let bad_utf8 = Buffer::from(vec![0xff, 0xfe]);
    assert!(
        crate::async_task::observed::execute_json(&bad_utf8, |_| Ok("{}".to_string()), false)
            .is_err()
    );
    let bad_diag = Buffer::from(br#"{"diagnosticsId":123}"#.to_vec());
    assert!(
        crate::async_task::observed::execute_json(&bad_diag, |_| Ok("{}".to_string()), false)
            .is_err()
    );
    assert!(
        crate::async_task::observed::execute_pure(json!({ "diagnosticsId": 123 }), |_| Ok(
            "{}".to_string()
        ))
        .is_err()
    );

    #[cfg(unix)]
    {
        assert!(
            crate::filesystem::rename_no_replace_impl(Path::new("a\0b"), Path::new("c")).is_err()
        );
        assert!(
            crate::filesystem::rename_no_replace_impl(Path::new("a"), Path::new("b\0c")).is_err()
        );
        assert!(
            crate::filesystem::acquire_planning_artifact_lock_impl(Path::new(
                "/nonexistent-no-mistakes-validation-root/lock"
            ))
            .is_err()
        );
    }
}

#[test]
fn analyze_project_rejects_invalid_report_options_across_report_types() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/simple/fixture");
    for report in [
        json!({ "type": "symbols", "unknownField": true }),
        json!({ "type": "importUsages", "unknownField": true }),
        json!({ "type": "queues", "unknownField": true }),
        json!({ "type": "flow", "unknownField": true }),
        json!({ "type": "playwrightCheck", "unknownField": true }),
        json!({ "type": "effects", "unknownField": true }),
        json!({ "type": "rscCallers", "unknownField": true }),
        json!({ "type": "dependencies", "unknownField": true }),
        json!({ "type": "check", "unknownField": true }),
        json!({ "type": "resolveConfig", "unknownField": true }),
        json!({ "type": "lockStatus", "unknownField": true }),
        json!({ "type": "serverContracts", "unknownField": true }),
    ] {
        assert!(crate::analyze_project::analyze_project_value_impl(json!({
            "root": fixture,
            "reports": [report]
        }))
        .is_err());
    }
}

#[test]
fn rlib_lockfile_and_tsconfig_alias_error_paths_match_core_contracts() {
    use no_mistakes::codebase::lockfile::{
        binary_lockfile_warning, detect_manager, detect_planning_manager, diff, is_binary_lockfile,
        parse_lockfile, LockfileDiff, PackageManager,
    };
    let managers = [
        ("package-lock.json", PackageManager::Npm, "npm"),
        ("pnpm-lock.yaml", PackageManager::Pnpm, "pnpm"),
        ("yarn.lock", PackageManager::Yarn, "yarn"),
        ("bun.lock", PackageManager::Bun, "bun"),
        ("poetry.lock", PackageManager::Poetry, "poetry"),
        ("uv.lock", PackageManager::Uv, "uv"),
        ("Pipfile.lock", PackageManager::Pipfile, "pipfile"),
        ("go.mod", PackageManager::Go, "go"),
        ("Cargo.lock", PackageManager::Cargo, "cargo"),
        ("Gemfile.lock", PackageManager::Gemfile, "gemfile"),
        ("composer.lock", PackageManager::Composer, "composer"),
    ];
    for (file, manager, name) in managers {
        assert_eq!(detect_manager(file), Some(manager));
        assert_eq!(manager.as_str(), name);
        let _ = parse_lockfile(manager, "");
    }
    assert_eq!(detect_manager("unknown.lock"), None);
    assert!(detect_planning_manager("pnpm-lock.yaml").is_some());
    assert!(detect_planning_manager("Cargo.lock").is_none());
    assert!(is_binary_lockfile("bun.lockb"));
    assert!(binary_lockfile_warning("bun.lockb").contains("bun.lockb"));
    let empty_diff = diff(&[], &[]);
    assert!(empty_diff.is_empty());
    assert_eq!(empty_diff.all_changed_names().count(), 0);
    let _ = LockfileDiff::default();

    use no_mistakes::codebase::rules::tsconfig_alias_folder_mapping::{
        check as check_alias, RULE_ID,
    };
    use no_mistakes::config::v2::{
        schema::{RuleDef, RuleScope},
        NoMistakesConfig,
    };
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/tsconfig-alias-folder-mapping/fixture");
    let yaml = "tsconfig: tsconfig.json\nbaseDir: backend\nmappings:\n  - prefix: \"@agents\"\n    root: agents\n";
    let mut cfg = NoMistakesConfig::default();
    cfg.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    assert_eq!(
        check_alias(&fixture.join("extends-missing"), &cfg)
            .unwrap()
            .len(),
        1
    );
    let mut skip_cfg = NoMistakesConfig::default();
    skip_cfg.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(&yaml.replace("tsconfig.json", "nonexistent.json")).unwrap(),
        include: vec!["[".to_string()],
        ..Default::default()
    });
    assert!(check_alias(&fixture.join("pass"), &skip_cfg)
        .unwrap()
        .is_empty());
}
