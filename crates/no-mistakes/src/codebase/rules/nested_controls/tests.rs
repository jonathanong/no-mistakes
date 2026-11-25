use super::*;
use crate::codebase::check_facts::CheckFileFacts;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/react-no-nested-buttons-and-links/fixture"),
    )
}

#[test]
fn native_control_nesting_merges_disjoint_fact_demands() {
    let root = fixture_root();
    let actions = root.join("actions.tsx");
    let variants = root.join("variants.jsx");
    let files = vec![actions.clone(), variants.clone()];
    let sources = super::super::source_store_for_files(&files);
    let mut plan = CheckFactPlan {
        nested_control_paths: Arc::new(BTreeSet::from([actions.clone()])),
        ..Default::default()
    };
    plan.include(CheckFactPlan {
        nested_control_paths: Arc::new(BTreeSet::from([variants.clone()])),
        ..Default::default()
    });

    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files,
            Vec::new(),
            plan,
            None,
            Arc::clone(&sources),
        );
    let actions_diag = check_file(
        &root,
        &actions,
        Some(&facts),
        &sources,
        None,
        Unanalyzable::Report,
    )
    .unwrap();
    assert_eq!(actions_diag.findings.len(), 1);
    let variants_report = check_file(
        &root,
        &variants,
        Some(&facts),
        &sources,
        None,
        Unanalyzable::Report,
    )
    .unwrap();
    let definite = variants_report
        .findings
        .iter()
        .filter(|finding| finding.target.is_none())
        .count();
    let unanalyzable = variants_report
        .findings
        .iter()
        .filter(|finding| finding.target.as_deref() == Some("unanalyzable"))
        .count();
    assert_eq!(definite, 7);
    assert_eq!(unanalyzable, 15);

    let variants_advise = check_file(
        &root,
        &variants,
        Some(&facts),
        &sources,
        None,
        Unanalyzable::Advise,
    )
    .unwrap();
    assert_eq!(variants_advise.findings.len(), 7);
    assert_eq!(variants_advise.advisories.len(), 15);

    let variants_ignore = check_file(
        &root,
        &variants,
        Some(&facts),
        &sources,
        None,
        Unanalyzable::Ignore,
    )
    .unwrap();
    assert_eq!(variants_ignore.findings.len(), 7);
    assert!(variants_ignore.advisories.is_empty());
}

#[test]
fn native_control_nesting_defaults_unanalyzable_to_advise_and_inherits_top_level() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/react-no-nested-buttons-and-links");
    let root = fixture_root();
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&fixture.join("configs/check.yml"))).unwrap();
    assert_eq!(
        resolve_unanalyzable(&config, &config.rules[0]).unwrap(),
        Unanalyzable::Advise,
    );
    config.unanalyzable = Some(Unanalyzable::Report);
    assert_eq!(
        resolve_unanalyzable(&config, &config.rules[0]).unwrap(),
        Unanalyzable::Report,
    );
}

#[test]
fn native_control_nesting_distinguishes_missing_file_and_missing_rule_facts() {
    let root = fixture_root();
    let path = root.join("actions.tsx");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let mut facts = CheckFactMap::default();

    let absent_file = check_file(
        &root,
        &path,
        Some(&facts),
        &sources,
        None,
        Unanalyzable::Report,
    )
    .err()
    .unwrap();
    assert_eq!(
        absent_file.to_string(),
        format!("{RULE_ID}: missing prepared JSX facts for actions.tsx"),
    );

    facts
        .ts
        .insert(path.clone(), Arc::new(CheckFileFacts::default()));
    let absent_rule = check_file(
        &root,
        &path,
        Some(&facts),
        &sources,
        None,
        Unanalyzable::Report,
    )
    .err()
    .unwrap();
    assert_eq!(
        absent_rule.to_string(),
        format!("{RULE_ID}: missing prepared nesting facts for actions.tsx"),
    );
}

#[test]
fn native_control_nesting_handles_clean_bare_children_and_nested_spread() {
    let fixture_dir = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/react-no-nested-buttons-and-links"),
    );
    let clean = fixture_dir.join("fixture/clean.jsx");
    let edge = fixture_dir.join("edge-cases/children-and-spread.jsx");
    let files = vec![clean.clone(), edge.clone()];
    let sources = super::super::source_store_for_files(&files);
    let plan = CheckFactPlan {
        nested_control_paths: Arc::new(BTreeSet::from([clean.clone(), edge.clone()])),
        ..Default::default()
    };
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &fixture_dir,
            files,
            Vec::new(),
            plan,
            None,
            Arc::clone(&sources),
        );

    let clean_diag = check_file(
        &fixture_dir,
        &clean,
        Some(&facts),
        &sources,
        None,
        Unanalyzable::Report,
    )
    .unwrap();
    assert!(clean_diag.findings.is_empty(), "{:?}", clean_diag.findings);
    assert!(
        clean_diag.advisories.is_empty(),
        "{:?}",
        clean_diag.advisories
    );

    let edge_diag = check_file(
        &fixture_dir,
        &edge,
        Some(&facts),
        &sources,
        None,
        Unanalyzable::Report,
    )
    .unwrap();
    let lines: Vec<usize> = edge_diag.findings.iter().map(|f| f.line).collect();
    assert_eq!(lines, vec![4, 7, 11], "{:?}", edge_diag.findings);
    assert!(
        edge_diag
            .findings
            .iter()
            .all(|finding| finding.target.is_none()),
        "{:?}",
        edge_diag.findings
    );
    assert!(
        edge_diag.advisories.is_empty(),
        "{:?}",
        edge_diag.advisories
    );
}

#[test]
fn native_control_nesting_propagates_parse_errors_invalid_options_and_custom_messages() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/react-no-nested-buttons-and-links");
    let parse_root = crate::codebase::ts_resolver::normalize_path(&fixture.join("parse-error"));
    let parse_cfg =
        crate::config::v2::load_v2_config(&parse_root, Some(&fixture.join("configs/parse-error.yml")))
            .unwrap();
    let disabled = parse_root.join("a-disabled.tsx");
    let broken = parse_root.join("broken.tsx");
    assert!(check_with_files(&parse_root, &parse_cfg, std::slice::from_ref(&disabled))
        .unwrap()
        .is_empty());
    let parse_err = check_with_files(&parse_root, &parse_cfg, &[disabled, broken])
        .err()
        .unwrap();
    assert!(parse_err.to_string().contains("Repair the JSX"), "{parse_err}");

    let root = fixture_root();
    let actions = root.join("actions.tsx");
    let files = vec![actions.clone()];
    let sources = super::super::source_store_for_files(&files);
    let mut cfg =
        crate::config::v2::load_v2_config(&root, Some(&fixture.join("configs/check.yml"))).unwrap();
    assert!(check_prepared(&root, &cfg, &files, None, &sources, None).is_err());

    cfg.rules[0].message = Some("custom nesting hint".into());
    let custom = check_with_files(&root, &cfg, &files).unwrap();
    assert_eq!(custom.len(), 1);
    assert_eq!(custom[0].message, "custom nesting hint");

    cfg.rules[0].options = serde_yaml::from_str("unanalyzable: bogus").unwrap();
    assert!(diagnostics_prepared(&root, &cfg, &files, None, &sources, None, false).is_err());
}
