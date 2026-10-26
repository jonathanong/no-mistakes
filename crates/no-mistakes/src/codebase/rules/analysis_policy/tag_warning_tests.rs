use super::*;

#[test]
fn ignored_tags_warn_only_for_enabled_low_annotation_applications() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/rule-analysis-policy/fixture");
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(Path::new("annotation-tags.yml"))).unwrap();
    let sources = super::super::source_store_for_files(&[]);
    prepare_rule_analysis(&root, &mut config, &[], &sources).unwrap();
    assert_eq!(config.analysis_warnings, [
        "tagged-low: trustedSqlTags is deprecated and ignored at effort low; remove it: the literal leading /* name */ determines annotation for every tag",
        "empty-low: trustedSqlTags is deprecated and ignored at effort low; remove it: the literal leading /* name */ determines annotation for every tag",
    ]);
    assert!(config.analysis_fallbacks.is_empty());
    // Re-preparation replaces warnings; an explicit max override stays quiet.
    prepare_rule_analysis(&root, &mut config, &[], &sources).unwrap();
    assert_eq!(config.analysis_warnings.len(), 2);
    config.effort_override = Some(Effort::Max);
    prepare_rule_analysis(&root, &mut config, &[], &sources).unwrap();
    assert!(config.analysis_warnings.is_empty());
}
