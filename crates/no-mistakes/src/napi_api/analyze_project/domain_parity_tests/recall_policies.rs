fn recall_check_report(root: &Path, config: &str) -> Value {
    let standalone = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": config}),
        ))
        .unwrap(),
    );
    let aggregate = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": config, "reports": [{"type": "check"}]
        })))
        .unwrap(),
    );
    assert_eq!(aggregate["reports"][0]["result"], standalone, "{config}");
    standalone
}

#[test]
fn forbidden_calls_unknown_default_matches_standalone_and_explicit_policies() {
    let root = category_fixture("rules", "forbidden-calls/consumer-matrix");
    let mut reports = Vec::new();
    for (config, expected) in [
        ("configs/unknown-calls-default.yml", 1),
        ("configs/unknown-calls-finding.yml", 1),
        ("configs/unknown-calls-ignore.yml", 0),
    ] {
        let standalone = recall_check_report(&root, config);
        assert_eq!(
            standalone["rules"].as_array().unwrap().len(),
            expected,
            "{config}"
        );
        assert_eq!(standalone["warnings"], json!([]), "{config}");
        reports.push(standalone);
    }
    // Application names are echoed in messages; only the policy differs here.
    let message = reports[0]["rules"][0]["message"].as_str().unwrap();
    reports[0]["rules"][0]["message"] =
        json!(message.replace("unknown-calls-default", "unknown-calls-finding"));
    assert_eq!(reports[0], reports[1]);
}

#[test]
fn postgres_ddl_recovery_policies_match_standalone_checks() {
    let root = category_fixture("rules", "postgres-ddl-unanalyzable");
    for (config, findings, advisories) in [
        ("report.yml", 22, 0),
        ("advise.yml", 0, 22),
        ("ignore.yml", 0, 0),
        ("low.yml", 22, 0),
        ("narrow.yml", 2, 0),
    ] {
        let standalone = recall_check_report(&root, config);
        assert_eq!(
            standalone["rules"].as_array().unwrap().len(),
            findings,
            "{config}"
        );
        assert_eq!(
            standalone["advisories"].as_array().unwrap().len(),
            advisories,
            "{config}"
        );
        if config == "low.yml" {
            assert_eq!(standalone["warnings"].as_array().unwrap().len(), 11);
        } else {
            assert_eq!(standalone["warnings"], json!([]), "{config}");
        }
    }
}

#[test]
fn postgres_ddl_nested_locations_and_suppressions_match_standalone() {
    let root = category_fixture("rules", "postgres-ddl-unanalyzable");
    let nested = recall_check_report(&root.join("nested"), "report.yml");
    let findings = nested["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{nested:#?}");
    assert_eq!(findings[0]["line"], 4);
    assert_eq!(findings[1]["line"], 5);
    assert!(findings[0]["target"].as_str().unwrap().contains("alpha"));
    assert!(findings[1]["target"].as_str().unwrap().contains("beta"));
    let suppressed = recall_check_report(&root.join("suppress"), "report.yml");
    let findings = suppressed["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{suppressed:#?}");
    assert_eq!(findings[0]["line"], 4);
    let advised = recall_check_report(&root.join("suppress"), "advise.yml");
    assert_eq!(advised["rules"], json!([]));
    let advisories = advised["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 1, "{advised:#?}");
    assert_eq!(advisories[0]["line"], 4);
    let disabled = recall_check_report(&root.join("suppress-file"), "report.yml");
    assert_eq!(disabled["rules"], json!([]));
    let dynamic = recall_check_report(&root.join("dynamic"), "report.yml");
    let findings = dynamic["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{dynamic:#?}");
    assert_eq!(findings[0]["line"], 2);
}

#[test]
fn valid_unsupported_sql_diagnostics_preserve_node_policy_guidance() {
    let root = category_fixture("rules", "postgres-ddl-unanalyzable").join("unsupported-valid");
    let report = recall_check_report(&root, "report.yml");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 3, "{report:#?}");
    for finding in findings {
        let message = finding["message"].as_str().unwrap();
        assert!(message.contains("no-mistakes-disable-next-line postgres-fk-index"));
        assert!(message.contains("options.unanalyzable"));
    }
}

#[test]
fn copy_stdin_data_and_terminators_do_not_add_node_findings() {
    let root = category_fixture("rules", "postgres-ddl-unanalyzable").join("copy");
    let report = recall_check_report(&root, "positive.yml");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report:#?}");
    assert_eq!(findings[0]["file"], "migrations/001.sql");
    assert_eq!(findings[0]["line"], 8);
    assert_eq!(findings[0]["target"], "FROBULATE accounts");
}

#[test]
fn procedural_control_noise_keeps_node_reports_and_real_statement_lines_in_sync() {
    let root = category_fixture("rules", "postgres-ddl-unanalyzable").join("procedural");
    let report = recall_check_report(&root, "report.yml");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 69, "{report:#?}");
    assert!(findings.iter().all(|finding| {
        !["migrations/valid.sql", "migrations/quoted-do-bodies.sql"]
            .contains(&finding["file"].as_str().unwrap())
    }));
    let lines: Vec<_> = findings.iter()
        .filter(|finding| finding["file"] == "migrations/top-level.sql")
        .map(|finding| finding["line"].as_u64().unwrap()).collect();
    assert_eq!(lines, vec![7, 13, 19, 21]);
}
