use super::tests::{config, fixture};
use super::*;
use crate::config::v2::Effort;

fn run(name: &str, effort: Effort) -> Vec<RuleFinding> {
    let root = fixture();
    let mut cfg = config("");
    cfg.effort = Some(effort);
    check_with_files(&root, &cfg, &[root.join(name), root.join("schema.json")]).unwrap()
}

#[test]
fn aggregates_and_parameter_side_functions_preserve_bare_uuid_operands() {
    for effort in [Effort::Low, Effort::Max] {
        let findings = run("aggregate-operands-pass.sql", effort);
        assert!(findings.is_empty(), "{effort:?}: {findings:#?}");
    }
}

#[test]
fn filter_predicates_and_both_uuid_comparison_operands_remain_checked() {
    let max = run("aggregate-operands-positive.sql", Effort::Max);
    assert_eq!(run("aggregate-operands-positive.sql", Effort::Low), max);
    assert_eq!(
        max.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        (2..=11).chain(14..=17).collect::<Vec<_>>(),
        "{max:#?}"
    );
    assert!(max.iter().all(|finding| {
        finding.target.as_deref()
            == Some(if (9..=11).contains(&finding.line) {
                "accounts.token"
            } else {
                "accounts.id"
            })
    }));
}

#[test]
fn executor_literals_share_aggregate_and_operand_semantics_at_both_efforts() {
    let max = run("aggregate-operands.ts", Effort::Max);
    assert_eq!(run("aggregate-operands.ts", Effort::Low), max);
    assert_eq!(
        max.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        (10..=15).collect::<Vec<_>>(),
        "{max:#?}"
    );
    assert!(max.iter().all(|finding| {
        finding.target.as_deref()
            == Some(if finding.line >= 14 {
                "accounts.token"
            } else {
                "accounts.id"
            })
    }));
}
