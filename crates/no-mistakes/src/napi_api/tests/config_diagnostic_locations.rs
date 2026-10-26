use super::*;

fn root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/config-diagnostic-locations/fixture")
            .join(name),
    )
}

fn check(input: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(input.to_string())).unwrap(),
    )
    .unwrap()
}

fn located(name: &str, config: Option<&str>) -> serde_json::Value {
    let root = root(name);
    let mut input = json!({ "root": root });
    if let Some(config) = config {
        input["config"] = json!(root.join(config));
    }
    let report = check(input.clone());
    input["reports"] = json!([{ "type": "check" }]);
    let project: serde_json::Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(input.to_string()),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        project["reports"][0]["result"], report,
        "{name}: analyzeProject diverged from check"
    );
    report
}

fn locations(report: &serde_json::Value) -> Vec<(String, String, u64, String)> {
    report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["rule"].as_str().unwrap().to_string(),
                finding["file"].as_str().unwrap().to_string(),
                finding["line"].as_u64().unwrap(),
                finding["message"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn custom_config_diagnostics_use_the_selected_file_and_authored_line() {
    let report = located("production-custom", Some("check.yml"));
    assert_eq!(
        locations(&report),
        vec![(
            "production-dependency-declarations".to_string(),
            "check.yml".to_string(),
            3,
            "production-dependency-declarations: workspaceRoots is required and must list at \
             least one workspace root; it is not inferred from the check root"
                .to_string(),
        )],
        "{report}"
    );

    let report = located("production-default", None);
    assert_eq!(locations(&report)[0].1, ".no-mistakes.yml", "{report}");
    assert_eq!(locations(&report)[0].2, 3, "{report}");

    let report = located("production-json", Some("check.json"));
    assert_eq!(locations(&report)[0].1, "check.json", "{report}");
    assert_eq!(locations(&report)[0].2, 4, "{report}");

    let report = located("forbidden-dependencies", Some("check.yml"));
    assert_eq!(
        locations(&report),
        vec![(
            "forbidden-dependencies".to_string(),
            "check.yml".to_string(),
            3,
            "forbidden-dependencies: each rule entry requires at least one root and \
             one forbiddenModules or forbiddenFiles entry"
                .to_string(),
        )],
        "{report}"
    );

    let report = located("forbidden-workspace", Some("check.yml"));
    assert_eq!(locations(&report).len(), 1, "{report}");
    assert_eq!(locations(&report)[0].0, "forbidden-workspace-closure");
    assert_eq!(locations(&report)[0].1, "check.yml");
    assert_eq!(locations(&report)[0].2, 3, "{report}");

    let report = located("entrypoint", Some("check.yml"));
    let mut entrypoint = locations(&report);
    entrypoint.sort();
    assert_eq!(
        entrypoint,
        vec![
            (
                "required-entrypoint-reachability".to_string(),
                "check.yml".to_string(),
                3,
                "required-entrypoint-reachability: each rule entry requires at least one \
                 entrypoint"
                    .to_string(),
            ),
            (
                "required-entrypoint-reachability".to_string(),
                "check.yml".to_string(),
                3,
                "required-entrypoint-reachability: each rule entry requires at least one \
                 sourceGlobs pattern"
                    .to_string(),
            ),
        ],
        "{report}"
    );

    let report = located("generated-predicates", Some("check.yml"));
    assert!(
        locations(&report).iter().any(|finding| {
            finding.0 == "postgres-generated-column-predicates"
                && finding.1 == "check.yml"
                && finding.2 == 9
                && finding
                    .3
                    .contains("extraGeneratedColumns entry: orders.created_at")
        }),
        "{report}"
    );

    let writes = locations(&located("generated-writes", Some("check.yml")));
    assert!(
        writes.iter().any(|finding| {
            finding.0 == "postgres-no-generated-column-writes"
                && finding.1 == "check.yml"
                && finding.2 == 9
                && finding.3.contains("extraGeneratedColumns entry")
        }),
        "{writes:?}"
    );
    assert!(
        writes.iter().any(|finding| {
            finding.0 == "postgres-no-generated-column-writes"
                && finding.1 == "check.yml"
                && finding.2 == 11
                && finding.3.contains("triggerMaintainedColumns entry")
        }),
        "{writes:?}"
    );
}
