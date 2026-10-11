use super::{check, PathBuf};

#[test]
fn branch_assigned_boolean_guards_exclude_impossible_offset_versions() {
    let report = check("postgres-no-offset", "branch-correlation-safe");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
}

#[test]
fn correlated_else_and_inverse_guards_keep_reachable_offset_versions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants/branch-correlation-unsafe");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// findings: offset").then_some(index + 1))
        .collect::<Vec<_>>();
    let report = check("postgres-no-offset", "branch-correlation-unsafe");
    let findings = report["rules"].as_array().unwrap();
    let actual = findings
        .iter()
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
    assert_eq!(actual, expected, "{report}");
}
