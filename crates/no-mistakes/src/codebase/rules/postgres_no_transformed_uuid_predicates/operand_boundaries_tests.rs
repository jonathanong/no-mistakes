use super::tests::{config, fixture};
use super::*;
fn run(name: &str, effort: crate::config::v2::Effort) -> Vec<RuleFinding> {
    let root = fixture();
    let mut cfg = config("");
    cfg.effort = Some(effort);
    check_with_files(&root, &cfg, &[root.join(name), root.join("schema.json")]).unwrap()
}
#[test]
fn boolean_wrappers_and_case_conditions_preserve_bare_uuid_operands() {
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
    ] {
        let findings = run("operand-boundaries-pass.sql", effort);
        assert!(findings.is_empty(), "{effort:?}: {findings:#?}");
    }
}
#[test]
fn actual_uuid_transformations_survive_predicate_boundaries_and_value_wrappers() {
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
    ] {
        let findings = run("operand-boundaries-positive.sql", effort);
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            (2..=13).collect::<Vec<_>>(),
            "{effort:?}: {findings:#?}"
        );
        assert!(findings
            .iter()
            .all(|finding| finding.target.as_deref() == Some("accounts.id")));
    }
}
#[test]
fn executor_literals_share_the_same_operand_boundaries_at_both_efforts() {
    let max = run("operand-boundaries.ts", crate::config::v2::Effort::Max);
    let low = run("operand-boundaries.ts", crate::config::v2::Effort::Low);
    assert_eq!(low, max);
    assert_eq!(
        max.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        [8, 9, 10, 11]
    );
}
