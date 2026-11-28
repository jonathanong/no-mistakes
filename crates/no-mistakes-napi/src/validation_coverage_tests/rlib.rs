use serde_json::json;
use std::path::{Path, PathBuf};

#[test]
fn observed_and_filesystem_reject_malformed_inputs() {
    use napi::bindgen_prelude::Buffer;
    fn ok_json(_: serde_json::Value) -> napi::Result<String> {
        Ok("{}".to_string())
    }

    let bad_utf8 = Buffer::from(vec![0xff, 0xfe]);
    assert!(crate::async_task::observed::execute_json(&bad_utf8, ok_json, false).is_err());
    let bad_json = Buffer::from(b"{".to_vec());
    assert!(crate::async_task::observed::execute_json(&bad_json, ok_json, false).is_err());
    let bad_diag = Buffer::from(br#"{"diagnosticsId":123}"#.to_vec());
    assert!(crate::async_task::observed::execute_json(&bad_diag, ok_json, false).is_err());
    let ok_text = Buffer::from(br#"{"diagnosticsId":"obs-text-ok"}"#.to_vec());
    assert!(crate::async_task::observed::execute_json(&ok_text, ok_json, true).is_ok());
    let err_diag = Buffer::from(br#"{"diagnosticsId":"obs-err"}"#.to_vec());
    assert!(crate::async_task::observed::execute_json(
        &err_diag,
        |_| Err(napi::Error::from_reason("boom")),
        false
    )
    .is_err());
    assert!(
        crate::async_task::observed::execute_pure(json!({ "diagnosticsId": 123 }), ok_json)
            .is_err()
    );

    assert!(crate::analyze_project::analyze_project_json_impl(json!({
        "unknownField": true
    }))
    .is_err());
    assert!(crate::analyze_project::analyze_project_value_impl(json!({
        "unknownField": true
    }))
    .is_err());
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(crate::lockfile_diff_json_impl(json!({
        "root": repo_root,
        "base": "HEAD",
        "head": "nonexistent-ref-no-mistakes-test"
    }))
    .is_err());
    let bad = json!({ "unknownField": true });
    assert!(crate::queue_check_json_impl(bad.clone()).is_err());
    assert!(crate::queue_edges_json_impl(bad.clone()).is_err());
    assert!(crate::queue_related_json_impl(bad.clone()).is_err());
    assert!(crate::queues_json_impl(bad.clone()).is_err());
    assert!(crate::playwright_check_json_impl(bad.clone()).is_err());
    assert!(crate::playwright_edges_json_impl(bad.clone()).is_err());
    assert!(crate::playwright_related_json_impl(bad.clone()).is_err());
    assert!(crate::playwright_tests_json_impl(bad).is_err());

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

    let mut pnpm_cfg = NoMistakesConfig::default();
    pnpm_cfg.rules.push(RuleDef {
        rule: no_mistakes::codebase::rules::pnpm_overrides_ban::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    assert!(no_mistakes::codebase::rules::pnpm_overrides_ban::check(
        &fixture.join("pass"),
        &pnpm_cfg
    )
    .unwrap()
    .is_empty());
}
