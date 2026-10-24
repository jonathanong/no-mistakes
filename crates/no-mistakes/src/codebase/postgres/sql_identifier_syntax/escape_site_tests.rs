use super::*;
fn fixture() -> std::path::PathBuf {
    crate::test_support::rule_fixture_root("postgres-no-offset").join("opaque-escape-sites")
}
#[test]
fn opaque_escapes_anchor_to_calls_without_losing_reference_identity() {
    let root = fixture();
    let options = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join(".no-mistakes-low.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(options).unwrap()).unwrap();
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    let rows = report["advisories"].as_array().unwrap();
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let mut expected = 0;
    for (index, line) in source.lines().enumerate() {
        let wanted = if line.contains("// TWO:") {
            2
        } else if line.contains("// ONE:") {
            1
        } else {
            0
        };
        expected += wanted;
        let count = rows.iter().filter(|row| row["line"] == index + 1).count();
        assert_eq!(count, wanted, "line {}: {report}", index + 1);
    }
    assert_eq!(rows.len(), expected);

    // Reporting lines may coincide, but each unknown retains its original span.
    let path = root.join("src/query.mts");
    crate::ast::with_program(&path, &source, |program, _| {
        use crate::codebase::ts_source::facts::{
            collect_file_facts_from_program, TsFactContext, TsFactPlan,
        };
        let ts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        let profile = EmbeddedSqlOptions::configured("@example/db", &[])
            .with_scoped_executors(&[], &["QueryExecutor".into(), "QueryOptions".into()]);
        let facts = collect(program, &ts, &source, &[profile]);
        let call_line = source
            .lines()
            .position(|line| line.contains("// TWO:"))
            .unwrap() as u32
            + 1;
        let unknown = facts.profiles[0]
            .1
            .iter()
            .filter(|call| call.line == call_line && matches!(call.input, Input::Unknown))
            .collect::<Vec<_>>();
        assert_eq!(unknown.len(), 2);
        assert_ne!(unknown[0].span, unknown[1].span);
        assert!(unknown
            .iter()
            .all(|call| call.span.0 > source.find("helper( // TWO:").unwrap() as u32));
    })
    .unwrap();
}
