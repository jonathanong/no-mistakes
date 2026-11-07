use super::*;

#[test]
fn unused_identifier_sources_report_the_authored_config_entry() {
    let root = fixture();
    let facts = CheckFactMap::default();
    let sources = super::super::super::source_store_for_files(&[]);
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        Default::default(),
        Default::default(),
    );
    for (shape, selector) in [
        ("unused", "@example/catalog#getRelationTableNameOrThrow"),
        ("unused-name", "getRelationTableNameOrThrow"),
    ] {
        let config_file = format!(".no-mistakes-{shape}.yml");
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(&config_file))).unwrap();
        let findings = check(
            &root,
            &config,
            &config.rules[0],
            &graph,
            &facts,
            Some(&sources),
        )
        .unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "unused-allow-entry");
        assert_eq!(findings[0].file, config_file);
        assert_eq!(findings[0].line, 10);
        assert_eq!(findings[0].target.as_deref(), Some(selector));
    }
}

#[test]
fn an_incomplete_source_scope_does_not_claim_a_selector_is_unused() {
    let root = fixture();
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes-unused.yml")))
            .unwrap();
    let sources =
        super::super::super::source_store_for_files(&[root.join("apps/worker/unused.ts")]);
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        Default::default(),
        Default::default(),
    );
    assert!(check(
        &root,
        &config,
        &config.rules[0],
        &graph,
        &CheckFactMap::default(),
        Some(&sources)
    )
    .unwrap()
    .is_empty());
    for failure in ["parse", "operation", "flow"] {
        let mut ts = crate::codebase::ts_source::facts::TsFileFacts::default();
        match failure {
            "parse" => ts.parse_error = Some("incomplete".into()),
            "operation" => ts.operational_error = Some("incomplete".into()),
            _ => ts.value_flow.incomplete_reason = Some("incomplete".into()),
        }
        let mut facts = CheckFactMap::default();
        facts.ts.insert(
            root.join("apps/worker/unused.ts"),
            crate::codebase::check_facts::CheckFileFacts {
                ts: std::sync::Arc::new(ts),
                ..Default::default()
            }
            .into(),
        );
        assert!(
            check(
                &root,
                &config,
                &config.rules[0],
                &graph,
                &facts,
                Some(&sources)
            )
            .unwrap()
            .is_empty(),
            "{failure}"
        );
    }
}

#[test]
fn a_missing_source_inventory_does_not_claim_selectors_are_unused() {
    let root = fixture();
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes-unused.yml")))
            .unwrap();
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        Default::default(),
        Default::default(),
    );
    assert!(check(
        &root,
        &config,
        &config.rules[0],
        &graph,
        &CheckFactMap::default(),
        None,
    )
    .unwrap()
    .is_empty());
}

fn fixture() -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-workspace-producer"),
    )
}
