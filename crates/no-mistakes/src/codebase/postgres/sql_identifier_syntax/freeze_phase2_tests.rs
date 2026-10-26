use std::collections::BTreeSet;

#[test]
fn proven_freeze_preserves_actual_executor_consumers_and_factory_metadata() {
    let root = crate::test_support::rule_fixture_root("postgres-no-offset").join("freeze-phase2");
    let files = [
        "intrinsic-valid.ts",
        "intrinsic-shadowed.ts",
        "intrinsic-mutated.ts",
        "intrinsic-escaped.ts",
        "intrinsic-globalthis.ts",
        "factory-options.ts",
        "options-transfers.ts",
        "guard-expressions.ts",
        "producer-expressions.ts",
        "bounded-carriers.ts",
        "transfer-patterns.ts",
        "carrier-properties.ts",
        "intrinsic-disabled.ts",
        "factory-return-data.ts",
    ];
    let mut expected_advice = BTreeSet::new();
    let mut expected_clear = BTreeSet::new();
    let mut non_sql = BTreeSet::new();
    let mut source_by_file = std::collections::BTreeMap::new();
    for name in files {
        let source = std::fs::read_to_string(root.join("src").join(name)).unwrap();
        for (index, line) in source.lines().enumerate() {
            let item = (format!("src/{name}"), index + 1);
            if line.contains("// ADVISE") {
                expected_advice.insert(item.clone());
            }
            if line.contains("// CLEAR") {
                expected_clear.insert(item.clone());
            }
            if line.contains("// NON_SQL") {
                non_sql.insert(item);
            }
        }
        source_by_file.insert(format!("src/{name}"), source);
    }
    for (policy, config) in [
        ("advise", ".no-mistakes-low.yml"),
        ("report", ".no-mistakes-low-report.yml"),
        ("ignore", ".no-mistakes-low-ignore.yml"),
    ] {
        let report: serde_json::Value = serde_json::from_str(
            &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                serde_json::json!({ "root": root, "config": root.join(config),
                    "includeSqlVerdicts": true }),
            ))
            .unwrap(),
        )
        .unwrap();
        assert!(
            report["warnings"].as_array().unwrap().is_empty(),
            "{report}"
        );
        for rule in [
            "postgres-no-offset",
            "postgres-no-unbounded-sql-identifier",
            "postgres-no-mutable-sql-registry",
            "postgres-no-transformed-uuid-predicates",
        ] {
            let locations = |rows: &[serde_json::Value]| {
                rows.iter()
                    .filter(|row| row["rule"] == rule)
                    .map(|row| {
                        (
                            row["file"].as_str().unwrap().to_string(),
                            row["line"].as_u64().unwrap() as usize,
                        )
                    })
                    .collect::<BTreeSet<_>>()
            };
            let advice = locations(report["advisories"].as_array().unwrap());
            let findings = locations(report["rules"].as_array().unwrap());
            assert_eq!(
                advice,
                if policy == "advise" {
                    expected_advice.clone()
                } else {
                    BTreeSet::new()
                },
                "{policy}/{rule}: {report}"
            );
            assert_eq!(
                findings,
                if policy == "report" {
                    expected_advice.clone()
                } else {
                    BTreeSet::new()
                },
                "{policy}/{rule}: {report}"
            );
            assert!(advice.is_disjoint(&non_sql), "{policy}/{rule}: {report}");
            assert!(findings.is_disjoint(&non_sql), "{policy}/{rule}: {report}");
            let mut actual_verdicts = BTreeSet::new();
            for row in report["sqlVerdicts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row["rule"] == rule)
            {
                let file = row["file"].as_str().unwrap();
                let source = &source_by_file[file];
                let line = crate::codebase::ts_source::byte_offset_to_line(
                    source,
                    row["span"][0].as_u64().unwrap() as usize,
                ) as usize;
                actual_verdicts.insert((
                    file.to_string(),
                    line,
                    row["status"].as_str().unwrap().to_string(),
                ));
                if expected_clear.contains(&(file.to_string(), line)) {
                    assert_eq!(row["status"], "proven-clear", "{policy}/{rule}: {row}");
                }
                if expected_advice.contains(&(file.to_string(), line)) {
                    assert_eq!(row["status"], "abstain", "{policy}/{rule}: {row}");
                }
            }
            let expected_verdicts = expected_clear
                .iter()
                .map(|(file, line)| (file.clone(), *line, "proven-clear".to_string()))
                .chain(
                    expected_advice
                        .iter()
                        .map(|(file, line)| (file.clone(), *line, "abstain".to_string())),
                )
                .collect::<BTreeSet<_>>();
            assert_eq!(
                actual_verdicts, expected_verdicts,
                "{policy}/{rule}: {report}"
            );
        }
    }
}
