use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_callback_forwarded_literals_check_json_snapshot() {
    const RULE: &str = "postgres-require-query-annotation";
    let root = rules_path(RULE, "fixture", "callback-forwarded-literals");
    let cases = ["advise", "report", "ignore"].map(|policy| {
        (
            policy,
            root.clone(),
            rules_path(RULE, "configs", &format!("callback-forwarded-{policy}.yml")),
        )
    });
    let reports =
        assert_saved_fixture_report("postgres_callback_forwarded_literals", &cases, &[], &[]);
    for policy in ["advise", "report", "ignore"] {
        let case = &reports[policy];
        assert_eq!(case["exitCode"], 1, "{policy}: {case}");
        let report = &case["report"];
        assert_eq!(report["warnings"], serde_json::json!([]));
        let findings = report["rules"].as_array().unwrap();
        let mut finding_sites: Vec<_> = findings
            .iter()
            .map(|finding| {
                assert_eq!(finding["target"], "annotation");
                (
                    finding["file"].as_str().unwrap(),
                    finding["line"].as_u64().unwrap(),
                )
            })
            .collect();
        finding_sites.sort_unstable();
        let expected_findings = if policy == "report" {
            vec![
                ("annotation.mts", 24),
                ("annotation.mts", 28),
                ("annotation.mts", 32),
                ("annotation.mts", 42),
                ("shared.mts", 4),
            ]
        } else {
            vec![("annotation.mts", 24), ("shared.mts", 4)]
        };
        assert_eq!(finding_sites, expected_findings, "{policy}: {report}");
        let mut advisory_sites: Vec<_> = report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| {
                (
                    finding["file"].as_str().unwrap(),
                    finding["line"].as_u64().unwrap(),
                )
            })
            .collect();
        advisory_sites.sort_unstable();
        let expected_advisories = if policy == "advise" {
            vec![
                ("annotation.mts", 24),
                ("annotation.mts", 28),
                ("annotation.mts", 32),
                ("annotation.mts", 42),
                ("external.mts", 4),
                ("shared.mts", 4),
            ]
        } else {
            vec![]
        };
        assert_eq!(advisory_sites, expected_advisories, "{policy}: {report}");
    }
}

#[test]
fn postgres_imported_callback_prefix_check_json_snapshot() {
    const RULE: &str = "postgres-require-query-annotation";
    let root = rules_path(RULE, "fixture", "imported-callback-prefix");
    let cases = ["advise", "report", "ignore"].map(|policy| {
        (
            policy,
            root.clone(),
            rules_path(
                RULE,
                "configs",
                &format!("imported-callback-prefix-{policy}.yml"),
            ),
        )
    });
    let reports =
        assert_saved_fixture_report("postgres_imported_callback_prefix", &cases, &[], &[]);
    for policy in ["advise", "report", "ignore"] {
        let report = &reports[policy]["report"];
        assert_eq!(reports[policy]["exitCode"], 1);
        assert_eq!(report["warnings"], serde_json::json!([]));
        for category in ["rules", "advisories"] {
            let mut sites = report[category]
                .as_array()
                .unwrap()
                .iter()
                .map(|finding| {
                    (
                        finding["file"].as_str().unwrap(),
                        finding["line"].as_u64().unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            sites.sort_unstable();
            let known = vec![("callers.mts", 9), ("callers.mts", 10), ("callers.mts", 11)];
            let unresolved = vec![
                ("callers.mts", 14),
                ("guards.mts", 8),
                ("guards.mts", 9),
                ("guards.mts", 10),
                ("guards.mts", 12),
                ("guards.mts", 15),
                ("guards.mts", 16),
                ("guards.mts", 18),
            ];
            let mut expected = if category == "rules" || policy == "advise" {
                known
            } else {
                vec![]
            };
            if category == "rules" && policy == "report"
                || category == "advisories" && policy == "advise"
            {
                expected.extend(unresolved);
            }
            expected.sort_unstable();
            assert_eq!(sites, expected, "{policy} {category}: {report}");
        }
    }
}
