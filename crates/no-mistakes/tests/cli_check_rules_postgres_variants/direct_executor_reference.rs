use super::*;

#[test]
fn direct_executor_reference_offset_is_fully_judged() {
    // Forms named by `postgres-no-executor-adapter`: `read(sql)`, a const alias of that
    // import, `read.query(sql)`, `q.query(sql)` when `const q = read`, an identifier
    // conditional callee, `tx.query(sql)` on a typed receiver, `(flag ? tx : u).query(sql)`
    // when both receivers have that type, and a scoped executor call. An untyped
    // `client.query` is not one of them.
    // `run(read, account)` is intentionally absent. That form is not judged; see
    // `passed-executor-value`. An empty report, or `unanalyzable`, means a named form is not judged.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants/direct-executor-reference");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let mut expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.split_once("// findings: ")
                .map(|(_, target)| (index + 1, target.trim().to_string()))
        })
        .collect::<Vec<_>>();
    expected.sort();
    assert!(!expected.is_empty(), "fixture must expect a judged finding");
    let report = check("postgres-no-offset", "direct-executor-reference");
    let mut actual = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap() as usize,
                finding["target"]
                    .as_str()
                    .unwrap()
                    .split('#')
                    .next()
                    .unwrap()
                    .to_string(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort();
    assert_eq!(actual, expected, "{report}");
    assert!(
        actual.iter().all(|(_, target)| target == "offset"),
        "{report}"
    );
}
