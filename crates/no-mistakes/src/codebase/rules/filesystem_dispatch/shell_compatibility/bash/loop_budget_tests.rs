use super::analyze;

#[test]
fn loops_preserve_guard_and_helper_exit_boundaries() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/shell-compatibility/loop-exits/exit-paths.sh"
    ));
    let result = analyze(source);
    assert_eq!(
        result.findings,
        [
            ("remaining", 45),
            ("declared", 54),
            ("outside", 59),
            ("opaque", 65)
        ]
    );
    let uncertain: Vec<_> = result.unanalyzable.iter().map(|(line, _)| *line).collect();
    assert_eq!(uncertain, [9, 12, 34]);
}

#[test]
fn independent_flat_loops_do_not_share_the_replay_budget() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/shell-compatibility/flat-loop-budget/flat.sh"
    ));
    let result = analyze(source);
    assert!(result.unanalyzable.is_empty(), "{:?}", result.unanalyzable);
    let expected = (0..80)
        .map(|index| ("items", 6 + 6 * index))
        .collect::<Vec<_>>();
    assert_eq!(result.findings, expected);
}

#[test]
fn nest_exhaustion_abstains_on_a_refilled_array() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/shell-compatibility/exhaustion-refill/refill.sh"
    ));
    let result = analyze(source);
    assert!(
        !result.findings.iter().any(|(name, _)| *name == "safe"),
        "{:?}",
        result.findings
    );
    assert_eq!(result.findings, [("carry", 266)]);
    let safe_uncertain = result.unanalyzable.iter().any(|(line, detail)| {
        *line == 265 && detail.contains("array `safe`") && detail.contains("loop widening")
    });
    assert!(safe_uncertain, "{:?}", result.unanalyzable);
    let exhausted = result.unanalyzable.iter().any(|(line, detail)| {
        *line == 264 && detail.starts_with("Bash loop nest replay budget exhausted")
    });
    assert!(exhausted, "{:?}", result.unanalyzable);
}

#[test]
fn nest_exhaustion_keeps_an_uncertain_array_uncertain() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/shell-compatibility/exhaustion-refill/uncertain-unset.sh"
    ));
    let result = analyze(source);
    assert!(
        !result.findings.iter().any(|(name, _)| *name == "a"),
        "{:?}",
        result.findings
    );
}

#[test]
fn infinite_loop_exhaustion_keeps_the_break_nounset_state() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/shell-compatibility/exhaustion-refill/infinite.sh"
    ));
    let result = analyze(source);
    assert!(
        result
            .unanalyzable
            .iter()
            .any(|(_, detail)| detail.starts_with("Bash loop nest replay budget exhausted")),
        "{:?}",
        result.unanalyzable
    );
    assert!(
        !result.findings.iter().any(|(name, _)| *name == "items"),
        "{:?}",
        result.findings
    );
}
