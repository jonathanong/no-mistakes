#[test]
fn cov_p2_callback_contracts_keep_exact_companion_executor_verdict_spans() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("cov-p2-callbacks");
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let report: serde_json::Value = serde_json::from_str(
        &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "root": root, "config": root.join(".no-mistakes.yml"), "includeSqlVerdicts": true
            }),
        ))
        .unwrap(),
    )
    .unwrap();
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    assert!(
        report["warnings"].as_array().unwrap().is_empty(),
        "{report}"
    );
    let mut offset = 0;
    let mut checked = 0;
    for line in source.split_inclusive('\n') {
        if let Some((code, marker)) = line.split_once("// ") {
            let (start, end, status) = if marker.starts_with("CLEAR_CALL") {
                (offset, offset + code.trim_end().len(), "proven-clear")
            } else {
                let (kind, target) = marker.trim().split_once(':').unwrap();
                let start = offset + code.find(target).unwrap();
                (
                    start,
                    start + target.len(),
                    if kind == "CLEAR" {
                        "proven-clear"
                    } else {
                        "abstain"
                    },
                )
            };
            assert!(
                report["sqlVerdicts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|row| row["file"] == "src/query.mts"
                        && row["span"] == serde_json::json!([start, end])
                        && row["status"] == status),
                "{line}: {report}"
            );
            checked += 1;
        }
        offset += line.len();
    }
    assert_eq!(checked, 5);
}
