use super::check;

#[test]
fn shared_predicate_sites_keep_every_missing_requirement_in_one_diagnostic_per_site() {
    for name in ["multiple-requirements", "complementary-requirements"] {
        let report = check("postgres-required-predicates", name);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 2, "{name}: {report}");
        for finding in findings {
            assert_eq!(finding["line"], 2, "{name}: {report}");
            assert_eq!(finding["target"], "accounts", "{name}: {report}");
            assert!(finding.get("import").is_none(), "{name}: {report}");
        }
        let messages = findings
            .iter()
            .map(|finding| finding["message"].as_str().unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        for required in [
            "active IS TRUE",
            "deleted_at IS NULL",
            "tenant_id",
            "region_id",
        ] {
            assert_eq!(messages.matches(required).count(), 1, "{name}: {report}");
        }
    }
}
