use serde_json::{json, Value};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-lock-inventory/fixture")
            .join(name),
    )
}

fn parse_json(value: &str) -> Value {
    serde_json::from_str(value).unwrap()
}

#[test]
fn lock_inventory_napi_and_check_reports_match_library_results() {
    for name in [
        "added",
        "clean",
        "unknown",
        "gap-multiplicity",
        "moved",
        "removed",
        "suppressed",
        "helper-only",
        "routine",
        "imported-helper",
        "callbacks",
    ] {
        let root = fixture(name);
        let mut options = json!({ "root": root, "config": ".no-mistakes.yml" });
        let library = crate::postgres_lock_inventory::run(
            &crate::postgres_lock_inventory::PostgresLockInventoryOptions {
                root: root.clone(),
                config: Some(PathBuf::from(".no-mistakes.yml")),
                write_baseline: false,
            },
        )
        .unwrap();
        let napi = super::postgres_lock_inventory_json_impl(options.clone()).unwrap();
        assert_eq!(
            parse_json(&napi),
            serde_json::to_value(&library).unwrap(),
            "{name}"
        );

        match name {
            "callbacks" => assert_eq!(library.added.len(), 1, "{name}"),
            "added" | "helper-only" => assert!(!library.added.is_empty(), "{name}"),
            "unknown" | "routine" => assert!(!library.unanalyzed.is_empty(), "{name}"),
            "removed" => assert!(!library.removed.is_empty(), "{name}"),
            "suppressed" => assert_eq!(library.added.len(), 1, "{name}"),
            "imported-helper" => assert_eq!(
                library
                    .added
                    .iter()
                    .map(|site| (site.file.as_str(), site.line))
                    .collect::<Vec<_>>(),
                [("app.ts", 8), ("app.ts", 15)]
            ),
            _ => {}
        }
        let expected_findings = library.added.len()
            + library.unanalyzed.len()
            + usize::from(!library.removed.is_empty());
        let expected_suppressed = usize::from(matches!(name, "suppressed" | "imported-helper"));
        options["includeSuppressed"] = json!(true);
        let standalone = parse_json(
            &super::check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                .unwrap(),
        );
        let aggregate = parse_json(
            &super::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({
                        "root": root,
                        "config": ".no-mistakes.yml",
                        "reports": [{ "type": "check", "includeSuppressed": true }]
                    })
                    .to_string(),
                ),
            )
            .unwrap(),
        );
        let findings = standalone["rules"].as_array().unwrap();
        let suppressed = standalone["suppressed"].as_array().unwrap();
        assert_eq!(
            findings.len() + suppressed.len(),
            expected_findings,
            "{name}: {standalone}"
        );
        assert_eq!(
            suppressed.len(),
            expected_suppressed,
            "{name}: {standalone}"
        );
        assert!(
            findings
                .iter()
                .all(|finding| finding["rule"] == "postgres-lock-inventory"),
            "{name}: {standalone}"
        );
        if name == "suppressed" {
            let entry = &suppressed[0];
            assert_eq!(entry["rule"], "postgres-lock-inventory");
            assert_eq!(entry["file"], library.added[0].file);
            assert_eq!(entry["line"], library.added[0].line);
            assert_eq!(entry["directive"], json!({ "kind": "nextLine", "line": 1 }));
        }
        if name == "imported-helper" {
            assert_eq!(suppressed[0]["line"], 15);
            assert_eq!(
                suppressed[0]["directive"],
                json!({ "kind": "nextLine", "line": 14 })
            );
        }
        assert_eq!(aggregate["reports"][0]["result"], standalone, "{name}");
    }
}
