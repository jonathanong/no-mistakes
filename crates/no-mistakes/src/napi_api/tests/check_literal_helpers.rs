use super::*;

#[test]
fn configured_literal_helpers_have_matching_low_and_max_policy_outcomes() {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("sql-literal-helpers");
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    for policy in ["report", "advise", "ignore"] {
        let mut prior = None;
        for effort in ["low", "max"] {
            let request = serde_json::json!({"root":root,
                "config":root.join(format!(".no-mistakes-{effort}-{policy}.yml")),
                "includeSqlVerdicts":true});
            let report: serde_json::Value = serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(request.to_string()))
                    .unwrap(),
            )
            .unwrap();
            let rows = report["rules"].as_array().unwrap();
            let verdicts = report["sqlVerdicts"].as_array().unwrap();
            let receivers = verdicts
                .iter()
                .filter(|row| row["subcheck"] == "unresolved-configured-executor-receiver")
                .collect::<Vec<_>>();
            assert_eq!(
                receivers.len(),
                1,
                "helper profiles retain receiver diagnostics: {report}"
            );
            assert_eq!(receivers[0]["advisory"], policy == "advise");
            assert_eq!(
                rows.iter()
                    .any(|row| row["target"] == "unresolved-configured-executor-receiver"),
                policy == "report"
            );
            assert!(
                !rows
                    .iter()
                    .any(|row| row["target"] == "unmatched-executor-name"),
                "PoolClient remains a matched configured name"
            );
            let mut statuses = Vec::new();
            for (index, line) in source.lines().enumerate() {
                let Some((_, marker)) = line.split_once("// ") else {
                    continue;
                };
                let expected = match marker {
                    "clear" => "proven-clear",
                    "missing" => "violation",
                    "unknown" => "abstain",
                    _ => continue,
                };
                let verdict = verdicts
                    .iter()
                    .find(|verdict| {
                        let start = verdict["span"][0].as_u64().unwrap() as usize;
                        crate::codebase::ts_source::byte_offset_to_line(&source, start) as usize
                            == index + 1
                    })
                    .unwrap_or_else(|| panic!("missing {effort} verdict for {line}: {report}"));
                assert_eq!(verdict["status"], expected, "{effort}/{policy}: {line}");
                assert_eq!(
                    verdict["advisory"],
                    marker == "unknown" && policy == "advise",
                    "{effort}/{policy}: {line}"
                );
                let findings = rows
                    .iter()
                    .filter(|row| row["line"] == index + 1)
                    .collect::<Vec<_>>();
                assert_eq!(
                    !findings.is_empty(),
                    marker == "missing" || (marker == "unknown" && policy == "report"),
                    "{effort}/{policy}: {line}"
                );
                if marker == "missing" {
                    assert!(findings[0]["message"]
                        .as_str()
                        .unwrap()
                        .contains("must start with a /* name */ annotation"));
                }
                statuses.push(verdict["status"].clone());
            }
            assert_eq!(statuses.len(), 25, "every saved executor has an oracle");
            if let Some(prior) = &prior {
                assert_eq!(&statuses, prior, "max must retain low's definite findings");
            }
            prior = Some(statuses);
        }
    }
}

#[test]
fn malformed_literal_helper_identities_are_public_configuration_errors() {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("sql-literal-helpers");
    for index in 0..7 {
        let request = serde_json::json!({"root":root,"config":root.join(format!(".no-mistakes-invalid-{index}.yml"))});
        let error = check_json_impl(crate::napi_api::options::test_json_arg(request.to_string()))
            .unwrap_err();
        assert!(error.to_string().contains("sqlLiteralHelpers"), "{error}");
    }
}

#[test]
fn mixed_helper_profiles_do_not_share_configuration_authority() {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("sql-literal-helpers");
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let request = serde_json::json!({"root":root,"config":root.join(".no-mistakes-mixed.yml"),"includeSqlVerdicts":true});
    let report: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(request.to_string())).unwrap(),
    )
    .unwrap();
    let verdicts = report["sqlVerdicts"].as_array().unwrap();
    for (needle, clear) in [
        (
            "client.query(copyFrom('/* named */ COPY users FROM STDIN')); // clear",
            "pg-copy-streams#from",
        ),
        ("client.query(otherFrom(", "other-copy-streams#from"),
    ] {
        let rows = verdicts
            .iter()
            .filter(|row| {
                let start = row["span"][0].as_u64().unwrap() as usize;
                source[start..].starts_with(needle)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            rows.len(),
            2,
            "each application retains its own profile: {report}"
        );
        for row in rows {
            let helper = row["profile"]["sqlLiteralHelpers"][0].as_str().unwrap();
            assert_eq!(
                row["status"],
                if helper == clear {
                    "proven-clear"
                } else {
                    "abstain"
                }
            );
        }
    }
}

#[test]
fn helper_configuration_preserves_ordinary_cross_file_max_inference() {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("sql-literal-helpers");
    let mut baseline = None;
    for config in [
        ".no-mistakes-plain-baseline.yml",
        ".no-mistakes-plain-max.yml",
    ] {
        let request =
            serde_json::json!({"root":root,"config":root.join(config),"includeSqlVerdicts":true});
        let report: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(request.to_string())).unwrap(),
        )
        .unwrap();
        assert_eq!(report["rules"], serde_json::json!([]), "{report}");
        let rows = report["sqlVerdicts"].as_array().unwrap();
        assert_eq!(rows.len(), 1, "{report}");
        assert_eq!(
            rows[0]["status"], "proven-clear",
            "ordinary imported helper is still traced"
        );
        if let Some(prior) = &baseline {
            assert_eq!(&report["advisories"], prior);
        }
        baseline = Some(report["advisories"].clone());
    }
}
