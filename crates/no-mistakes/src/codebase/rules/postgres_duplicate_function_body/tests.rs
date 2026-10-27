use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/duplicate-function-body")
}

fn config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

fn tokens(body: &str, identifiers: bool, raise: bool, keep: &[&str]) -> Vec<String> {
    normalize::normalized_tokens(
        body,
        &normalize::Settings {
            normalize_identifiers: identifiers,
            normalize_raise: raise,
            keep_identifiers: keep.iter().map(|word| word.to_ascii_uppercase()).collect(),
        },
        Some("plpgsql"),
    )
    .unwrap_or_default()
}

const RAISE: &[&str] = &["BEGIN", "RAISE", "EXCEPTION", "?", ";", "END"];

#[test]
fn copied_raise_functions_normalize_to_one_body() {
    assert_eq!(
        tokens(
            "BEGIN RAISE EXCEPTION 'orders rows are immutable' USING ERRCODE = '23514'; END",
            true,
            true,
            &[],
        ),
        RAISE
    );
    assert_eq!(
        tokens(
            "BEGIN RAISE EXCEPTION 'invoices rows are immutable'; END;",
            true,
            true,
            &[]
        ),
        RAISE
    );
    assert_eq!(
        tokens(
            "BEGIN\n  RAISE EXCEPTION '% rows are immutable', TG_TABLE_NAME USING ERRCODE = 'check_violation';\nEND;",
            true,
            true,
            &[],
        ),
        RAISE
    );
}

#[test]
fn normaliser_covers_literals_calls_levels_and_kept_words() {
    assert_eq!(
        tokens("BEGIN PERFORM fn_a(); RETURN NEW; END", true, true, &[]),
        ["BEGIN", "ID1", "fn_a", "(", ")", ";", "RETURN", "NEW", ";", "END"]
    );
    assert_ne!(
        tokens("BEGIN PERFORM fn_a(); END", true, true, &[]),
        tokens("BEGIN PERFORM fn_b(); END", true, true, &[])
    );
    assert_eq!(
        tokens("RETURN 'a'", true, true, &[]),
        tokens("RETURN $$b$$", true, true, &[])
    );
    assert_eq!(tokens("RETURN $$b$$", true, true, &[]), ["RETURN", "'?'"]);
    assert_eq!(
        tokens("RETURN $tag$c$tag$", true, true, &[]),
        ["RETURN", "'?'"]
    );
    assert_eq!(tokens("RETURN 1", true, true, &[]), ["RETURN", "0"]);
    assert_eq!(tokens("RETURN 2.5", true, true, &[]), ["RETURN", "0"]);
    assert_eq!(
        tokens("RETURN NEW.title", true, true, &[]),
        ["RETURN", "NEW", ".", "ID1"]
    );
    assert_eq!(
        tokens("RETURN NEW.\"Body\"", false, true, &[]),
        ["RETURN", "NEW", ".", "\"Body\""]
    );
    assert_eq!(
        tokens("BEGIN RETURN now; END", true, true, &["now"]),
        ["BEGIN", "RETURN", "NOW", ";", "END"]
    );
    assert_ne!(
        tokens("BEGIN RETURN now; END", true, true, &["now"]),
        tokens("BEGIN RETURN today; END", true, true, &["now"])
    );
    assert_eq!(
        tokens(
            "IF TG_TABLE_NAME IS NULL THEN RETURN NEW; END IF",
            true,
            true,
            &[]
        ),
        [
            "IF",
            "TG_TABLE_NAME",
            "IS",
            "NULL",
            "THEN",
            "RETURN",
            "NEW",
            ";",
            "END",
            "IF"
        ]
    );
    for level in ["WARNING", "NOTICE", "INFO", "LOG", "DEBUG", "EXCEPTION"] {
        let body = format!("BEGIN RAISE {level} 'x' USING ERRCODE = '1', format('%s', 1); END");
        assert_eq!(
            tokens(&body, true, true, &[]),
            ["BEGIN", "RAISE", level, "?", ";", "END"]
        );
    }
    assert_eq!(
        tokens("BEGIN RAISE 'plain'; END", true, true, &[]),
        ["BEGIN", "RAISE", "EXCEPTION", "?", ";", "END"]
    );
    assert_ne!(
        tokens("BEGIN RAISE NOTICE 'x'; END", true, false, &[]),
        tokens("BEGIN RAISE EXCEPTION 'x'; END", true, false, &[])
    );
    assert_eq!(
        tokens(
            "BEGIN RAISE EXCEPTION 'a; b'; RETURN 1; END",
            true,
            true,
            &[]
        ),
        [
            "BEGIN",
            "RAISE",
            "EXCEPTION",
            "?",
            ";",
            "RETURN",
            "0",
            ";",
            "END"
        ]
    );
    assert!(tokens("BEGIN RAISE EXCEPTION 'unterminated", true, true, &[]).is_empty());
    assert_eq!(
        tokens("BEGIN RETURN NEW; END;", true, true, &[]),
        tokens("BEGIN RETURN NEW; END", true, true, &[])
    );
}

