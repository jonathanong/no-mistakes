use super::{project::project, tests::profile};
use crate::codebase::{
    check_facts::{
        sql_source_prerequisites::{source_failed, SqlProjection, SqlSourceKey},
        CheckFileFacts,
    },
    postgres::{sql_identifier_syntax::collect as collect_sql_syntax, EmbeddedSqlOptions},
    ts_source::{
        facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
        FileIdMap,
    },
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

fn load(path: &Path, profiles: &[EmbeddedSqlOptions]) -> CheckFileFacts {
    let source = std::fs::read_to_string(path).unwrap();
    crate::ast::with_program(path, &source, |program, _| {
        let ts = collect_file_facts_from_program(
            path,
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
        let sql_identifier_syntax = collect_sql_syntax(program, &ts, &source, profiles);
        CheckFileFacts {
            ts: Arc::new(ts),
            sql_identifier_syntax,
            ..Default::default()
        }
    })
    .unwrap_or_else(|error| {
        let error = error.to_string();
        CheckFileFacts {
            parse_error: Some(error.clone()),
            ts: Arc::new(crate::codebase::ts_source::facts::TsFileFacts {
                parse_error: Some(error),
                ..Default::default()
            }),
            ..Default::default()
        }
    })
}

#[test]
fn low_helper_failures_are_scoped_to_resolved_application_inputs() {
    let root = crate::test_support::rule_fixture_root("postgres-no-offset")
        .join("low-helper-recovery/src");
    let failed_query = root.join("failure-query.mts");
    let unrelated_query = root.join("unrelated-query.mts");
    let excluded_query = root.join("excluded-query.mts");
    let helper = root.join("failure-helper.mts");
    let excluded_helper = root.join("excluded-helper.mts");
    let profile = profile();
    let other_profile = EmbeddedSqlOptions::configured("@example/other", &[]);
    let profiles = [profile.clone(), other_profile.clone()];
    let mut files = FileIdMap::default();
    for path in [
        &failed_query,
        &unrelated_query,
        &excluded_query,
        &helper,
        &excluded_helper,
    ] {
        let mut facts = load(path, &profiles);
        facts.sql_identifier_syntax.low_applications = if path == &failed_query {
            vec![
                (profile.clone(), "postgres-no-offset".into(), 0),
                (other_profile.clone(), "postgres-no-offset".into(), 2),
            ]
        } else if path == &unrelated_query || path == &excluded_helper {
            vec![(profile.clone(), "postgres-no-offset".into(), 1)]
        } else {
            vec![(profile.clone(), "postgres-no-offset".into(), 0)]
        };
        files.insert(path.to_path_buf(), facts);
    }
    assert!(source_failed(&files, &helper));
    assert!(source_failed(&files, &excluded_helper));
    let resolve = |specifier: &str, from: &Path| -> Option<PathBuf> {
        match (specifier, from) {
            ("./failure-helper.mjs", path) if path == failed_query.as_path() => {
                Some(helper.clone())
            }
            ("./excluded-helper.mjs", path) if path == excluded_query.as_path() => {
                Some(excluded_helper.clone())
            }
            _ => None,
        }
    };

    project(&mut files, resolve);

    let failed_key =
        SqlSourceKey::application(SqlProjection::LowSyntax, &profile, "postgres-no-offset", 0);
    assert_eq!(
        files[&failed_query]
            .sql_source_prerequisites
            .failures(&failed_key)
            .cloned()
            .collect::<Vec<_>>(),
        std::slice::from_ref(&helper),
    );
    for (path, key) in [
        (
            &unrelated_query,
            SqlSourceKey::application(SqlProjection::LowSyntax, &profile, "postgres-no-offset", 1),
        ),
        (
            &excluded_query,
            SqlSourceKey::application(SqlProjection::LowSyntax, &profile, "postgres-no-offset", 0),
        ),
        (
            &failed_query,
            SqlSourceKey::application(
                SqlProjection::LowSyntax,
                &other_profile,
                "postgres-no-offset",
                2,
            ),
        ),
    ] {
        assert!(files[path]
            .sql_source_prerequisites
            .failures(&key)
            .next()
            .is_none());
    }
}
