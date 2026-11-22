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

#[test]
fn replacement_and_extension_conflict_even_when_empty_for_every_executor_rule() {
    let config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/config/postgres-executor-names/conflict.yml"
    )))
    .unwrap();
    for id in ALL_RULE_IDS {
        let mut rule = config.rules[0].clone();
        rule.rule = (*id).into();
        let error = rule.try_rule_options::<Value>().unwrap_err().to_string();
        assert!(error.contains("cannot both be set"), "{error}");
    }
}

#[test]
fn lock_inventory_requires_scope_but_allows_helper_only_configuration() {
    let bare: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/config/postgres-executor-names/lock-inventory-bare.yml"
    )))
    .unwrap();
    let error = bare.rules[0]
        .try_rule_options::<Value>()
        .unwrap_err()
        .to_string();
    assert!(error.contains("postgres-lock-inventory"), "{error}");
    assert!(error.contains("executorNames: []"), "{error}");

    let helper_only: crate::config::v2::NoMistakesConfig =
        serde_yaml::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/config/postgres-executor-names/lock-inventory-helper-only.yml"
        )))
        .unwrap();
    assert!(helper_only.rules[0].try_rule_options::<Value>().is_ok());
}

#[test]
fn lock_inventory_inherits_executor_scope_and_keeps_reporting_uncertainty() {
    use crate::config::v2::{load_v2_config, Unanalyzable};

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/postgres-executor-names");
    let config = load_v2_config(&root, Some(&root.join("lock-inventory-inherited.yml"))).unwrap();
    let [app, offset, schema] = config.rules.as_slice() else {
        panic!("expected three rule applications");
    };
    assert_eq!(app.options["importSpecifier"], "@example/db");
    assert_eq!(
        schema.options["executorNames"].as_sequence().unwrap().len(),
        0
    );
    assert_eq!(config.unanalyzable, Some(Unanalyzable::Ignore));
    for rule in [app, schema] {
        assert!(rule.options.get("unanalyzableSql").is_none());
        assert_eq!(
            config.requested_rule_analysis(rule).unwrap().unanalyzable,
            Unanalyzable::Report
        );
    }
    assert_eq!(offset.options["unanalyzableSql"], "advise");
    assert_eq!(
        config.requested_rule_analysis(offset).unwrap().unanalyzable,
        Unanalyzable::Advise
    );
}

#[test]
fn executor_validation_covers_every_prepared_public_rule() {
    let ids = |values: &[&str]| {
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_eq!(
        ids(ALL_RULE_IDS),
        ids(crate::codebase::postgres::PREPARED_EMBEDDED_SQL_RULE_IDS)
    );
}
