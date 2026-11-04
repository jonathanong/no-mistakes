#[test]
fn stale_allowlist_entries_point_to_the_fragment_entry() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/config-v2/split-provenance/fixture/rule-diagnostics"),
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let rule = config
        .rules
        .iter()
        .find(|rule| rule.rule == super::super::RULE_ID)
        .unwrap();
    let options = rule.try_rule_options().unwrap();
    let empty = std::collections::BTreeSet::new();
    let mut findings = super::super::scan_application(
        &options,
        &empty,
        &empty,
        &empty,
        &empty,
        &empty,
        ".no-mistakes.yml",
    );
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    super::annotate(&root, &config, rule, &sources, &mut findings);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes/rules/types.yml");
    assert_eq!(findings[0].line, 4);
}
