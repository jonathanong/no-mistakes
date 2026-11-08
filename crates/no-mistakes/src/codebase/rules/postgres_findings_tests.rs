use super::{site_suffixes, RuleFinding};

#[test]
fn suffixes_number_only_fully_identical_public_sites_and_ignore_input_order() {
    let finding = RuleFinding {
        causes: Vec::new(),
        rule: "postgres-no-transformed-uuid-predicates".into(),
        file: "shared.ts".into(),
        line: 2,
        message: "shared.ts:2: transformed UUID".into(),
        import: None,
        target: Some("accounts.id".into()),
    };
    let mut different = finding.clone();
    different.target = Some("accounts.token".into());
    let sites = [
        (&finding, "first.ts", (1, 30)),
        (&finding, "first.ts", (1, 30)),
        (&finding, "second.ts", (1, 30)),
        (&finding, "first.ts", (1, 50)),
        (&different, "first.ts", (1, 30)),
    ];
    let expected: [Option<String>; 5] = [
        Some(" (expression at SQL 1:30 from first.ts; site 1)".into()),
        Some(" (expression at SQL 1:30 from first.ts; site 2)".into()),
        Some(" (expression at SQL 1:30 from second.ts)".into()),
        Some(" (expression at SQL 1:50 from first.ts)".into()),
        None,
    ];
    assert_eq!(site_suffixes(sites.iter().copied(), "expression"), expected);
    let mut forward = site_suffixes(sites.iter().copied(), "expression");
    let mut reverse = site_suffixes(sites.iter().rev().copied(), "expression");
    forward.sort();
    reverse.sort();
    assert_eq!(forward, reverse);
}
