use super::*;

#[test]
fn async_check_options_preserve_sql_alias_and_default_effort_reports() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/rule-analysis-policy/fixture");
    for mode in ["report", "advise", "ignore"] {
        let execute = |config: String, effort: Option<&str>| {
            let mut options = serde_json::json!({"root": root, "config": config});
            if let Some(effort) = effort {
                options["effort"] = effort.into();
            }
            serde_json::from_str::<serde_json::Value>(
                &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                    .unwrap(),
            )
            .unwrap()
        };
        let default = execute(format!("sql-{mode}.yml"), None);
        assert_eq!(execute(format!("legacy-sql-{mode}.yml"), None), default);
        assert_eq!(execute(format!("sql-{mode}.yml"), Some("max")), default);
        let low = execute(format!("sql-{mode}.yml"), Some("low"));
        assert_eq!(execute(format!("legacy-sql-{mode}.yml"), Some("low")), low);
        // Low deliberately replaces deep inference with static rewrite guidance.
        // Policy routing and finding identity must still match the max tier.
        for field in ["rules", "advisories"] {
            let rows = low[field].as_array().unwrap();
            let expected = default[field].as_array().unwrap();
            assert_eq!(rows.len(), expected.len());
            for (actual, expected) in rows.iter().zip(expected) {
                for key in ["rule", "file", "line"] {
                    assert_eq!(actual[key], expected[key]);
                }
                let message = actual["message"].as_str().unwrap();
                assert!(message.contains("query(\"/* name */ \" + sql)"));
                assert!(message.contains("one local const"));
            }
        }
    }
}

#[test]
fn low_literal_annotation_violations_survive_every_policy_in_the_public_check_api() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads");
    let source = std::fs::read_to_string(root.join("low-missing.mts")).unwrap();
    for policy in ["report", "advise", "ignore"] {
        let execute = |include| {
            let options = serde_json::json!({
                "root": root,
                "config": root.join(format!(".no-mistakes-literals-{policy}.yml")),
                "includeSqlVerdicts": include,
            });
            serde_json::from_str::<serde_json::Value>(
                &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                    .unwrap(),
            )
            .unwrap()
        };
        let plain = execute(false);
        let detailed = execute(true);
        assert_eq!(plain["rules"], detailed["rules"]);
        assert_eq!(plain["advisories"], detailed["advisories"]);
        for line in [3, 4, 5, 7, 8, 11] {
            assert!(
                plain["rules"].as_array().unwrap().iter().any(|finding| {
                    finding["line"] == line
                        && finding["message"]
                            .as_str()
                            .unwrap()
                            .contains("query execution must start with a /* name */ annotation")
                }),
                "{policy}, line {line}: {plain}"
            );
            assert!(!plain["advisories"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["line"] == line));
            let verdict = detailed["sqlVerdicts"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| {
                    let start = row["span"][0].as_u64().unwrap() as usize;
                    source[..start]
                        .bytes()
                        .filter(|byte| *byte == b'\n')
                        .count()
                        + 1
                        == line
                })
                .unwrap();
            assert_eq!(verdict["status"], "violation", "{policy}: {verdict}");
            assert_eq!(verdict["advisory"], false, "{policy}: {verdict}");
        }
        let advice_lines = plain["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["line"].as_u64().unwrap())
            .filter(|line| *line <= 18)
            .collect::<Vec<_>>();
        assert_eq!(
            advice_lines,
            if policy == "advise" {
                (12..=18).collect()
            } else {
                Vec::new()
            }
        );
    }
}

#[test]
fn low_typed_executors_keep_statement_lines_in_the_public_check_api() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/fixture/low-typed-executors");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let options = serde_json::json!({
        "root": root,
        "config": root.join(".no-mistakes-default.yml"),
    });
    let result: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap();
    let rows = result["rules"].as_array().unwrap();
    for rule in [
        "postgres-no-offset",
        "postgres-require-query-annotation",
        "postgres-no-mutable-sql-registry",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-transformed-uuid-predicates",
    ] {
        for (index, line) in source.lines().enumerate() {
            let found = rows
                .iter()
                .any(|row| row["rule"] == rule && row["line"] == index + 1);
            if line.contains("// UNKNOWN")
                || (matches!(
                    rule,
                    "postgres-no-offset"
                        | "postgres-require-query-annotation"
                        | "postgres-no-transformed-uuid-predicates"
                ) && line.contains("// FALLBACK_UNKNOWN"))
                || (rule != "postgres-require-query-annotation"
                    && line.contains("// UNSUPPORTED_TYPE"))
            {
                assert!(found, "{rule}:{}", index + 1);
            }
            if line.contains("// CLEAR")
                || line.starts_with("export function")
                || line.starts_with("type ")
            {
                assert!(!found, "{rule}:{}", index + 1);
            }
            if rule == "postgres-no-offset" && line.contains("// OFFSET") {
                assert!(found, "OFFSET violation must survive the default policy");
            }
        }
    }
}

#[test]
fn low_executor_transports_preserve_native_callsite_advisories() {
    let root = crate::test_support::rule_fixture_root("postgres-no-offset")
        .join("low-executor-transports");
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// ADVISE").then_some(index + 1))
        .collect::<Vec<_>>();
    let options = serde_json::json!({"root":root,"config":root.join(".no-mistakes-low.yml")});
    let report: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap(),
    )
    .unwrap();
    for rule in [
        "postgres-no-offset",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
    ] {
        let mut actual = report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["rule"] == rule)
            .map(|row| row["line"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        actual.sort_unstable();
        actual.dedup();
        assert_eq!(actual, expected, "{rule}: {report}");
    }
    assert!(
        report["warnings"].as_array().unwrap().is_empty(),
        "{report}"
    );
}
