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
