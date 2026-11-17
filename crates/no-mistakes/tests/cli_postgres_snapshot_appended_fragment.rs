use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};
use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql, EmbeddedSqlOptions, ExecutedPostgresSource,
};

#[test]
fn postgres_appended_fragment_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "fixture",
        "appended-fragment",
    );
    let reports = assert_saved_fixture_report(
        "postgres_appended_fragment",
        &[(
            "appended_helper_fragment",
            root,
            rules_path(
                "postgres-no-test-state-mutation",
                "configs",
                "appended-fragment.yml",
            ),
        )],
        &[],
        &[],
    );
    let findings = reports["appended_helper_fragment"]["report"]["rules"]
        .as_array()
        .unwrap();
    assert_eq!(findings.len(), 1, "{reports}");
    assert_eq!(findings[0]["file"], "tests/mutation.test.mts");
    assert_eq!(findings[0]["target"], "pg_sleep");
}

#[test]
fn appended_fragment_positive_controls_render_exact_sql() {
    let mut rendered = Vec::new();
    for (rule, fixture, file, executor) in [
        (
            "postgres-lock-ordering",
            "appended-scope-fragment",
            "src/violation.ts",
            "@example/db",
        ),
        (
            "postgres-no-test-state-mutation",
            "appended-fragment",
            "tests/mutation.test.mts",
            "db",
        ),
    ] {
        let path = rules_path(rule, "fixture", fixture).join(file);
        let source = ExecutedPostgresSource {
            file_name: path.to_string_lossy().into_owned(),
            source: std::fs::read_to_string(path).unwrap(),
        };
        let statements =
            extract_executed_postgres_sql(&[source], &EmbeddedSqlOptions::configured(executor, &[]))
                .unwrap();
        assert_eq!(statements.len(), 1, "{fixture}: {statements:?}");
        assert_eq!(statements[0].incomplete_reason, None);
        rendered.push((fixture, statements[0].sql_text.clone().unwrap()));
    }
    insta::with_settings!({
        snapshot_path => "common/snapshots",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(
            "postgres_appended_fragment_rendered_sql",
            serde_json::to_string_pretty(&rendered).unwrap()
        );
    });
}
