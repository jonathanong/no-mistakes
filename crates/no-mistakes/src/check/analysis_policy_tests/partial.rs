use super::*;

#[test]
fn invalid_legacy_sql_alias_keeps_its_original_parser_error_at_max() {
    let execute = |effort| {
        check_runner::run_all_with_options(
            root(),
            Some("invalid-alias.yml".into()),
            None,
            false,
            false,
            check_runner::AnalysisOptions {
                effort,
                include_sql_verdicts: false,
            },
        )
        .err()
        .unwrap()
        .to_string()
    };
    let message = execute(None);
    assert_eq!(execute(Some(Effort::Max)), message);
    assert!(message.contains("invalid-alias.yml:7:"), "{message}");
    assert!(message.contains("options.unanalyzableSql"), "{message}");
    assert!(message.contains(" REPORT "), "{message}");
}
