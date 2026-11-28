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
