use super::*;

#[test]
fn unanalyzable_option_defaults_to_fail_and_rejects_unknown_modes() {
    assert!(
        compile_options(&Options::default())
            .unwrap()
            .fail_unanalyzable
    );
    let ignore = Options {
        unanalyzable_sql: "ignore".to_string(),
        ..Default::default()
    };
    assert!(!compile_options(&ignore).unwrap().fail_unanalyzable);
    let invalid = Options {
        unanalyzable_sql: "sometimes".to_string(),
        ..Default::default()
    };
    let error = compile_options(&invalid).err().unwrap();
    assert!(
        error.to_string().contains(
            "postgres-lock-ordering: unanalyzableSql must be `fail`, `ignore`, or `advise`"
        ),
        "{error}"
    );
}
