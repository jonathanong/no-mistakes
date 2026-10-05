use super::*;

#[test]
fn every_executor_rule_requires_selection_and_accepts_explicit_opt_out() {
    for id in RULE_IDS {
        for options in [
            "{}",
            "null",
            "importSpecifier: null",
            "importSpecifier: ''",
            "executorNames: null",
        ] {
            let rule = RuleDef {
                rule: (*id).into(),
                options: serde_yaml::from_str(options).unwrap(),
                ..Default::default()
            };
            let error = rule.try_rule_options::<Value>().unwrap_err().to_string();
            assert!(error.contains(id), "{error}");
            assert!(error.contains("executorNames: []"), "{error}");
        }
        for options in [
            "executorNames: []",
            "executorNames: [run]",
            "importSpecifier: '@example/db'",
        ] {
            let rule = RuleDef {
                rule: (*id).into(),
                options: serde_yaml::from_str(options).unwrap(),
                ..Default::default()
            };
            assert!(rule.try_rule_options::<Value>().is_ok());
        }
    }
}

#[test]
fn unrelated_rules_and_malformed_shapes_keep_their_existing_validation() {
    for (id, options) in [
        ("postgres-table-shape", "null"),
        ("postgres-lock-ordering", "importSpecifier: false"),
        ("postgres-lock-ordering", "executorNames: run"),
    ] {
        let rule = RuleDef {
            rule: id.into(),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        };
        assert!(validate(&rule).is_ok());
    }
}
