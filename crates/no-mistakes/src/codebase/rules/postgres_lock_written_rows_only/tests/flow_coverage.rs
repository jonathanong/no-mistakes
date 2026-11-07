use super::*;

fn findings(case: &str) -> Vec<(usize, String)> {
    let root = fixture(case);
    let path = root
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs/checks-fail.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&path)).unwrap();
    let mut findings = check(&root, &config)
        .unwrap()
        .into_iter()
        .map(|finding| (finding.line, finding.target.unwrap()))
        .collect::<Vec<_>>();
    findings.sort();
    findings
}

#[test]
fn nested_closed_exits_and_constant_branches_preserve_only_live_writes() {
    assert_eq!(
        findings("flow-coverage"),
        [
            (17, "unanalyzable".to_owned()),
            (29, "unanalyzable".to_owned()),
            (59, CHECKS[0].to_owned()),
            (66, CHECKS[0].to_owned()),
        ]
    );
}

#[test]
fn typed_executors_without_factories_do_not_claim_transaction_membership() {
    assert_eq!(
        findings("flow-coverage-no-factory"),
        [3, 8, 19].map(|line| (line, "unanalyzable".to_owned()))
    );
}

#[test]
fn only_definite_transaction_closes_discharge_an_early_exit() {
    let root = fixture("conditional-close-coverage");
    let path = root
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs/conditional-close-coverage.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&path)).unwrap();
    let mut findings = check(&root, &config)
        .unwrap()
        .into_iter()
        .map(|finding| (finding.line, finding.target.unwrap()))
        .collect::<Vec<_>>();
    findings.sort();
    let mut expected = [4, 24, 33, 57, 67, 77, 87, 97, 107, 117, 127]
        .map(|line| (line, "unanalyzable".to_owned()))
        .to_vec();
    expected.extend([140, 152, 163, 185, 192, 200].map(|line| (line, CHECKS[0].to_owned())));
    expected.extend([214, 224, 234, 238].map(|line| (line, "unanalyzable".to_owned())));
    expected.extend([245, 256, 262].map(|line| (line, CHECKS[0].to_owned())));
    expected.extend([276, 286].map(|line| (line, "unanalyzable".to_owned())));
    assert_eq!(findings, expected);
}
