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
        .unwrap()
        .incomplete
    };
    let default = execute(None);
    assert!(!default.is_empty());
    assert_eq!(execute(Some(Effort::Max)), default);
    let message = default
        .iter()
        .map(|failure| failure.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(message.contains("options.unanalyzableSql"));
    assert!(message.contains(" REPORT "));
}
