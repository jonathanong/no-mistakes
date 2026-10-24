use super::*;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-no-offset").join("template-binds-low")
}
fn inspect(name: &str, check: impl FnOnce(&FileFacts, &str)) {
    let path = fixture().join("src").join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
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
        check(
            &collect(
                program,
                &ts,
                &source,
                &[EmbeddedSqlOptions::configured("@example/db", &[])],
            ),
            &source,
        );
    })
    .unwrap();
}
#[test]
fn low_template_binds_use_exact_library_bindings_and_keep_physical_lines() {
    inspect("binds.mts", |facts, source| {
        let calls = &facts.profiles[0].1;
        assert_eq!(
            calls.len(),
            source.lines().filter(|line| line.contains("read(")).count()
        );
        for call in calls {
            assert!(
                matches!(&call.input, Input::Sql { parameterized: Some(_), holes, .. } if holes.is_empty()),
                "{call:?}"
            );
        }
        for call in calls {
            let Input::Sql {
                text, positions, ..
            } = &call.input
            else {
                panic!()
            };
            for fact in super::super::sql_offset_facts(text).unwrap() {
                let origin = positions
                    .iter()
                    .rev()
                    .find(|position| {
                        (position.sql_line as usize, position.sql_column as usize)
                            <= (fact.line, fact.column)
                    })
                    .unwrap();
                let physical = origin.source_line as usize + fact.line - origin.sql_line as usize;
                assert!(source
                    .lines()
                    .nth(physical - 1)
                    .unwrap()
                    .contains("// offset:"));
            }
        }
        let Input::Sql { text, .. } = &calls.last().unwrap().input else {
            panic!()
        };
        assert!(text.contains("$1"));
        assert!(
            !text.contains('\n'),
            "bind source lines must not change SQL text"
        );
    });
}
#[test]
fn low_template_binds_do_not_accept_dynamic_tags_shadowing_writes_or_builder_mutation() {
    for name in ["unknown.mts", "writes.mts"] {
        inspect(name, |facts, source| {
            for call in &facts.profiles[0].1 {
                let line = source.lines().nth(call.line as usize - 1).unwrap();
                if line.contains("// unknown") {
                    assert!(
                        !matches!(
                            call.input,
                            Input::Sql {
                                parameterized: Some(_),
                                ..
                            }
                        ),
                        "{name}: {call:?}"
                    );
                }
            }
        });
    }
}
#[test]
fn all_three_low_policies_accept_binds_and_keep_actual_dynamic_advice() {
    let root = fixture();
    let options = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join(".no-mistakes-low.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(options).unwrap()).unwrap();
    let advisories = report["advisories"].as_array().unwrap();
    assert!(
        advisories.iter().all(|row| row["file"] != "src/binds.mts"),
        "{report}"
    );
    for rule in [
        "postgres-no-offset",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
    ] {
        assert!(
            advisories
                .iter()
                .any(|row| row["rule"] == rule && row["file"] == "src/unknown.mts"),
            "{report}"
        );
    }
    let findings = report["rules"].as_array().unwrap();
    let source = std::fs::read_to_string(root.join("src/binds.mts")).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// offset:").then_some(index + 1))
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), expected.len(), "{report}");
    for (finding, line) in findings.iter().zip(expected) {
        assert_eq!(finding["rule"], "postgres-no-offset");
        assert_eq!(finding["line"], line);
    }
}
