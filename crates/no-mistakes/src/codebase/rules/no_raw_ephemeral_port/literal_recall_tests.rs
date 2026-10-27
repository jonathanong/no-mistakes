use super::tests::{fixture, run_deferred};

// Output cases moved to cli_rra_snapshot_no_raw_ephemeral_port.

fn locations(findings: Vec<super::RuleFinding>) -> Vec<(String, usize)> {
    findings.into_iter().map(|f| (f.file, f.line)).collect()
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
