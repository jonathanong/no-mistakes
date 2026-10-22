use super::*;
use std::path::PathBuf;

pub(super) fn saved_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn visible_files(root: &Path) -> Vec<PathBuf> {
    crate::codebase::ts_source::discover_files(root, &[])
}

#[test]
fn default_plan_skips_only_families_with_proven_absent_inputs() {
    assert!(super::plan::union_allowed_set(&[]).is_none());
    assert!(try_graph_plan(&NoMistakesConfig::default())
        .unwrap()
        .is_none());
    let root = saved_fixture("forbidden-dependencies-basic");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = visible_files(&root);
    let plan = try_graph_plan_for_files(&root, &config, &files)
        .unwrap()
        .unwrap();

    assert!(graph_plan(&config).is_some_and(|full| full.markdown && full.ci));
    assert!(try_graph_plan(&config)
        .unwrap()
        .is_some_and(|full| full.markdown));
    assert!(plan.imports && plan.workspace && plan.package && plan.resources);
    assert!(!plan.markdown && !plan.ci && !plan.workflow_topology);
    assert!(!plan.dotnet && !plan.swift && !plan.terraform);
    assert!(crate::codebase::rules::canonical_graph_requires_full_file_universe(&config));
    assert_eq!(
        check_with_config(&root, &config, None, None).unwrap().len(),
        1
    );
}

#[test]
fn absent_markdown_never_enters_its_graph_collector() {
    let root = saved_fixture("forbidden-dependencies-inventory-absent");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let scans_before = super::inventory_scan_tests::absent_fixture_scans();
    let before =
        crate::codebase::dependencies::graph::markdown_dispatch_probe::absent_fixture_dispatches();
    let findings = check_with_config(&root, &config, None, None).unwrap();
    let after =
        crate::codebase::dependencies::graph::markdown_dispatch_probe::absent_fixture_dispatches();
    assert_eq!(findings.len(), 1);
    assert_eq!(after - before, 0);
    assert_eq!(
        super::inventory_scan_tests::absent_fixture_scans() - scans_before,
        1
    );
    // An explicit MD relationship must enter the same collector even though
    // this saved fixture has no Markdown source.
    let explicit_path = root.join("explicit-md.no-mistakes.yml");
    let explicit = crate::config::v2::load_v2_config(&root, Some(&explicit_path)).unwrap();
    let _ = check_with_config(&root, &explicit, Some(&explicit_path), None).unwrap();
    assert_eq!(
        crate::codebase::dependencies::graph::markdown_dispatch_probe::absent_fixture_dispatches()
            - after,
        1
    );
}

#[test]
fn default_plan_keeps_present_markdown_manifest_workflow_and_terraform_inputs() {
    let root = saved_fixture("forbidden-dependencies-inventory");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = visible_files(&root);
    let plan = try_graph_plan_for_files(&root, &config, &files)
        .unwrap()
        .unwrap();

    assert!(plan.markdown && plan.package && plan.ci && plan.workflow_topology);
    assert!(plan.terraform);
    assert!(!plan.dotnet && !plan.swift);
    let findings = check_with_config(&root, &config, None, None).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("sharp"));
}

#[test]
fn explicit_and_mixed_relationships_keep_requested_families() {
    let root = saved_fixture("forbidden-dependencies-basic");
    let files = visible_files(&root);
    let mixed_path = root.join("mixed-md.no-mistakes.yml");
    let mixed = crate::config::v2::load_v2_config(&root, Some(&mixed_path)).unwrap();
    let mixed_plan = try_graph_plan_for_files(&root, &mixed, &files)
        .unwrap()
        .unwrap();
    // This fixture has no Markdown source; the explicit app still requests it.
    assert!(!files
        .iter()
        .any(|path| path.extension() == Some("md".as_ref())));
    assert!(mixed_plan.markdown && mixed_plan.imports);
    assert!(!mixed_plan.ci && !mixed_plan.terraform);
    assert_eq!(
        check_with_config(&root, &mixed, Some(&mixed_path), None)
            .unwrap()
            .len(),
        1
    );

    let rich_root = saved_fixture("forbidden-dependencies-inventory");
    let explicit_path = rich_root.join("explicit.no-mistakes.yml");
    let explicit = crate::config::v2::load_v2_config(&rich_root, Some(&explicit_path)).unwrap();
    let explicit_plan = try_graph_plan_for_files(&rich_root, &explicit, &visible_files(&rich_root))
        .unwrap()
        .unwrap();
    assert!(explicit_plan.markdown);
    assert!(!explicit_plan.package && !explicit_plan.ci);
    let findings = check_with_config(&rich_root, &explicit, Some(&explicit_path), None).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("src/forbidden.mts"));
}

#[test]
fn configured_external_metadata_and_other_graph_consumers_remain_demanded() {
    let root = saved_fixture("forbidden-dependencies-basic");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config
        .tests
        .dotnet
        .solutions
        .push("external/solution.sln".into());
    config
        .tests
        .swift
        .packages
        .push("external/Package.swift".into());
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: crate::codebase::rules::FORBIDDEN_CALLS.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        ..Default::default()
    });
    let files = visible_files(&root);
    let plan = crate::codebase::rules::try_canonical_graph_plan_for_files(&root, &config, &files)
        .unwrap()
        .unwrap();
    assert!(plan.dotnet && plan.swift && plan.calls);
    assert!(!plan.markdown && !plan.ci);
}

#[test]
fn standalone_in_memory_rules_keep_discovered_graph_settings() {
    let root = saved_fixture("forbidden-dependencies-inventory");
    let discovered = crate::config::v2::load_v2_config(&root, None).unwrap();
    let rules_only = NoMistakesConfig {
        rules: discovered.rules.clone(),
        ..Default::default()
    };
    let opts = rules_only
        .rule_applications(RULE_ID)
        .iter()
        .map(|rule| rule.try_rule_options())
        .collect::<Result<Vec<Options>>>()
        .unwrap();
    let plan = super::plan::graph_plan_for_options_with_graph_config(
        &root,
        &rules_only,
        &visible_files(&root),
        &opts,
        false,
    );
    assert!(plan.markdown && plan.package && plan.ci && plan.workflow_topology);
    assert!(plan.terraform && plan.dotnet && plan.swift);
    let findings = check(&root, &rules_only, None).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("sharp"));
}
