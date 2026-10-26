use super::*;

#[test]
fn malformed_explicit_component_is_reported_by_public_aggregate_rule() {
    let root = fixture("malformed-selected");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "components/Broken.tsx");
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}

fn fixture_config() -> NoMistakesConfig {
    crate::config::v2::load_v2_config(&fixture("malformed-selected"), None).unwrap()
}

fn report(name: &str, audit: bool) -> serde_json::Value {
    let root = fixture("malformed-selected");
    let result = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({
            "root": root, "config": root.join(name), "includeSuppressed": audit,
        })
        .to_string(),
    ))
    .unwrap();
    crate::test_support::without_hygiene_advisory_json(serde_json::from_str(&result).unwrap())
}

#[test]
fn malformed_selected_direct_and_node_reports_match_aggregate() {
    let root = fixture("malformed-selected");
    let direct = check(&root, &fixture_config(), None).unwrap();
    assert_eq!(direct.len(), 1, "{direct:#?}");
    assert_eq!(direct[0].line, 1);
    assert!(direct[0].message.contains("coverage is incomplete"));
    // The saved malformed source must retain the parser cause in every public path.
    assert!(
        direct[0].message.contains("Expected `:` but found `<`"),
        "{}",
        direct[0].message
    );
    assert!(!direct[0].message.contains("no reachable story"));
    let aggregate = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(direct, aggregate);
    assert_eq!(
        report(".no-mistakes.yml", false)["rules"],
        serde_json::to_value(direct).unwrap()
    );
}

#[test]
fn malformed_selected_shared_policy_routes_report_advise_and_ignore() {
    for (config, rules, advisories) in [
        (".no-mistakes.yml", 1, 0),
        ("advise.yml", 0, 1),
        ("ignore.yml", 0, 0),
        ("override.yml", 1, 0),
        ("mixed-advise.yml", 1, 1),
    ] {
        let value = report(config, false);
        assert_eq!(
            value["rules"].as_array().unwrap().len(),
            rules,
            "{config}: {value}"
        );
        assert_eq!(
            value["advisories"].as_array().unwrap().len(),
            advisories,
            "{config}: {value}"
        );
        if advisories > 0 {
            assert_eq!(value["advisories"][0]["target"], "unanalyzable");
            assert_eq!(value["advisories"][0]["file"], "components/Broken.tsx");
        }
        if config == "mixed-advise.yml" {
            assert_eq!(value["rules"][0]["target"], "outside/Valid.tsx#Valid");
        }
    }
}

#[test]
fn malformed_selected_audit_records_only_real_suppressions() {
    for config in [".no-mistakes.yml", "advise.yml", "ignore.yml"] {
        let baseline = report(config, false);
        let audit = report(config, true);
        assert_eq!(audit["rules"], baseline["rules"]);
        assert_eq!(audit["advisories"], baseline["advisories"]);
        let suppressed = audit["suppressed"].as_array().unwrap();
        if config == "ignore.yml" {
            assert!(suppressed.is_empty(), "{audit}");
        } else {
            assert_eq!(suppressed.len(), 2, "{audit}");
            assert_eq!(
                suppressed
                    .iter()
                    .map(|finding| finding["file"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["components/FileDisabled.tsx", "components/LineDisabled.tsx"]
            );
            assert!(suppressed.iter().all(|finding| finding["line"] == 1
                && finding["reason"]
                    .as_str()
                    .unwrap()
                    .contains("coverage is incomplete")));
        }
    }
}

#[test]
fn malformed_selected_scope_filters_remain_authoritative() {
    let root = fixture("malformed-selected");
    for scope in [
        "rule-include",
        "rule-exclude",
        "project-exclude",
        "allow-file",
    ] {
        let mut cfg = fixture_config();
        match scope {
            "rule-include" => cfg.rules[0].include = vec!["outside/**".into()],
            "rule-exclude" => cfg.rules[0].exclude.push("components/Broken.tsx".into()),
            "project-exclude" => cfg
                .projects
                .get_mut("web")
                .unwrap()
                .exclude
                .push("components/Broken.tsx".into()),
            _ => {
                cfg.rules[0].options["allowFiles"]["components/Broken.tsx"] =
                    "separate coverage".into()
            }
        }
        let findings = check(&root, &cfg, None).unwrap();
        assert!(findings.is_empty(), "{scope}: {findings:#?}");
    }
}

#[test]
fn malformed_inputs_do_not_invent_exports_under_inferred_selection() {
    let root = fixture("malformed-selected");
    for options in [
        "",
        "includeAllReactNamedExports: true",
        "includeAllReactDefaultExports: true",
    ] {
        let findings = check(&root, &config(options), None).unwrap();
        assert!(
            findings
                .iter()
                .all(|finding| finding.target.as_deref() != Some("unanalyzable")),
            "{options}: {findings:#?}"
        );
    }
}

#[test]
fn malformed_selected_jsx_is_reported_and_next_line_does_not_suppress_line_one() {
    let root = fixture("malformed-selected");
    for file in ["components/Broken.jsx", "outside/NextLine.tsx"] {
        let findings = check(&root, &config(&format!("include: ['{file}']")), None).unwrap();
        assert_eq!(findings.len(), 1, "{file}: {findings:#?}");
        assert_eq!(findings[0].file, file);
        assert_eq!(findings[0].line, 1);
        assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
    }
}

#[test]
fn malformed_selected_plain_ts_is_not_a_react_input_and_props_need_no_guessed_exports() {
    let root = fixture("malformed-selected");
    let findings = check(&root, &config("include: ['components/plain.ts']"), None).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
    let mut cfg = fixture_config();
    cfg.rules[0].options["requiredProps"] = serde_yaml::to_value(["data-testid"]).unwrap();
    let findings = check(&root, &cfg, None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}
