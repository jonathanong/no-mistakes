use super::*;

fn permissions_report(scenario: &str, policy: &str, audit: bool) -> serde_json::Value {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-job-permissions");
    let options = json!({
        "root": base.join("fixture").join(scenario),
        "config": base.join("configs").join(format!("{policy}.yml")),
        "includeSuppressed": audit,
    });
    crate::test_support::without_hygiene_advisory_json(
        serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
        )
        .unwrap(),
    )
}

#[test]
fn check_json_job_permissions_includes_reusable_jobs_and_policy_channels_once() {
    let fail = permissions_report("fail", "default", false);
    assert_eq!(fail["rules"].as_array().unwrap().len(), 4, "{fail}");
    assert!(fail["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["target"] == "call"));
    for (policy, findings, advisories) in [("default", 2, 0), ("advise", 1, 1), ("ignore", 1, 0)] {
        let report = permissions_report("mixed", policy, false);
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{report}"
        );
        assert!(report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["target"] == "build"));
    }
}

#[test]
fn check_json_job_permissions_preserves_suppression_audit_and_visible_findings() {
    for scenario in ["suppression", "permission-suppression"] {
        let plain = permissions_report(scenario, "default", false);
        let audit = permissions_report(scenario, "default", true);
        assert_eq!(plain["rules"], audit["rules"]);
        assert_eq!(audit["rules"].as_array().unwrap().len(), 1, "{audit}");
        assert_eq!(audit["rules"][0]["target"], "visible");
        let suppressed = audit["suppressed"].as_array().unwrap();
        assert_eq!(suppressed.len(), 3, "{audit}");
        for kind in ["file", "line", "nextLine"] {
            assert!(
                suppressed
                    .iter()
                    .any(|row| row["directive"]["kind"] == kind),
                "{audit}"
            );
        }
    }
}
