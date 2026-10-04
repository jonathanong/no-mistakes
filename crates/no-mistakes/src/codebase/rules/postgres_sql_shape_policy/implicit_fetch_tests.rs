use super::shape_tests::{config_yaml, fixture};
use super::*;

#[test]
fn implicit_fetch_locations_follow_each_table_body_and_set_arm() {
    let root = fixture("review-followups");
    let file = root.join("sql/implicit-fetch-origins.sql");
    let config = config_yaml(
        r#"sqlInclude: ['sql/**/*.sql']
bannedShapes: [literal-limit]
shapeOptions:
  literalLimit:
    allowedValues: []
"#,
    );
    let mut findings = check_with_files(&root, &config, std::slice::from_ref(&file)).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [4, 10, 14, 18, 23, 26, 31, 33, 36, 39],
        "{findings:#?}"
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [4, 14, 23, 26, 31, 33, 36],
        "{findings:#?}"
    );
}
