use super::super::{collect, FileFacts, Input};
use crate::codebase::{
    postgres::EmbeddedSqlOptions,
    ts_source::facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
};
use std::path::PathBuf;

fn fixture(name: &str, profiles: &[EmbeddedSqlOptions]) -> (String, FileFacts) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
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
        collect(program, &ts, &source, profiles)
    })
    .unwrap();
    (source, facts)
}

fn profile(names: &[&str], types: &[&str]) -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured(
        "@example/db",
        &names.iter().map(|name| (*name).into()).collect::<Vec<_>>(),
    )
    .with_scoped_executors(
        &[],
        &types.iter().map(|name| (*name).into()).collect::<Vec<_>>(),
    )
}

#[test]
fn executor_choices_keep_sql_and_suppress_only_proven_nonexecuting_uses() {
    let profile = profile(&["read", "write"], &["QueryExecutor", "QueryOptions"]);
    let (source, facts) = fixture("3304-helper-executor-choices.mts", &[profile]);
    let calls = &facts.profiles[0].1;
    let mut checked = 0;
    for (index, line) in source.lines().enumerate() {
        let line_number = (index + 1) as u32;
        let rows = calls
            .iter()
            .filter(|call| call.line == line_number)
            .collect::<Vec<_>>();
        if line.contains("// KNOWN_") {
            let expected = line
                .split('\'')
                .find(|part| part.starts_with("SELECT "))
                .unwrap();
            assert!(
                rows.iter()
                    .any(|call| matches!(&call.input, Input::Sql { text, .. } if text == expected)),
                "{line}: {rows:?}"
            );
            assert!(
                rows.iter()
                    .all(|call| !matches!(call.input, Input::Unknown)),
                "{line}: {rows:?}"
            );
            checked += 1;
        } else if line.contains("// UNKNOWN_") {
            assert!(
                rows.iter().any(|call| matches!(call.input, Input::Unknown)),
                "{line}: {rows:?}"
            );
            checked += 1;
        } else if line.contains("// QUIET_") {
            assert!(
                rows.iter()
                    .all(|call| !matches!(call.input, Input::Unknown)),
                "{line}: {rows:?}"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 22);
}

#[test]
fn typed_forwarding_keeps_other_profile_executor_obligations() {
    let profiles = [
        profile(&["read"], &["QueryExecutor"]),
        profile(&["write"], &["OtherExecutor"]),
    ];
    let (source, facts) = fixture("3304-helper-incompatible-choices.mts", &profiles);
    for (profile, calls) in &facts.profiles {
        for (index, line) in source
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains("// UNKNOWN_"))
        {
            assert!(
                calls
                    .iter()
                    .any(|call| call.line as usize == index + 1
                        && matches!(call.input, Input::Unknown)),
                "{} {line}: {calls:?}",
                profile.executor_names.join(",")
            );
        }
    }
}

#[path = "cov_p2_fixture_tests.rs"]
mod cov_p2;