#[test]
fn option_errors_name_the_field() {
    let cases = [
        ("minClusterSize: 2\n", "option schemaCatalogPath: required"),
        (
            "schemaCatalogPath: ' '\n",
            "option schemaCatalogPath: required",
        ),
        (
            "schemaCatalogPath: rejects.json\nminClusterSize: 1\n",
            "option minClusterSize: must be at least 2",
        ),
        (
            "schemaCatalogPath: rejects.json\nminTokens: 0\n",
            "option minTokens: must be at least 1",
        ),
        (
            "schemaCatalogPath: rejects.json\nallow:\n  - object: function:fn_reject_orders_update\n    reason: ' '\n",
            "option allow: entry function:fn_reject_orders_update needs a reason",
        ),
        (
            "schemaCatalogPath: rejects.json\nallow:\n  - object: nope\n    reason: because\n",
            "option allow: invalid object ref nope",
        ),
        (
            "schemaCatalogPath: rejects.json\nallow:\n  - object: function:fn_reject_orders_update\n    reason: one\n  - object: function:fn_reject_orders_update\n    reason: two\n",
            "option allow: duplicate entry function:fn_reject_orders_update",
        ),
    ];
    for (options, expected) in cases {
        let error = check_with_files(&fixture(), &config(options), &[]).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "{expected} not in {error}"
        );
    }
}

#[test]
fn names_keep_qualifiers_quotes_and_bare_raise() {
    assert_ne!(
        tokens("PERFORM accounting.refresh()", true, true, &[]),
        tokens("PERFORM reporting.refresh()", true, true, &[])
    );
    assert_eq!(
        tokens("SELECT Do_Work()", true, true, &[]),
        tokens("SELECT do_work()", true, true, &[])
    );
    assert_ne!(
        tokens("SELECT \"Do_Work\"()", true, true, &[]),
        tokens("SELECT do_work()", true, true, &[])
    );
    assert_ne!(
        tokens("SELECT MixedCase", false, true, &[]),
        tokens("SELECT \"MixedCase\"", false, true, &[])
    );
    assert_eq!(
        tokens("SELECT MixedCase", false, true, &[]),
        tokens("SELECT mixedcase", false, true, &[])
    );
    assert_ne!(
        tokens("BEGIN RAISE; END", true, true, &[]),
        tokens("BEGIN RAISE EXCEPTION 'x'; END", true, true, &[])
    );
    assert_ne!(
        normalize::normalized_tokens("SELECT raise;", &plain_settings(), Some("sql")),
        normalize::normalized_tokens("SELECT raise + 1;", &plain_settings(), Some("sql"))
    );
    assert_ne!(
        tokens("BEGIN a := 1; b := 2; RETURN a - b; END", true, true, &[]),
        tokens("BEGIN a := 1; b := 2; RETURN b - a; END", true, true, &[])
    );
    assert_eq!(
        tokens("BEGIN a := 1; b := 2; RETURN a - b; END", true, true, &[]),
        tokens("BEGIN x := 1; y := 2; RETURN x - y; END", true, true, &[])
    );
}

#[test]
fn named_argument_tokens_stay_distinct() {
    assert_ne!(
        tokens("SELECT target(left_value => 1)", true, true, &[]),
        tokens("SELECT target(right_value => 1)", true, true, &[])
    );
    assert_ne!(
        tokens("SELECT target(left_value := 1)", true, true, &[]),
        tokens("SELECT target(right_value := 1)", true, true, &[])
    );
}

#[test]
fn positional_parameters_are_not_search_path_quotes() {
    let path = super::search_path::extract("SET search_path TO $1, tenant_a");
    assert!(path.contains("tenant_a"), "{path}");
}

fn plain_settings() -> normalize::Settings {
    normalize::Settings {
        normalize_identifiers: true,
        normalize_raise: true,
        keep_identifiers: Vec::new(),
    }
}
