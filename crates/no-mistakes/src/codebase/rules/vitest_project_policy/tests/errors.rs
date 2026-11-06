use super::*;

#[test]
fn invalid_policy_options_return_actionable_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-project-policy/fixture/options-errors"),
    );
    let files = vec![root.join("vitest.config.ts")];
    let sources = SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    ));
    for (case, message) in [
        ("no-config", "requires options.config"),
        ("array-required", "scalar JSON literals"),
        ("object-required", "scalar JSON literals"),
        ("empty-property", "nonempty direct test property names"),
        ("dotted-property", "nonempty direct test property names"),
        ("parity-length", "array of length 2"),
        ("unknown-option", "unknown field"),
    ] {
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(format!("{case}.yml"))))
                .unwrap();
        let error = diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
            .err()
            .unwrap_or_else(|| panic!("{case} must reject invalid options"));
        assert!(format!("{error:#}").contains(message), "{case}: {error:#}");
    }
}

#[test]
fn unavailable_config_sources_remain_incomplete() {
    for case in [
        "missing-source",
        "malformed-source",
        "nonliteral-root",
        "missing-root-test",
        "projects-nonarray",
        "missing-helper",
        "malformed-helper",
    ] {
        let result = run(case);
        assert!(result.findings.is_empty(), "{case}: {:?}", result.findings);
        assert!(!result.incomplete.is_empty(), "{case}");
        assert!(
            result.incomplete.iter().all(|failure| {
                failure.consumer == RULE_ID
                    && failure.file.is_some()
                    && failure.message.contains("literal")
            }),
            "{case}: {:?}",
            result.incomplete
        );
    }
}

#[test]
fn unsupported_inputs_do_not_supply_config_values() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-project-policy/fixture/options-errors"),
    );
    for case in [
        "suffix-undeclared",
        "wrapper-type-only",
        "helper-type-only",
        "update-mutation",
        "delete-mutation",
    ] {
        let files = vec![root.join(format!("{case}.ts")), root.join("helper.ts")];
        let sources = SourceStore::new(std::sync::Arc::new(
            crate::codebase::ts_source::FileInventory::from_paths(&files),
        ));
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(format!("{case}.yml"))))
                .unwrap();
        let result =
            diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
        assert!(result.findings.is_empty(), "{case}: {:?}", result.findings);
        assert!(!result.incomplete.is_empty(), "{case}");
    }
}
