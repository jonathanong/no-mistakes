#[test]
fn selector_advisories_group_plain_sites_and_bound_shared_site_rows() {
    let root = super::fixture("selector-advisory-grouping");
    for (config, channel, count) in [
        (".no-mistakes.yml", "advisories", 5),
        ("report.yml", "rules", 5),
        ("ignore.yml", "advisories", 0),
    ] {
        let input = crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "root": root, "config": root.join(config)
            })
            .to_string(),
        );
        let report: serde_json::Value =
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
        let definite: Vec<_> = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["target"] != "unanalyzable")
            .collect();
        assert_eq!(definite.len(), 1, "{report}");
        assert_eq!(definite[0]["target"], "onlyTests");
        let uncertain: Vec<_> = report[channel]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["target"] == "unanalyzable")
            .collect();
        assert_eq!(uncertain.len(), count, "{report}");
        if count == 0 {
            continue;
        }
        let rows_for = |file: &str| {
            uncertain
                .iter()
                .copied()
                .filter(|row| row["file"] == file)
                .collect::<Vec<_>>()
        };
        let plain = rows_for("src/many-sites.mts");
        assert_eq!(plain.len(), 1, "{report}");
        let message = plain[0]["message"].as_str().unwrap();
        assert!(message.contains("`loneExport`"), "{message}");
        for line in 1..=4 {
            assert!(
                message.contains(&format!("src/many-sites.mts:{line}")),
                "{message}"
            );
        }
        let tied = rows_for("src/tied-consumer.mts");
        assert_eq!(tied.len(), 2, "{report}");
        for name in ["tiedFirst", "tiedSecond"] {
            let messages = tied
                .iter()
                .filter_map(|row| row["message"].as_str())
                .filter(|message| message.contains(&format!("`{name}`")))
                .collect::<Vec<_>>()
                .join(" ");
            for line in 1..=2 {
                assert!(
                    messages.contains(&format!("src/tied-consumer.mts:{line}")),
                    "{messages}"
                );
            }
        }
        let shared = rows_for("src/shared-consumer.mts");
        assert_eq!(shared.len(), 1, "{report}");
        for name in ["first", "second", "third"] {
            assert!(shared[0]["message"]
                .as_str()
                .unwrap()
                .contains(&format!("`{name}`")));
        }
        let unique = rows_for("src/unique-consumer.mts");
        assert_eq!(unique.len(), 1, "{report}");
        let message = unique[0]["message"].as_str().unwrap();
        assert!(message.contains("`first`"), "{message}");
        assert!(message.contains("src/unique-consumer.mts:1"), "{message}");
        assert!(message.contains("src/unique-consumer.mts:2"), "{message}");
    }
}

#[test]
fn selector_advisory_bound_uses_globally_deduplicated_export_groups() {
    let root = super::fixture("selector-advisory-cross-origin");
    for (config, channel) in [(".no-mistakes.yml", "advisories"), ("report.yml", "rules")] {
        let input = crate::napi_api::options::test_json_arg(
            serde_json::json!({"root": root, "config": root.join(config)}).to_string(),
        );
        let report: serde_json::Value =
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
        let findings: Vec<_> = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["target"] != "unanalyzable")
            .collect();
        assert_eq!(findings.len(), 1, "{report}");
        assert_eq!(findings[0]["target"], "onlyTests");
        let advisories: Vec<_> = report[channel]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["target"] == "unanalyzable")
            .collect();
        assert_eq!(advisories.len(), 2, "{report}");
        for name in ["same", "left", "middle", "right"] {
            let messages = advisories
                .iter()
                .filter_map(|row| row["message"].as_str())
                .filter(|message| message.contains(&format!("`{name}`")))
                .collect::<Vec<_>>()
                .join(" ");
            assert!(messages.contains("src/y-consumer.mts:1"), "{messages}");
            if name == "same" {
                assert!(messages.contains("src/x-consumer.mts:1"), "{messages}");
            }
        }
    }
}

#[test]
fn selector_grouping_cannot_reintroduce_suppressed_export_advisories() {
    let root = super::fixture("selector-advisory-suppression");
    for config in [".no-mistakes.yml", "report.yml"] {
        let input = crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "root": root, "config": root.join(config)
            })
            .to_string(),
        );
        let report: serde_json::Value =
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{report}");
        assert_eq!(findings[0]["target"], "onlyTests");
        assert!(
            report["advisories"].as_array().unwrap().is_empty(),
            "{report}"
        );
    }
}

#[test]
fn selector_groups_keep_declared_default_export_names() {
    let root = super::fixture("selector-advisory-default");
    let input =
        crate::napi_api::options::test_json_arg(serde_json::json!({"root": root}).to_string());
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
    let advisories = report["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 1, "{report}");
    assert!(
        advisories[0]["message"]
            .as_str()
            .unwrap()
            .contains("symbol consumers of `namedDefault`"),
        "{report}"
    );
}
