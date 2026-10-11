use super::{check, PathBuf};

#[test]
fn copied_immutable_guards_exclude_impossible_offsets_and_preserve_reachable_ones() {
    let safe = check("postgres-no-offset", "guard-alias-safe");
    assert!(safe["rules"].as_array().unwrap().is_empty(), "{safe}");

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants/guard-alias-unsafe");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.contains("// findings: offset")
                .then_some(index as u64 + 1)
        })
        .collect::<Vec<_>>();
    let unsafe_report = check("postgres-no-offset", "guard-alias-unsafe");
    let actual = unsafe_report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert!(
                finding["target"].as_str().unwrap().starts_with("offset"),
                "{unsafe_report}"
            );
            finding["line"].as_u64().unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{unsafe_report}");
}
