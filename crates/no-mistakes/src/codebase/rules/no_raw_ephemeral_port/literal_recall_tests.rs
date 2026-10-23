use super::tests::{fixture, run, run_deferred};

fn locations(findings: Vec<super::RuleFinding>) -> Vec<(String, usize)> {
    findings.into_iter().map(|f| (f.file, f.line)).collect()
}

#[test]
fn literal_recall_preserves_controls_and_exact_lines() {
    let mut expected = Vec::new();
    expected.extend((2..=7).map(|line| ("server.py".to_string(), line)));
    expected.extend((2..=11).map(|line| ("server.ts".to_string(), line)));
    assert_eq!(locations(run(&fixture("literal-recall"), "{}")), expected);
}

#[test]
fn literal_recall_respects_include_and_allow() {
    let expected: Vec<_> = (2..=7)
        .map(|line| ("server.py".to_string(), line))
        .collect();
    assert_eq!(
        locations(run(&fixture("literal-recall"), "include: ['**/*.py']")),
        expected
    );
    assert_eq!(
        locations(run(&fixture("literal-recall"), "allow: ['*.ts']")),
        expected
    );
}

#[test]
fn literal_recall_deferred_suppression_retains_locations() {
    // Deferred callers apply the same suppression after aggregating rule reports.
    let findings = locations(run_deferred(&fixture("literal-recall"), "{}", true));
    let mut expected = vec![
        ("disabled.py".to_string(), 2),
        ("disabled.py".to_string(), 3),
        ("disabled.ts".to_string(), 2),
        ("disabled.ts".to_string(), 3),
    ];
    expected.extend([2, 3, 4, 5, 6, 7, 16, 18].map(|line| ("server.py".to_string(), line)));
    expected.extend(
        (2..=11)
            .chain([20, 22])
            .map(|line| ("server.ts".to_string(), line)),
    );
    assert_eq!(findings, expected);
}
