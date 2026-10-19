use super::{check, PathBuf};

#[test]
fn shared_const_arms_report_at_each_call() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants/conditional-clause-shared-const");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let expected_offsets = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// findings: offset").then_some(index + 1))
        .collect::<Vec<_>>();
    let expected_unanalyzable = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.contains("// findings: unanalyzable")
                .then_some(index + 1)
        })
        .collect::<Vec<_>>();
    let report = check("postgres-no-offset", "conditional-clause-shared-const");
    let findings = report["rules"].as_array().unwrap();
    let offsets = findings
        .iter()
        .filter(|finding| finding["target"].as_str().unwrap().starts_with("offset"))
        .enumerate()
        .map(|(index, finding)| {
            let target = if index == 0 {
                "offset".to_string()
            } else {
                format!("offset#{}", index + 1)
            };
            assert_eq!(finding["target"], target, "{report}");
            finding["line"].as_u64().unwrap() as usize
        })
        .collect::<Vec<_>>();
    let unanalyzable = findings
        .iter()
        .filter(|finding| finding["target"] == "unanalyzable")
        .map(|finding| finding["line"].as_u64().unwrap() as usize)
        .collect::<Vec<_>>();
    assert_eq!(offsets, expected_offsets, "{report}");
    assert_eq!(unanalyzable, expected_unanalyzable, "{report}");
    assert_eq!(
        findings.len(),
        expected_offsets.len() + expected_unanalyzable.len(),
        "{report}"
    );
}
