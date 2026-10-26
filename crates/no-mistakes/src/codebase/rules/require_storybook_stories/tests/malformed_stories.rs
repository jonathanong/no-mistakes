use super::*;

fn fixture_config() -> NoMistakesConfig {
    crate::config::v2::load_v2_config(&fixture("malformed-stories"), None).unwrap()
}

#[test]
fn malformed_story_makes_only_its_application_incomplete_in_public_reports() {
    let root = fixture("malformed-stories");
    let direct = check(&root, &fixture_config(), None).unwrap();
    assert_eq!(direct.len(), 2, "{direct:#?}");
    assert_eq!(direct[0].file, "other/components/Missing.tsx");
    assert_eq!(direct[1].file, "web/stories/Broken.stories.tsx");
    assert_eq!(direct[1].target.as_deref(), Some("unanalyzable"));
    assert!(direct[1]
        .message
        .contains("Selected Storybook story could not be analyzed"));
    assert!(direct[1].message.contains("coverage is incomplete"));
    assert_eq!(
        direct,
        crate::codebase::rules::run_check(&root, None, None).unwrap()
    );
    let json = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({ "root": root }).to_string(),
    ))
    .unwrap();
    let report: serde_json::Value =
        crate::test_support::without_hygiene_advisory_json(serde_json::from_str(&json).unwrap());
    assert_eq!(report["rules"], serde_json::to_value(direct).unwrap());
}

#[test]
fn malformed_story_policy_and_visibility_never_create_missing_claims() {
    let root = fixture("malformed-stories");
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
    ] {
        for scope in ["include", "exclude", "project", "allow", "ignore", "advise"] {
            let mut cfg = fixture_config();
            cfg.effort_override = Some(effort);
            match scope {
                "include" => cfg.rules[0].include = vec!["web/components/**".into()],
                "exclude" => cfg.rules[0].exclude = vec!["web/stories/**".into()],
                "project" => {
                    cfg.projects.get_mut("web").unwrap().exclude = vec!["stories/**".into()]
                }
                "allow" => {
                    cfg.rules[0].options["allowFiles"]["stories/Broken.stories.tsx"] =
                        "Separate harness".into()
                }
                policy => cfg.rules[0].options["unanalyzable"] = policy.into(),
            }
            let findings = check(&root, &cfg, None).unwrap();
            let count = if matches!(scope, "ignore" | "advise") {
                1
            } else {
                2
            };
            assert_eq!(findings.len(), count, "{scope}: {findings:#?}");
            assert_eq!(findings[0].file, "other/components/Missing.tsx");
            if count == 2 {
                assert_eq!(findings[1].file, "web/components/Unknown.tsx");
                assert_eq!(findings[1].target.as_deref(), Some("unanalyzable"));
            }
        }
    }
}

#[test]
fn malformed_story_advisories_and_audit_preserve_policy() {
    let root = fixture("malformed-stories");
    for (config, count, advisories, suppressed) in [
        (".no-mistakes.yml", 2, 0, 2),
        ("advise.yml", 1, 1, 2),
        ("ignore.yml", 1, 0, 0),
    ] {
        let report = |audit| {
            let json = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                serde_json::json!({ "root": root, "config": root.join(config), "includeSuppressed": audit }).to_string(),
            )).unwrap();
            crate::test_support::without_hygiene_advisory_json(
                serde_json::from_str::<serde_json::Value>(&json).unwrap(),
            )
        };
        let baseline = report(false);
        let audit = report(true);
        assert_eq!(audit["rules"], baseline["rules"]);
        assert_eq!(audit["advisories"], baseline["advisories"]);
        assert_eq!(
            audit["rules"].as_array().unwrap().len(),
            count,
            "{config}: {audit}"
        );
        assert_eq!(
            audit["advisories"].as_array().unwrap().len(),
            advisories,
            "{config}: {audit}"
        );
        assert_eq!(
            audit["suppressed"].as_array().unwrap().len(),
            suppressed,
            "{config}: {audit}"
        );
    }
}

#[test]
fn hidden_story_errors_keep_component_residuals_and_audit_parity() {
    let root = fixture("malformed-stories");
    for (config, suppressed) in [("suppressed.yml", 2), ("filtered.yml", 0)] {
        let cfg = crate::config::v2::load_v2_config(&root, Some(&root.join(config))).unwrap();
        let direct = check(&root, &cfg, None).unwrap();
        assert_eq!(direct.len(), 2, "{config}: {direct:#?}");
        assert_eq!(direct[1].file, "web/components/Unknown.tsx");
        assert!(direct[1]
            .message
            .contains("unknown because a selected story"));
        for audit in [false, true] {
            let json = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                serde_json::json!({ "root": root, "config": root.join(config), "includeSuppressed": audit }).to_string(),
            )).unwrap();
            let value: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
                serde_json::from_str(&json).unwrap(),
            );
            assert_eq!(value["rules"], serde_json::to_value(&direct).unwrap());
            if audit {
                assert_eq!(value["suppressed"].as_array().unwrap().len(), suppressed);
            }
        }
    }
}

#[test]
fn incomplete_application_does_not_hide_same_project_independent_application() {
    let root = fixture("malformed-stories");
    let mut cfg = fixture_config();
    let mut independent = cfg.rules[0].clone();
    independent.options["stories"] = serde_yaml::to_value(["stories/Card.stories.tsx"]).unwrap();
    cfg.rules.push(independent);
    // Identical project roots do not imply identical configured story universes.
    let findings = check(&root, &cfg, None).unwrap();
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.target.as_deref() == Some("components/Unknown.tsx#Unknown")));
}

#[test]
fn incomplete_coverage_preserves_definite_allow_entry_findings() {
    let root = fixture("malformed-stories");
    let mut cfg = fixture_config();
    cfg.rules[0].options["allowComponents"]["components/Absent.tsx#Absent"] =
        "Separate harness".into();
    let findings = check(&root, &cfg, None).unwrap();
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(findings.iter().any(|finding| finding
        .message
        .contains("does not match a selected component")));
}
