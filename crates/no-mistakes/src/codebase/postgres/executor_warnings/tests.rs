use super::{configure_executor_warnings, executor_config_warnings};
use crate::codebase::check_facts::{
    CheckFactPlan, PlaywrightFactPlan, collect_check_facts, collect_check_facts_with_playwright,
};
use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::schema::postgres_executors::ALL_RULE_IDS;
use std::path::PathBuf;

#[test]
fn every_executor_rule_warns_from_one_scoped_fact_pass() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/postgres-executor-names");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("override.yml"))).unwrap();
    let original = config.rules[0].clone();
    config.rules = ALL_RULE_IDS
        .iter()
        .map(|id| {
            let mut rule = original.clone();
            rule.rule = (*id).to_string();
            if *id == "postgres-no-test-state-mutation" {
                let options = rule.options.as_mapping_mut().unwrap();
                let include = options.remove(serde_yaml::Value::from("include")).unwrap();
                options.insert("testGlobs".into(), include);
            }
            rule
        })
        .collect();
    let mut plan = CheckFactPlan {
        raw_source: true,
        ..Default::default()
    };
    configure_executor_warnings(&root, &config, &mut plan).unwrap();
    assert_eq!(
        plan.postgres_executor_warning_scans.len(),
        ALL_RULE_IDS.len()
    );
    let selected = root.join("src/sites.ts");
    let outside = root.join("src/annotated.ts");
    for playwright in [None, Some(PlaywrightFactPlan::default())] {
        crate::ast::begin_parse_count(&root);
        let facts = collect_check_facts_with_playwright(
            &root,
            vec![selected.clone(), outside.clone()],
            plan.clone(),
            playwright,
        );
        let warnings = executor_config_warnings(&root, &facts);
        let counts = crate::ast::finish_parse_count(&root);
        assert_eq!(counts.get(&selected), Some(&1), "{counts:#?}");
        assert!(!counts.contains_key(&outside), "{counts:#?}");
        assert_eq!(facts.stats.files_parsed, 1);
        assert_eq!(facts.stats.parse_errors, 0);
        let outside_facts = facts.ts.get(&outside).unwrap();
        assert!(outside_facts.ts.call_sites.is_empty());
        assert!(outside_facts.ts.module_bindings.is_none());
        assert_eq!(warnings.len(), ALL_RULE_IDS.len() * 3, "{warnings:#?}");
        for id in ALL_RULE_IDS {
            let rows = warnings
                .iter()
                .filter(|warning| warning.starts_with(&format!("{id}:")))
                .collect::<Vec<_>>();
            assert_eq!(rows.len(), 3, "{id}: {warnings:#?}");
            for (name, line) in [("query", 2), ("read", 3), ("write", 4)] {
                assert!(
                    rows.iter()
                        .any(|warning| warning.contains(&format!("`{name}`"))
                            && warning.contains(&format!("src/sites.ts:{line}"))
                            && warning.contains("extraExecutorNames")),
                    "{id} {name}: {rows:#?}"
                );
            }
        }
    }
}

#[test]
fn default_empty_and_complete_executor_lists_need_no_warning_facts() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/postgres-executor-names");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    for name in ["default-names", "empty-names", "full-names"] {
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(format!("{name}.yml"))))
                .unwrap();
        let mut plan = CheckFactPlan {
            raw_source: true,
            ..Default::default()
        };
        configure_executor_warnings(&root, &config, &mut plan).unwrap();
        assert!(plan.postgres_executor_warning_scans.is_empty(), "{name}");
        let facts = collect_check_facts(&root, vec![root.join("src/sites.ts")], plan);
        assert!(executor_config_warnings(&root, &facts).is_empty(), "{name}");
        assert_eq!(facts.stats.files_parsed, 0, "{name}");
    }
}
