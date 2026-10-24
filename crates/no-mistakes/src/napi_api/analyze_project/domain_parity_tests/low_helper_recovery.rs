fn helper_recovery_root() -> PathBuf {
    repo_fixture(&["fixtures", "napi", "low-helper-recovery"])
}

fn helper_recovery_check(root: &Path, config: &str) -> Value {
    parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": config, "includeSqlVerdicts": true
        })))
        .unwrap(),
    )
}

fn helper_recovery_line(source: &str, marker: &str) -> usize {
    source
        .lines()
        .position(|line| line.contains(marker))
        .unwrap()
        + 1
}

#[test]
fn prepared_helper_recovery_preserves_findings_parses_and_application_scope() {
    // One test exclusively owns this physical root, including its Rayon parse counter.
    let root = helper_recovery_root();
    let helpers = std::fs::read_to_string(root.join("src/helpers.mts")).unwrap();
    let caller = std::fs::read_to_string(root.join("src/policy-consumers.mts")).unwrap();
    let transports = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let origin = helper_recovery_line(&helpers, "OFFSET_ORIGIN");
    let mixed_origin = helper_recovery_line(&helpers, "SQL` OFFSET 3`");
    let residual = helper_recovery_line(&caller, "OFFSET_WITH_RESIDUAL");
    for (config, advises) in [
        (".no-mistakes-low.yml", true),
        (".no-mistakes-low-ignore.yml", false),
    ] {
        let standalone = helper_recovery_check(&root, config);
        crate::ast::begin_parse_count(&root);
        let aggregate = parse_json(
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
                "root": root, "config": config,
                "reports": [{"type": "check", "includeSqlVerdicts": true}]
            })))
            .unwrap(),
        );
        let counts = crate::ast::finish_parse_count(&root);
        let result = &aggregate["reports"][0]["result"];
        assert_eq!(result, &standalone);
        assert!(result["warnings"].as_array().unwrap().is_empty());
        for line in [origin, mixed_origin] {
            assert!(
                result["rules"].as_array().unwrap().iter().any(|row| {
                    row["rule"] == "postgres-no-offset"
                        && row["file"] == "src/helpers.mts"
                        && row["line"] == line
                }),
                "helper SQL must remain a physical finding under {config}: {result}"
            );
        }
        assert_eq!(
            result["advisories"].as_array().unwrap().iter().any(|row| {
                row["rule"] == "postgres-no-offset"
                    && row["file"] == "src/policy-consumers.mts"
                    && row["line"] == residual
            }),
            advises,
            "{result}"
        );
        // Both typed helpers execute modeled SQL; their argument references are not obligations.
        for marker in ["forward(query, id)", "executeOptions(options, id)"] {
            let line = helper_recovery_line(&transports, marker);
            for field in ["rules", "advisories"] {
                assert!(
                    !result[field]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|row| { row["file"] == "src/query.mts" && row["line"] == line }),
                    "{marker}: {result}"
                );
            }
        }
        for path in [
            "src/helpers.mts",
            "src/policy-consumers.mts",
            "src/query.mts",
        ] {
            assert_eq!(counts.get(&root.join(path)), Some(&1), "{counts:?}");
        }
        assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    }
    helper_recovery_keeps_low_max_and_scoped_application_outputs();
}

fn helper_application_verdicts(report: &Value, name: &str, index: usize) -> Vec<Value> {
    let mut rows = report["sqlVerdicts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["application"]["name"] == name)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        !rows.is_empty(),
        "application {name} must actually execute: {report}"
    );
    for row in &mut rows {
        assert_eq!(row["application"]["index"], index, "{row}");
        row.as_object_mut().unwrap().remove("application");
    }
    rows.sort_by_key(Value::to_string);
    rows
}

fn helper_recovery_keeps_low_max_and_scoped_application_outputs() {
    let root = helper_recovery_root();
    let mixed = helper_recovery_check(&root, ".no-mistakes-api-mixed.yml");
    let aggregate = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": ".no-mistakes-api-mixed.yml",
            "reports": [{"type": "check", "includeSqlVerdicts": true}]
        })))
        .unwrap(),
    );
    assert_eq!(aggregate["reports"][0]["result"], mixed);
    assert!(mixed["warnings"].as_array().unwrap().is_empty());
    for (name, index, config) in [
        ("api-wide-low", 0, ".no-mistakes-api-wide-low.yml"),
        ("api-max", 1, ".no-mistakes-api-max.yml"),
        ("api-narrow-low", 2, ".no-mistakes-api-narrow-low.yml"),
    ] {
        let baseline = helper_recovery_check(&root, config);
        assert_eq!(
            helper_application_verdicts(&mixed, name, index),
            helper_application_verdicts(&baseline, name, 0),
            "{name}"
        );
    }
    let caller = std::fs::read_to_string(root.join("src/policy-consumers.mts")).unwrap();
    let span = caller
        .find("read(SQL`SELECT 1`.append(offsetFragment")
        .unwrap();
    for (name, index, status) in [
        ("api-wide-low", 0, "violation"),
        ("api-narrow-low", 2, "abstain"),
    ] {
        assert!(
            helper_application_verdicts(&mixed, name, index)
                .iter()
                .any(|row| {
                    row["file"] == "src/policy-consumers.mts"
                        && row["span"][0] == span
                        && row["status"] == status
                }),
            "excluded helpers must not borrow another application's proof: {mixed}"
        );
    }
}
